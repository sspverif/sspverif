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
        ..
    } = inst;

    let fixup = |expr| match expr {
        Expression::Identifier(Identifier::Scalar(id)) => {
            if state.clone().iter().any(|(id_, _)| id == *id_) {
                Expression::Identifier(Identifier::State {
                    name: id,
                    pkgname: name.clone(),
                    compname: comp_name.into(),
                })
            } else if params.clone().iter().any(|(id_, _)| id == *id_) {
                Expression::Identifier(Identifier::Params {
                    name: id,
                    pkgname: name.clone(),
                    compname: comp_name.into(),
                })
            } else {
                Expression::Identifier(Identifier::Local(id))
            }
        }
        Expression::TableAccess(Identifier::Scalar(id), expr) => {
            if state.clone().iter().any(|(id_, _)| id == *id_) {
                Expression::TableAccess(Identifier::State {
                    name: id,
                    pkgname: name.clone(),
                    compname: comp_name.into(),
                }, expr)
            } else if params.clone().iter().any(|(id_, _)| id == *id_) {
                Expression::TableAccess(Identifier::Params {
                    name: id,
                    pkgname: name.clone(),
                    compname: comp_name.into(),
                }, expr)
            } else {
                Expression::Identifier(Identifier::Local(id))
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
                Statement::Assign(id, expr) => {
                    if let Expression::Identifier(id) = fixup(id.to_expression()) {
                        Statement::Assign(id, expr.map(fixup))
                    } else {
                        unreachable!()
                    }
                }
                Statement::IfThenElse(expr, ifcode, elsecode) => Statement::IfThenElse(
                    expr.map(fixup),
                    var_specify_helper(inst, ifcode.clone(), comp_name),
                    var_specify_helper(inst, elsecode.clone(), comp_name),
                ),
                Statement::TableAssign(table, index, expr) => {
                    if let Expression::Identifier(table) = fixup(table.to_expression()) {
                        Statement::TableAssign(table, index.map(fixup), expr.map(fixup))
                    } else {
                        unreachable!()
                    }
                }
            })
            .collect(),
    )
}

fn var_specify(inst: &PackageInstance, comp_name: &str) -> PackageInstance {
    PackageInstance {
        name: inst.name.clone(),
        params: inst.params.clone(),
        pkg: Package {
            params: inst.pkg.params.clone(),
            state: inst.pkg.state.clone(),
            oracles: inst
                .pkg
                .oracles
                .iter()
                .map(|def| OracleDef {
                    sig: def.sig.clone(),
                    code: var_specify_helper(inst, def.code.clone(), comp_name),
                })
                .collect(),
        },
    }
}



#[cfg(test)]
mod test {
    use super::{Transformation,var_specify};
    use crate::expressions::Expression;
    use crate::identifier::Identifier;
    use crate::package::{Package,PackageInstance, OracleDef,OracleSig};
    use crate::statement::{CodeBlock, Statement};
    use crate::types::Type;
    use crate::block;
    use std::collections::HashMap;

    fn generate_code_blocks(source_id: Identifier, target_id:Identifier) -> Vec<(CodeBlock, CodeBlock)>{
        [
            |id:&Identifier| block!{
                    Statement::Assign(id.clone(),
                                      Expression::Sample(Type::Integer))
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
            }
        ].iter().map(|f| (f(&source_id), f(&target_id))).collect()
    }

    #[test]
    fn variable_is_local() {
        let params : HashMap<String, String> = HashMap::new();
        let param_t: Vec<(String, Type)> = Vec::new();
        let state  : Vec<(String, Type)> = Vec::new();

        let source_id = Identifier::Scalar("v".to_string());
        let target_id = Identifier::Local("v".to_string());

        let code = generate_code_blocks(source_id, target_id);
            code.iter().for_each(|c| {
                let res = var_specify(&PackageInstance{
                    params: params.clone(),
                    name: "test".to_string(),
                    pkg: Package{
                        params: param_t.clone(),
                        state: state.clone(),
                        oracles: vec![
                            OracleDef{
                                code: c.0.clone(),
                                sig: OracleSig {
                                    tipe: Type::Empty,
                                    name: "test".to_string(),
                                    args: vec![]
                                }
                            }
                        ]
                    }}, "test");
                assert_eq!(res.pkg.oracles[0].code, c.1)
            })
    }

    #[test]
    fn variable_is_state() {
        let params : HashMap<String, String> = HashMap::new();
        let param_t: Vec<(String, Type)> = Vec::new();
        let mut state  : Vec<(String, Type)> = Vec::new();
        state.push(("v".to_string(), Type::Integer));

        let source_id = Identifier::Scalar("v".to_string());
        let target_id = Identifier::State{name: "v".to_string(), pkgname: "testpkg".to_string(), compname: "testcomp".to_string()};

        let code = generate_code_blocks(source_id, target_id);
        code.iter().for_each(|c| {
            let res = var_specify(&PackageInstance{
                params: params.clone(),
                name: "testpkg".to_string(),
                pkg: Package{
                    params: param_t.clone(),
                    state: state.clone(),
                    oracles: vec![
                        OracleDef{
                            code: c.0.clone(),
                            sig: OracleSig {
                                tipe: Type::Empty,
                                name: "test".to_string(),
                                args: vec![]
                            }
                        }
                    ]
                }}, "testcomp");
            assert_eq!(res.pkg.oracles[0].code, c.1)
        })
    }

    #[test]
    fn variable_is_param() {
        let params : HashMap<String, String> = HashMap::new();
        let mut param_t: Vec<(String, Type)> = Vec::new();
        let state  : Vec<(String, Type)> = Vec::new();
        param_t.push(("v".to_string(), Type::Integer));

        let source_id = Identifier::Scalar("v".to_string());
        let target_id = Identifier::Params{name: "v".to_string(), pkgname: "testpkg".to_string(), compname: "testcomp".to_string()};

        let code = generate_code_blocks(source_id, target_id);
        code.iter().for_each(|c| {
            let res = var_specify(&PackageInstance{
                params: params.clone(),
                name: "testpkg".to_string(),
                pkg: Package{
                    params: param_t.clone(),
                    state: state.clone(),
                    oracles: vec![
                        OracleDef{
                            code: c.0.clone(),
                            sig: OracleSig {
                                tipe: Type::Empty,
                                name: "test".to_string(),
                                args: vec![]
                            }
                        }
                    ]
                }}, "testcomp");
            assert_eq!(res.pkg.oracles[0].code, c.1)
        })
    }
}
