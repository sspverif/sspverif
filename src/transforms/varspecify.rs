use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::{Composition, OracleDef, Package, PackageInstance};
use crate::statement::{CodeBlock, Statement};

pub struct Transformation<'a>(pub &'a Composition);

impl<'a> super::Transformation for Transformation<'a> {
    type Err = ();
    type Aux = ();

    fn transform(&self) -> Result<(Composition, ()), ()> {
        Ok((
            Composition {
                pkgs: self
                    .0
                    .pkgs
                    .iter()
                    .map(|inst| var_specify(inst, &self.0.name))
                    .collect(),
                ..self.0.clone()
            },
            (),
        ))
    }
}

fn var_specify_helper(inst: &PackageInstance, block: CodeBlock, comp_name: &str) -> CodeBlock {
    let PackageInstance {
        name,
        pkg: Package { state, params, .. },
        params: inst_params,
        ..
    } = inst;

    let fixup = |expr| match expr {
        Expression::FnCall(Identifier::Scalar(id), args) => {
            if state.clone().iter().any(|(id_, _)| id == *id_) {
                Expression::FnCall(
                    Identifier::State {
                        name: id,
                        pkgname: name.clone(),
                        compname: comp_name.into(),
                    },
                    args,
                )
            } else if params.clone().iter().any(|(id_, _name_in_comp)| id == *id_) {
                Expression::FnCall(
                    Identifier::Params {
                        name_in_pkg: id.clone(),
                        pkgname: name.clone(),

                        name_in_comp: inst_params[&id].clone(),
                        compname: comp_name.into(),
                    },
                    args,
                )
            } else {
                Expression::FnCall(Identifier::Local(id), args)
            }
        }
        Expression::Identifier(Identifier::Scalar(id)) => {
            if state.clone().iter().any(|(id_, _)| id == *id_) {
                Expression::Identifier(Identifier::State {
                    name: id,
                    pkgname: name.clone(),
                    compname: comp_name.into(),
                })
            } else if params.clone().iter().any(|(id_, _name_in_comp)| id == *id_) {
                Expression::Identifier(Identifier::Params {
                    name_in_pkg: id.clone(),
                    pkgname: name.clone(),

                    name_in_comp: inst_params[&id].clone(),
                    compname: comp_name.into(),
                })
            } else {
                Expression::Identifier(Identifier::Local(id))
            }
        }
        Expression::TableAccess(Identifier::Scalar(id), expr) => {
            if state.clone().iter().any(|(id_, _)| id == *id_) {
                Expression::TableAccess(
                    Identifier::State {
                        name: id,
                        pkgname: name.clone(),
                        compname: comp_name.into(),
                    },
                    expr,
                )
            } else if params.clone().iter().any(|(id_, _)| id == *id_) {
                Expression::TableAccess(
                    Identifier::Params {
                        name_in_pkg: id.clone(),
                        pkgname: name.clone(),

                        name_in_comp: inst_params[&id].clone(),
                        compname: comp_name.into(),
                    },
                    expr,
                )
            } else {
                Expression::TableAccess(Identifier::Local(id), expr)
            }
        }
        _ => expr,
    };
    CodeBlock(
        block
            .0
            .iter()
            .map(|stmt| match stmt {
                Statement::Abort => Statement::Abort,
                Statement::Return(None) => Statement::Return(None),
                Statement::Return(Some(expr)) => Statement::Return(Some(expr.map(fixup))),
                Statement::Assign(id, None, expr) => {
                    if let Expression::Identifier(id) = fixup(id.to_expression()) {
                        Statement::Assign(id, None, expr.map(fixup))
                    } else {
                        unreachable!()
                    }
                }
                Statement::Assign(table, Some(index), expr) => {
                    if let Expression::Identifier(table) = fixup(table.to_expression()) {
                        Statement::Assign(table, Some(index.map(fixup)), expr.map(fixup))
                    } else {
                        unreachable!()
                    }
                }
                Statement::Sample(id, opt_idx, sample_id, t) => {
                    let opt_idx = opt_idx.clone().map(|expr| expr.map(fixup));
                    if let Expression::Identifier(id) = fixup(id.to_expression()) {
                        Statement::Sample(id, opt_idx, *sample_id, t.clone())
                    } else {
                        unreachable!()
                    }
                }
                Statement::Parse(idents, expr) => {
                    let idents = idents
                        .iter()
                        .map(|id| {
                            if let Expression::Identifier(id) = fixup(id.to_expression()) {
                                id
                            } else {
                                unreachable!()
                            }
                        })
                        .collect();

                    Statement::Parse(idents, expr.map(fixup))
                }
                Statement::InvokeOracle {
                    id,
                    opt_idx,
                    name,
                    args,
                    target_inst_name,
                    tipe,
                } => {
                    let opt_idx = opt_idx.clone().map(|expr| expr.map(fixup));
                    let args = args.iter().map(|expr| expr.map(fixup)).collect();
                    if let Expression::Identifier(id) = fixup(id.to_expression()) {
                        Statement::InvokeOracle {
                            id,
                            opt_idx,
                            name: name.clone(),
                            args,
                            target_inst_name: target_inst_name.clone(),
                            tipe: tipe.clone(),
                        }
                    } else {
                        unreachable!()
                    }
                }
                Statement::IfThenElse(expr, ifcode, elsecode) => Statement::IfThenElse(
                    expr.map(fixup),
                    var_specify_helper(inst, ifcode.clone(), comp_name),
                    var_specify_helper(inst, elsecode.clone(), comp_name),
                ),
            })
            .collect(),
    )
}

