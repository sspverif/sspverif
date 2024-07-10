use miette::SourceSpan;

use super::PackageInstanceTransform;
use crate::expressions::Expression;
use crate::identifier::game_ident::{GameConstIdentifier, GameIdentifier};
use crate::identifier::pkg_ident::PackageIdentifier;
use crate::identifier::proof_ident::{ProofConstIdentifier, ProofIdentifier};
use crate::identifier::Identifier;
use crate::package::{Composition, Edge, Export, OracleDef, OracleSig, PackageInstance};
use crate::statement::{CodeBlock, FilePosition, Statement};
use crate::types::Type;

use std::collections::HashMap;
use std::iter::FromIterator;

pub struct ResolveTypesPackageInstanceTransform;

impl super::PackageInstanceTransform for ResolveTypesPackageInstanceTransform {
    type Aux = ();
    type Err = ResolutionError;

    fn transform_package_instance(
        &self,
        inst: &PackageInstance,
    ) -> std::result::Result<(PackageInstance, Self::Aux), Self::Err> {
        let mut inst = inst.clone();

        let inst_name = &inst.name;
        let type_mapping = HashMap::from_iter(
            inst.types
                .iter()
                .map(|(name, newtipe)| (Type::UserDefined(name.to_string()), newtipe.clone())),
        );

        /*
        Things we need to do here:
        - resolve types for params
        - resolve types for state
        - resolve types for oracles
          - signature
          - code body
        */

        // resolve params
        for (param_name, tipe, file_pos) in &mut inst.pkg.params {
            let place = Place::Param {
                inst_name: inst_name.clone(),
                param_name: param_name.clone(),
            };

            type_walker(&type_mapping, place, file_pos, tipe)?;
        }

        // resolve state
        for (state_name, tipe, file_pos) in &mut inst.pkg.state {
            let place = Place::State {
                inst_name: inst_name.clone(),
                state_name: state_name.clone(),
            };
            type_walker(&type_mapping, place, file_pos, tipe)?;
        }

        // resolve oracle definitions
        for OracleDef {
            sig,
            code,
            file_pos,
        } in &mut inst.pkg.oracles
        {
            let OracleSig {
                name: oracle_name,
                args,
                tipe,
                ..
            } = sig;

            // resolve return type
            let return_place = Place::OracleReturn {
                oracle_name: oracle_name.clone(),
                inst_name: inst_name.clone(),
            };

            type_walker(&type_mapping, return_place, file_pos, tipe)?;

            // resolve oracle arg types
            for (arg_name, tipe) in args {
                let place = Place::OracleArg {
                    oracle_name: oracle_name.clone(),
                    arg_name: arg_name.clone(),
                    inst_name: inst_name.clone(),
                };
                type_walker(&type_mapping, place, file_pos, tipe)?;
            }

            let place = Place::OracleBody {
                inst_name: inst_name.clone(),
                oracle_name: oracle_name.clone(),
            };

            // resolve user-defined types in code blocks
            codeblock_walker(&type_mapping, place, code)?
        }

        // resolve oracle import sigs
        for (
            OracleSig {
                name: oracle_name,
                args,
                tipe,
                ..
            },
            file_pos,
        ) in &mut inst.pkg.imports
        {
            let place = Place::ImportReturn {
                inst_name: inst_name.clone(),
                oracle_name: oracle_name.clone(),
            };

            type_walker(&type_mapping, place, file_pos, tipe)?;

            // resolve oracle arg types
            for (arg_name, tipe) in args {
                let place = Place::ImportArg {
                    oracle_name: oracle_name.clone(),
                    arg_name: arg_name.clone(),
                    inst_name: inst_name.clone(),
                };
                type_walker(&type_mapping, place, file_pos, tipe)?;
            }
        }

        Ok((inst, ()))
    }
}

pub struct ResolveTypesTypeTransform(Place);

impl ResolveTypesTypeTransform {
    pub fn new(place: Place) -> Self {
        Self(place)
    }

    pub fn transform_type(
        &self,
        tipe: &Type,
        file_pos: &SourceSpan,
    ) -> std::result::Result<Type, ResolutionError> {
        let mut tipe = tipe.clone();

        type_walker(&HashMap::new(), self.0.clone(), file_pos, &mut tipe)?;

        Ok(tipe)
    }
}

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
    Types {
        inst_name: String,
        type_name: String,
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

// TODO implement new trait(s)

#[derive(Debug, Clone)]
pub struct ResolutionError {
    pub tipe: Type,
    pub place: Place,
    pub file_pos: FilePosition,
}

impl std::error::Error for ResolutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "couln't resolve {:?} at {:?}", self.tipe, self.place)
    }
}

type Result<T> = std::result::Result<T, ResolutionError>;

pub struct Transformation<'a>(pub &'a Composition);

impl<'a> super::Transformation for Transformation<'a> {
    type Err = ResolutionError;
    type Aux = ();

