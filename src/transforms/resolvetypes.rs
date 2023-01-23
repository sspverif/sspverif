use crate::expressions::Expression;
use crate::package::{Composition, Edge, Export, OracleDef, OracleSig};
use crate::statement::{CodeBlock, Statement};
use crate::types::Type;

use std::collections::HashMap;

pub struct Transformation<'a>(pub &'a Composition);

#[derive(Debug, Clone)]
pub enum Place {
    Param {
        inst_name: String,
        param_name: String,
    },
    State {
        inst_name: String,
        state_name: String,
    },
    ImportArg {
        inst_name: String,
        oracle_name: String,
        arg_name: String,
    },
    ImportReturn {
        inst_name: String,
        oracle_name: String,
    },
    OracleArg {
        inst_name: String,
        oracle_name: String,
        arg_name: String,
    },
    OracleReturn {
        inst_name: String,
        oracle_name: String,
    },
    OracleBody {
        inst_name: String,
        oracle_name: String,
    },
}

#[derive(Debug, Clone)]
pub struct ResolutionError {
    pub tipe: Type,
    pub place: Place,
}

type Result<T> = std::result::Result<T, ResolutionError>;

impl<'a> super::Transformation for Transformation<'a> {
    type Err = ResolutionError;
    type Aux = ();

    fn transform(&self) -> Result<(Composition, ())> {
        let mut game = self.0.clone();

        /*
        things we need to do in this function:
        - fix the packages in the package instances
        - fix the exports and edges based on the updated packages
        */

        for inst in &mut game.pkgs {
            let inst_name = &inst.name;
            let type_mapping = &inst.types;

            /*
            Things we need to do here:
            - resolve types for params
            - resolve types for state
            - resolve types for oracles
              - signature
              - code body
            */

            // resolve params
            for (param_name, tipe) in &mut inst.pkg.params {
                let place = Place::Param {
                    inst_name: inst_name.clone(),
                    param_name: param_name.clone(),
                };

                type_walker(type_mapping, place, tipe)?;
            }

            // resolve state
            for (state_name, tipe) in &mut inst.pkg.state {
                let place = Place::State {
                    inst_name: inst_name.clone(),
                    state_name: state_name.clone(),
                };
                type_walker(type_mapping, place, tipe)?;
            }

            // resolve oracle definitions
            for OracleDef { sig, code } in &mut inst.pkg.oracles {
                let OracleSig {
                    name: oracle_name,
                    args,
                    tipe,
                } = sig;

                // resolve return type
                let return_place = Place::OracleReturn {
                    oracle_name: oracle_name.clone(),
                    inst_name: inst_name.clone(),
                };

                type_walker(type_mapping, return_place, tipe)?;

                // resolve oracle arg types
                for (arg_name, tipe) in args {
                    let place = Place::OracleArg {
                        oracle_name: oracle_name.clone(),
                        arg_name: arg_name.clone(),
                        inst_name: inst_name.clone(),
                    };
                    type_walker(type_mapping, place, tipe)?;
                }

                let place = Place::OracleBody {
                    inst_name: inst_name.clone(),
                    oracle_name: oracle_name.clone(),
                };

                // resolve user-defined types in code blocks
                codeblock_walker(&type_mapping, place, code)?
            }

            // resolve oracle import sigs
            for OracleSig {
                name: oracle_name,
                args,
                tipe,
            } in &mut inst.pkg.imports
            {
                let place = Place::ImportReturn {
                    inst_name: inst_name.clone(),
                    oracle_name: oracle_name.clone(),
                };

                type_walker(type_mapping, place, tipe)?;

                // resolve oracle arg types
                for (arg_name, tipe) in args {
                    let place = Place::ImportArg {
                        oracle_name: oracle_name.clone(),
                        arg_name: arg_name.clone(),
                        inst_name: inst_name.clone(),
                    };
                    type_walker(type_mapping, place, tipe)?;
                }
            }
        }

        // resolve the signatures in the exports
        for Export(inst_offs, osig) in &mut game.exports {
            let oracle_name = &osig.name;

            let odef = game.pkgs[*inst_offs]
                .pkg
                .oracles
                .iter()
                .find(|odef| &odef.sig.name == oracle_name)
                .unwrap();
            *osig = odef.sig.clone();
        }

        // resolve the signatures in the edges
        for Edge(_, inst_offs, osig) in &mut game.edges {
            let oracle_name = &osig.name;

            let odef = game.pkgs[*inst_offs]
                .pkg
                .oracles
                .iter()
                .find(|odef| &odef.sig.name == oracle_name)
                .unwrap();
            *osig = odef.sig.clone();
        }

        Ok((game, ()))
    }
}

