use std::convert::Infallible;

use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::{Composition, OracleDef, Package, PackageInstance};
use crate::proof::GameInstance;
use crate::split::SplitOracleDef;
use crate::statement::{CodeBlock, FilePosition, Statement};
use crate::types::Type;
use crate::util::resolver::{Resolver, SliceResolver};

pub struct Transformation<'a>(pub &'a Composition);

/*
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
*/

// TODO: add support for resolving to expression literals
fn resolve_var(
    pkg_state: &[(String, Type, FilePosition)],
    pkg_inst_params: &[(String, Expression)],
    game_inst_params: &[(String, Expression)],
    name: String,
    pkg_name: &str,
    pkg_inst_name: &str,
    game_name: &str,
    game_inst_name: &str,
) -> Identifier {
    let pkg_state = SliceResolver(pkg_state);
    let pkg_inst_params = SliceResolver(pkg_inst_params);
    let game_inst_params = SliceResolver(game_inst_params);

    if let Some(_) = pkg_state.resolve_value(&name) {
        Identifier::State {
            name: name.to_string(),
            pkg_inst_name: pkg_inst_name.to_string(),
            game_inst_name: game_inst_name.to_string(),
        }
    } else if let Some((_, expr)) = &pkg_inst_params.resolve_value(&name) {
        let id = if let Expression::Identifier(id) = expr {
            id
        } else {
            unreachable!()
        };

        let id_in_proof = if let Some((_, Expression::Identifier(id_in_proof))) =
            game_inst_params.resolve_value(&id.ident())
        {
            id_in_proof
        } else {
            unreachable!()
        };

        Identifier::Parameter {
            name_in_pkg: name.to_string(),
            pkgname: pkg_name.to_string(),
            name_in_comp: id.ident(),
            name_in_proof: id_in_proof.ident(),
            game_inst_name: game_inst_name.to_string(),
        }
    } else {
        Identifier::Local(name)
    }
}

fn var_specify_helper(
    game_inst: &GameInstance,
    pkg_inst: &PackageInstance,
    block: CodeBlock,
) -> CodeBlock {
    let PackageInstance {
        name,
        pkg:
            Package {
                state: pkg_state,
                name: pkg_name,
                ..
            },
        params: pkg_inst_params,
        ..
    } = pkg_inst;

    let game_inst_params = game_inst.consts();

    let fixup = |expr| match expr {
        Expression::FnCall(Identifier::Scalar(id), args) => Expression::FnCall(
            resolve_var(
                pkg_state,
                pkg_inst_params,
                game_inst_params,
                id,
                pkg_name,
                &name,
                game_inst.game_name(),
                game_inst.name(),
            ),
            args,
        ),
        Expression::Identifier(Identifier::Scalar(id)) => Expression::Identifier(resolve_var(
            pkg_state,
            pkg_inst_params,
            game_inst_params,
            id,
            pkg_name,
            name,
            game_inst.game_name(),
            game_inst.name(),
        )),
        Expression::TableAccess(Identifier::Scalar(id), expr) => Expression::TableAccess(
            resolve_var(
                pkg_state,
                pkg_inst_params,
                game_inst_params,
                id,
                pkg_name,
                name,
                game_inst.game_name(),
                game_inst.name(),
            ),
            expr,
        ),
        _ => expr,
    };
    CodeBlock(
        block
            .0
            .iter()
            .map(|stmt| match stmt {
                Statement::Abort(file_pos) => Statement::Abort(file_pos.clone()),
                Statement::Return(None, file_pos) => Statement::Return(None, file_pos.clone()),
                Statement::Return(Some(expr), file_pos) => {
                    Statement::Return(Some(expr.map(fixup)), file_pos.clone())
                }
                Statement::Assign(id, None, expr, file_pos) => {
                    if let Expression::Identifier(id) = fixup(id.to_expression()) {
                        Statement::Assign(id, None, expr.map(fixup), file_pos.clone())
                    } else {
                        unreachable!()
                    }
                }
                Statement::Assign(table, Some(index), expr, file_pos) => {
                    if let Expression::Identifier(table) = fixup(table.to_expression()) {
                        Statement::Assign(
                            table,
                            Some(index.map(fixup)),
                            expr.map(fixup),
                            file_pos.clone(),
                        )
                    } else {
                        unreachable!()
                    }
                }
                Statement::Sample(id, opt_idx, sample_id, t, file_pos) => {
                    let opt_idx = opt_idx.clone().map(|expr| expr.map(fixup));
                    if let Expression::Identifier(id) = fixup(id.to_expression()) {
                        Statement::Sample(id, opt_idx, *sample_id, t.clone(), file_pos.clone())
                    } else {
                        unreachable!()
                    }
                }
                Statement::Parse(idents, expr, file_pos) => {
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

                    Statement::Parse(idents, expr.map(fixup), file_pos.clone())
                }
                Statement::InvokeOracle {
                    id,
                    opt_idx,
                    name,
                    args,
                    target_inst_name,
                    tipe,
                    file_pos,
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
                            file_pos: file_pos.clone(),
                        }
                    } else {
                        unreachable!()
                    }
                }
                Statement::IfThenElse(expr, ifcode, elsecode, file_pos) => Statement::IfThenElse(
                    expr.map(fixup),
                    var_specify_helper(game_inst, pkg_inst, ifcode.clone()),
                    var_specify_helper(game_inst, pkg_inst, elsecode.clone()),
                    file_pos.clone(),
                ),
                Statement::For(ident, lower_bound, upper_bound, body, file_pos) => {
                    let resolved_ident =
                        if let Expression::Identifier(ident) = fixup(ident.to_expression()) {
                            ident
                        } else {
                            unreachable!()
                        };
                    Statement::For(
                        resolved_ident,
                        lower_bound.map(fixup),
                        upper_bound.map(fixup),
                        var_specify_helper(game_inst, pkg_inst, body.clone()),
                        file_pos.clone(),
                    )
                }
            })
            .collect(),
    )
}