    fn transform(&self) -> Result<(Composition, ())> {
        let mut game = self.0.clone();
        let pkg_inst_tf = ResolveTypesPackageInstanceTransform;

        /*
        things we need to do in this function:
        - fix the packages in the package instances
        - fix the exports and edges based on the updated packages
        */

        for inst in &mut game.pkgs {
            (*inst, _) = pkg_inst_tf.transform_package_instance(inst)?;
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
            Statement::Abort(_) => {}
            Statement::Return(ret, file_pos) => {
                if let Some(expr) = ret {
                    expression_walker(type_mapping, place.clone(), file_pos, expr)?;
                }
            }
            Statement::Assign(_id, opt_idx, val, file_pos) => {
                if let Some(idx) = opt_idx {
                    expression_walker(type_mapping, place.clone(), file_pos, idx)?;
                }
                expression_walker(type_mapping, place.clone(), file_pos, val)?;
            }
            Statement::Parse(_ids, expr, file_pos) => {
                expression_walker(type_mapping, place.clone(), file_pos, expr)?;
            }
            Statement::IfThenElse(cond, ifcode, elsecode, file_pos) => {
                expression_walker(type_mapping, place.clone(), file_pos, cond)?;
                codeblock_walker(type_mapping, place.clone(), ifcode)?;
                codeblock_walker(type_mapping, place.clone(), elsecode)?;
            }
            Statement::For(_, lower_bound, upper_bound, body, file_pos) => {
                expression_walker(type_mapping, place.clone(), file_pos, lower_bound)?;
                expression_walker(type_mapping, place.clone(), file_pos, upper_bound)?;
                codeblock_walker(type_mapping, place.clone(), body)?;
            }
            Statement::Sample(_id, opt_idx, _, tipe, file_pos) => {
                if let Some(idx) = opt_idx {
                    expression_walker(type_mapping, place.clone(), file_pos, idx)?;
                }
                type_walker(type_mapping, place.clone(), file_pos, tipe)?;
            }
            Statement::InvokeOracle {
                opt_idx,
                args,
                tipe,
                file_pos,
                ..
            } => {
                if let Some(idx) = opt_idx {
                    expression_walker(type_mapping, place.clone(), file_pos, idx)?;
                }
                for arg in args {
                    expression_walker(type_mapping, place.clone(), file_pos, arg)?;
                }

                if let Some(tipe) = tipe {
                    type_walker(type_mapping, place.clone(), file_pos, tipe)?;
                }
            }
        }
    }

    Ok(())
}

fn expression_walker(
    type_mapping: &HashMap<Type, Type>,
    place: Place,
    file_pos: &SourceSpan,
    expr: &mut Expression,
) -> Result<()> {
    let mut result = Ok(());

    let mut visitor = |expr: &mut Expression| {
        if result.is_err() {
            return false;
        }

        result = match expr {
            Expression::Identifier(ident) => {
                ident_walker(type_mapping, place.clone(), file_pos, ident)
            }
            Expression::None(tipe) => type_walker(type_mapping, place.clone(), file_pos, tipe),
            _ => Ok(()),
        };

        true
    };

    expr.walk(&mut visitor);

    result
}
fn ident_walker(
    type_mapping: &HashMap<Type, Type>,
    place: Place,
    file_pos: &SourceSpan,
    ident: &mut Identifier,
) -> Result<()> {
    let tipe = match ident {
        Identifier::PackageIdentifier(pkg_ident) => {
            // these are always Integer, skip
            if matches!(
                pkg_ident,
                PackageIdentifier::CodeLoopVar(_) | PackageIdentifier::ImportsLoopVar(_)
            ) {
                return Ok(());
            }
            pkg_ident.type_mut()
        }
        Identifier::GameIdentifier(GameIdentifier::Const(GameConstIdentifier { tipe, .. })) => tipe,
        Identifier::GameIdentifier(GameIdentifier::LoopVar(_)) => {
            // these are always Integer, skip
            return Ok(());
        }
        Identifier::ProofIdentifier(ProofIdentifier::Const(ProofConstIdentifier {
            tipe, ..
        })) => tipe,
        Identifier::ProofIdentifier(ProofIdentifier::LoopVar(_)) => {
            // these are always Integer, skip
            return Ok(());
        }
        Identifier::Local(_) => panic!("shouldn't use Identifier::Local, but found {:?}", ident),
        Identifier::Scalar(_) => panic!("shouldn't use Identifier::Scalar, but found {:?}", ident),
        Identifier::Parameter(_) => {
            panic!("shouldn't use Identifier::Parameter, but found {:?}", ident)
        }
        Identifier::GameInstanceConst(_) => panic!(
            "shouldn't use Identifier::GameInstanceConst, but found {:?}",
            ident
        ),
    };

    type_walker(type_mapping, place, file_pos, tipe)
}

fn type_walker(
    type_mapping: &HashMap<Type, Type>,
    place: Place,
    file_pos: &SourceSpan,
    tipe: &mut Type,
) -> Result<()> {
    match tipe {
        Type::UserDefined(_) => {
            if let Some(resolved) = type_mapping.get(tipe) {
                *tipe = resolved.clone();

                // the resolved value may contain user-defined types itself
                type_walker(type_mapping, place, file_pos, tipe)
            } else {
                panic!(
                    r#"
                return Err(ResolutionError {{
                    tipe     # {tipe:?}
                    place,   # {place:?}
                    file_pos # {file_pos:?}
                }})"#,
                    tipe = tipe,
                    place = place,
                    file_pos = file_pos
                );
            }
        }
        Type::Table(key_type, value_type) => {
            type_walker(type_mapping, place.clone(), file_pos, key_type.as_mut())?;
            type_walker(type_mapping, place, file_pos, value_type.as_mut())
        }
        Type::Maybe(tipe) => type_walker(type_mapping, place, file_pos, tipe.as_mut()),
        Type::Tuple(types) => {
            for tipe in types {
                type_walker(type_mapping, place.clone(), file_pos, tipe)?;
            }

            Ok(())
        }
        Type::Fn(args, ret) => {
            for arg in args {
                type_walker(type_mapping, place.clone(), file_pos, arg)?;
            }

            type_walker(type_mapping, place.clone(), file_pos, ret)
        }
        _ => Ok(()),
    }
}