fn var_specify(inst: &PackageInstance, comp_name: &str) -> PackageInstance {
    PackageInstance {
        pkg: Package {
            oracles: inst
                .pkg
                .oracles
                .iter()
                .map(|def| OracleDef {
                    sig: def.sig.clone(),
                    code: var_specify_helper(inst, def.code.clone(), comp_name),
                })
                .collect(),
            ..inst.pkg.clone()
        },
        ..inst.clone()
    }
}

#[cfg(test)]
mod test {
    use super::var_specify;
    use crate::block;
    use crate::expressions::Expression;
    use crate::identifier::Identifier;
    use crate::package::{OracleDef, OracleSig, Package, PackageInstance};
    use crate::statement::{CodeBlock, Statement};
    use crate::types::Type;
    use std::collections::HashMap;
    use std::iter::FromIterator;

    fn generate_code_blocks(
        source_id: Identifier,
        target_id: Identifier,
    ) -> Vec<(CodeBlock, CodeBlock)> {
        [
            |id:&Identifier| block!{
                Statement::Sample(id.clone(), None, None, Type::Integer)
            },
            |id:&Identifier| block!{
                Statement::IfThenElse(
                    Expression::new_equals(vec![&(id.clone().to_expression()),
                                                &(Expression::IntegerLiteral("5".to_string()))]),
                    block!{
                        Statement::Abort
                    },
                    block!{
                        Statement::Abort
                    })
            },
            |id:&Identifier| block!{
                Statement::IfThenElse(
                    Expression::new_equals(vec![&(Expression::IntegerLiteral("5".to_string())),
                                                &(id.clone().to_expression())]),
                    block!{
                        Statement::Abort
                    },
                    block!{
                        Statement::Abort
                    })
            },
            |id:&Identifier| block!{
                Statement::IfThenElse(
                    Expression::new_equals(vec![&(Expression::IntegerLiteral("5".to_string())),
                                                &(Expression::IntegerLiteral("5".to_string()))]),
                    block!{
                        Statement::Return(Some(id.clone().to_expression()))
                    },
                    block!{
                        Statement::Abort
                    })

            },
            |id:&Identifier| block!{
                Statement::IfThenElse(
                    Expression::new_equals(vec![&(Expression::IntegerLiteral("5".to_string())),
                                                &(Expression::IntegerLiteral("5".to_string()))]),
                    block!{
                        Statement::Abort
                    },
                    block!{
                        Statement::Return(Some(id.clone().to_expression()))
                    })
            },
        ].iter().map(|f| (f(&source_id), f(&target_id))).collect()
    }