fn var_specify_pkg_inst(game_inst: &GameInstance, pkg_inst: &PackageInstance) -> PackageInstance {
    PackageInstance {
        pkg: Package {
            oracles: pkg_inst
                .pkg
                .oracles
                .iter()
                .map(|def| OracleDef {
                    sig: def.sig.clone(),
                    code: var_specify_helper(game_inst, pkg_inst, def.code.clone()),
                    file_pos: def.file_pos.clone(),
                })
                .collect(),
            split_oracles: pkg_inst
                .pkg
                .split_oracles
                .iter()
                .map(|def| SplitOracleDef {
                    sig: def.sig.clone(),
                    code: var_specify_helper(game_inst, pkg_inst, def.code.clone()),
                    // TODO add file_pos to this structure
                })
                .collect(),
            ..pkg_inst.pkg.clone()
        },
        ..pkg_inst.clone()
    }
}

pub fn var_specify_game_inst(game_inst: &GameInstance) -> Result<Composition, Infallible> {
    game_inst
        .game()
        .map_pkg_inst(|pkg_inst| Ok(var_specify_pkg_inst(game_inst, pkg_inst)))
}

/*
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
        let params: HashMap<String, Expression> = HashMap::new();
        let param_t: Vec<(String, Type)> = Vec::new();
        let state: Vec<(String, Type)> = Vec::new();
        let types: Vec<Type> = Vec::new();

        let source_id = Identifier::Scalar("v".to_string());
        let target_id = Identifier::Local("v".to_string());

        let code = generate_code_blocks(source_id, target_id);
        code.iter().for_each(|c| {
            let res = var_specify(
                &PackageInstance {
                    params: params.clone().into_iter().collect(),
                    types: vec![],
                    name: "test".to_string(),
                    pkg: Package {
                        name: "testpkg".to_string(),
                        params: param_t.clone(),
                        types: types.clone(),
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
        let params: HashMap<String, Expression> = HashMap::new();
        let param_t: Vec<(String, Type)> = Vec::new();
        let types: Vec<Type> = Vec::new();
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
                    params: params.clone().into_iter().collect(),
                    types: vec![],
                    name: "testpkg".to_string(),
                    pkg: Package {
                        name: "testpkg".to_string(),
                        params: param_t.clone(),
                        types: types.clone(),
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
        let params: HashMap<String, Expression> = HashMap::from_iter(vec![(
            "v".to_string(),
            Expression::Identifier(Identifier::Local("v".to_string())),
        )]);
        let mut param_t: Vec<(String, Type)> = Vec::new();
        let types: Vec<Type> = Vec::new();
        let state: Vec<(String, Type)> = Vec::new();
        param_t.push(("v".to_string(), Type::Integer));

        let source_id = Identifier::Scalar("v".to_string());
        let target_id = Identifier::Parameter {
            name_in_pkg: "v".to_string(),
            pkgname: "testpkg".to_string(),

            name_in_comp: "v".to_string(),
            compname: "testcomp".to_string(),
        };

        let code = generate_code_blocks(source_id, target_id);
        code.iter().for_each(|c| {
            let res = var_specify(
                &PackageInstance {
                    params: params.clone().into_iter().collect(),
                    types: vec![],
                    name: "testpkg".to_string(),
                    pkg: Package {
                        name: "testpkg".to_string(),
                        params: param_t.clone(),
                        types: types.clone(),
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


*/