fn codeblock_walker(
    type_mapping: &HashMap<Type, Type>,
    place: Place,
    code: &mut CodeBlock,
) -> Result<()> {
    for stmt in &mut code.0 {
        match stmt {
            Statement::Abort => {}
            Statement::Return(ret) => {
                if let Some(expr) = ret {
                    expression_walker(type_mapping, place.clone(), expr)?;
                }
            }
            Statement::Assign(_id, opt_idx, val) => {
                if let Some(idx) = opt_idx {
                    expression_walker(type_mapping, place.clone(), idx)?;
                }
                expression_walker(type_mapping, place.clone(), val)?;
            }
            Statement::Parse(_ids, expr) => {
                expression_walker(type_mapping, place.clone(), expr)?;
            }
            Statement::IfThenElse(cond, ifcode, elsecode) => {
                expression_walker(type_mapping, place.clone(), cond)?;
                codeblock_walker(type_mapping, place.clone(), ifcode)?;
                codeblock_walker(type_mapping, place.clone(), elsecode)?;
            }
            Statement::Sample(_id, opt_idx, _, tipe) => {
                if let Some(idx) = opt_idx {
                    expression_walker(type_mapping, place.clone(), idx)?;
                }
                type_walker(type_mapping, place.clone(), tipe)?;
            }
            Statement::InvokeOracle {
                opt_idx,
                args,
                tipe,
                ..
            } => {
                if let Some(idx) = opt_idx {
                    expression_walker(type_mapping, place.clone(), idx)?;
                }
                for arg in args {
                    expression_walker(type_mapping, place.clone(), arg)?;
                }

                if let Some(tipe) = tipe {
                    type_walker(type_mapping, place.clone(), tipe)?;
                }
            }
        }
    }

    Ok(())
}

fn expression_walker(
    type_mapping: &HashMap<Type, Type>,
    place: Place,
    expr: &mut Expression,
) -> Result<()> {
    let mut result = Ok(());

    let mut visitor = |expr: &mut Expression| {
        if result.is_err() {
            return false;
        }

        result = match expr {
            Expression::Typed(tipe, _expr) => type_walker(type_mapping, place.clone(), tipe),
            Expression::None(tipe) => type_walker(type_mapping, place.clone(), tipe),
            _ => Ok(()),
        };

        return true;
    };

    expr.walk(&mut visitor);

    result
}

fn type_walker(type_mapping: &HashMap<Type, Type>, place: Place, tipe: &mut Type) -> Result<()> {
    match tipe {
        Type::UserDefined(_) => {
            if let Some(resolved) = type_mapping.get(tipe) {
                *tipe = resolved.clone();

                // the resolved value may contain user-defined types itself
                type_walker(type_mapping, place, tipe)
            } else {
                return Err(ResolutionError {
                    tipe: tipe.clone(),
                    place,
                });
            }
        }
        Type::Table(key_type, value_type) => {
            type_walker(type_mapping, place.clone(), key_type.as_mut())?;
            type_walker(type_mapping, place, value_type.as_mut())
        }
        Type::Maybe(tipe) => type_walker(type_mapping, place, tipe.as_mut()),
        Type::Tuple(types) => {
            for tipe in types {
                type_walker(type_mapping, place.clone(), tipe)?;
            }

            Ok(())
        }
        Type::Fn(args, ret) => {
            for arg in args {
                type_walker(type_mapping, place.clone(), arg)?;
            }

            type_walker(type_mapping, place.clone(), ret)
        }
        _ => Ok(()),
    }
}