    #[test]
    fn variable_is_local() {
        let params: HashMap<String, String> = HashMap::new();
        let param_t: Vec<(String, Type)> = Vec::new();
        let state: Vec<(String, Type)> = Vec::new();

        let source_id = Identifier::Scalar("v".to_string());
        let target_id = Identifier::Local("v".to_string());

        let code = generate_code_blocks(source_id, target_id);
        code.iter().for_each(|c| {
            let res = var_specify(
                &PackageInstance {
                    params: params.clone(),
                    name: "test".to_string(),
                    pkg: Package {
                        name: "testpkg".to_string(),
                        params: param_t.clone(),
                        state: state.clone(),
                        imports: vec![],
                        oracles: vec![OracleDef {
                            code: c.0.clone(),
                            sig: OracleSig {
                                tipe: Type::Empty,
                                name: "test".to_string(),
                                args: vec![],
                            },
                        }],
                    },
                },
                "test",
            );
            assert_eq!(res.pkg.oracles[0].code, c.1)
        })
    }

    #[test]
    fn variable_is_state() {
        let params: HashMap<String, String> = HashMap::new();
        let param_t: Vec<(String, Type)> = Vec::new();
        let mut state: Vec<(String, Type)> = Vec::new();
        state.push(("v".to_string(), Type::Integer));

        let source_id = Identifier::Scalar("v".to_string());
        let target_id = Identifier::State {
            name: "v".to_string(),
            pkgname: "testpkg".to_string(),
            compname: "testcomp".to_string(),
        };

        let code = generate_code_blocks(source_id, target_id);
        code.iter().for_each(|c| {
            let res = var_specify(
                &PackageInstance {
                    params: params.clone(),
                    name: "testpkg".to_string(),
                    pkg: Package {
                        name: "testpkg".to_string(),
                        params: param_t.clone(),
                        state: state.clone(),
                        imports: vec![],
                        oracles: vec![OracleDef {
                            code: c.0.clone(),
                            sig: OracleSig {
                                tipe: Type::Empty,
                                name: "test".to_string(),
                                args: vec![],
                            },
                        }],
                    },
                },
                "testcomp",
            );
            assert_eq!(res.pkg.oracles[0].code, c.1)
        })
    }

    #[test]
    fn variable_is_param() {
        let params: HashMap<String, String> =
            HashMap::from_iter(vec![("v".to_string(), "v".to_string())]);
        let mut param_t: Vec<(String, Type)> = Vec::new();
        let state: Vec<(String, Type)> = Vec::new();
        param_t.push(("v".to_string(), Type::Integer));

        let source_id = Identifier::Scalar("v".to_string());
        let target_id = Identifier::Params {
            name_in_pkg: "v".to_string(),
            pkgname: "testpkg".to_string(),

            name_in_comp: "v".to_string(),
            compname: "testcomp".to_string(),
        };

        let code = generate_code_blocks(source_id, target_id);
        code.iter().for_each(|c| {
            let res = var_specify(
                &PackageInstance {
                    params: params.clone(),
                    name: "testpkg".to_string(),
                    pkg: Package {
                        name: "testpkg".to_string(),
                        params: param_t.clone(),
                        state: state.clone(),
                        imports: vec![],
                        oracles: vec![OracleDef {
                            code: c.0.clone(),
                            sig: OracleSig {
                                tipe: Type::Empty,
                                name: "test".to_string(),
                                args: vec![],
                            },
                        }],
                    },
                },
                "testcomp",
            );
            assert_eq!(res.pkg.oracles[0].code, c.1)
        })
    }
}
