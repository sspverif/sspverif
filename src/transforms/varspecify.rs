use std::convert::Infallible;

use crate::expressions::Expression;
use crate::identifier::{
    ComposeLoopVar, GameInstanceConst, Identifier, PackageConst, PackageState,
};
use crate::package::{Composition, OracleDef, OracleSig, Package, PackageInstance};
use crate::parser::package::{ForSpec, MultiInstanceIndices};
use crate::proof::GameInstance;
use crate::split::SplitOracleDef;
use crate::statement::{CodeBlock, Statement};
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

// TODO: These should return an error
fn resolve_game_var(game_inst: &GameInstance, name: &str) -> Identifier {
    let game_inst_params = SliceResolver(game_inst.consts());
    let game_inst_name = game_inst.name().to_string();
    let result = game_inst_params.resolve_value(name);

    if let Some((_, Expression::Identifier(id_in_proof))) = result {
        Identifier::GameInstanceConst(GameInstanceConst {
            name_in_comp: name.to_string(),
            name_in_proof: id_in_proof.ident(),
            game_inst_name,
        })
    } else {
        unreachable!("{:?} {name}", result, name = name)
    }
}

// TODO: add support for resolving to expression literals
fn resolve_pkg_var(game_inst: &GameInstance, pkg_inst: &PackageInstance, name: &str) -> Identifier {
    let pkg_state = SliceResolver(&pkg_inst.pkg.state);
    let pkg_inst_params = SliceResolver(&pkg_inst.params);
    let game_inst_params = SliceResolver(game_inst.consts());

    let pkg_name = pkg_inst.pkg.name.clone();
    let game_inst_name = game_inst.name().to_string();

    if let Some(_) = pkg_state.resolve_value(&name) {
        Identifier::State(PackageState {
            name: name.to_string(),
            pkg_inst_name: pkg_inst.name.clone(),
            game_inst_name: game_inst.name().to_string(),
        })
    } else if let Some((_, Expression::Identifier(id))) = &pkg_inst_params.resolve_value(&name) {
        println!("game_inst_params: {game_inst_params:?}");

        let pkg_inst_forspecs = SliceResolver(
            &pkg_inst
                .multi_instance_indices
                .as_ref()
                .map(|idcs| &idcs.forspecs[..])
                .unwrap_or(&[]),
        );

        if let Some(forspec) = pkg_inst_forspecs.resolve_value(id.ident_ref()) {
            Identifier::ComposeLoopVar(ComposeLoopVar {
                name_in_pkg: name.to_string(),
                name_in_comp: forspec.ident().ident(),
                pkgname: pkg_name,
                game_inst_name,
            })
        } else if let Some((_, Expression::Identifier(id_in_proof))) =
            game_inst_params.resolve_value(&id.ident())
        {
            Identifier::Parameter(PackageConst {
                name_in_pkg: name.to_string(),
                pkgname: pkg_name,
                name_in_comp: id.ident(),
                name_in_proof: id_in_proof.ident(),
                game_inst_name,
            })
        } else {
            unreachable!("id: {:?}", id)
        }
    } else {
        Identifier::Local(name.into())
    }
}

fn var_specify_expression(
    game_inst: &GameInstance,
    pkg_inst: &PackageInstance,
    expr: &Expression,
) -> Expression {
    match expr {
        Expression::FnCall(Identifier::Scalar(id), args) => {
            Expression::FnCall(resolve_pkg_var(game_inst, pkg_inst, id), args.clone())
        }
        Expression::Identifier(Identifier::Scalar(id)) => {
            Expression::Identifier(resolve_pkg_var(game_inst, pkg_inst, id))
        }
        Expression::TableAccess(Identifier::Scalar(id), expr) => {
            Expression::TableAccess(resolve_pkg_var(game_inst, pkg_inst, id), expr.clone())
        }
        _ => expr.clone(),
    }
}

fn var_specify_helper(
    game_inst: &GameInstance,
    pkg_inst: &PackageInstance,
    block: CodeBlock,
) -> CodeBlock {
    let fixup = |expr| var_specify_expression(game_inst, pkg_inst, &expr);
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
                    opt_dst_inst_idx,
                    name,
                    args,
                    target_inst_name,
                    tipe,
                    file_pos,
                } => {
                    let opt_idx = opt_idx.clone().map(|expr| expr.map(fixup));
                    let opt_dst_inst_idx = opt_dst_inst_idx.clone().map(|expr| expr.map(fixup));
                    let args = args.iter().map(|expr| expr.map(fixup)).collect();
                    if let Expression::Identifier(id) = fixup(id.to_expression()) {
                        Statement::InvokeOracle {
                            id,
                            opt_idx,
                            opt_dst_inst_idx,
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

fn var_specify_oracle_sig(
    game_inst: &GameInstance,
    pkg_inst: &PackageInstance,
    oracle_sig: &OracleSig,
) -> OracleSig {
    let multi_inst_idx = oracle_sig
        .multi_inst_idx
        .clone()
        .map(|multi_instance_indices| {
            let mut new_forspecs = vec![];
            for forspec in &multi_instance_indices.forspecs {
                new_forspecs.push(ForSpec::new(
                    forspec.ident().clone(),
                    var_specify_expression(game_inst, pkg_inst, forspec.start()),
                    var_specify_expression(game_inst, pkg_inst, forspec.end()),
                    forspec.start_comp().clone(),
                    forspec.end_comp().clone(),
                ));
            }
            MultiInstanceIndices {
                forspecs: new_forspecs,
                ..multi_instance_indices.clone()
            }
        });

    OracleSig {
        multi_inst_idx,
        ..oracle_sig.clone()
    }
}

fn var_specify_pkg_inst(game_inst: &GameInstance, pkg_inst: &PackageInstance) -> PackageInstance {
    PackageInstance {
        pkg: Package {
            imports: pkg_inst
                .pkg
                .imports
                .iter()
                .map(|(oracle_sig, file_pos)| {
                    println!("import: {oracle_sig:#?}");
                    (
                        var_specify_oracle_sig(game_inst, pkg_inst, oracle_sig),
                        file_pos.clone(),
                    )
                })
                .collect(),
            oracles: pkg_inst
                .pkg
                .oracles
                .iter()
                .map(|def| OracleDef {
                    sig: var_specify_oracle_sig(game_inst, pkg_inst, &def.sig),
                    code: var_specify_helper(game_inst, pkg_inst, def.code.clone()),
                    file_pos: def.file_pos.clone(),
                })
                .collect(),

            split_oracles: pkg_inst
                .pkg
                .split_oracles
                .iter()
                .map(|def| SplitOracleDef {
                    // TODO add resolution for split oracle signature
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
    let mut game: Composition = game_inst
        .game()
        .map_pkg_inst::<Infallible, _>(|pkg_inst| Ok(var_specify_pkg_inst(game_inst, pkg_inst)))?;

    for i in 0..game.multi_inst_edges.len() {
        println!("game: {}", game.name());
        println!("game.multi_inst_edges: {:?}", game.multi_inst_edges);
        if let Some(multi_inst_idx) = &game.multi_inst_edges[i].oracle_sig.multi_inst_idx {
            game.multi_inst_edges[i].oracle_sig.multi_inst_idx = Some(
                var_specify_multi_instance_indices(game_inst, multi_inst_idx.clone())?,
            );
        }
    }

    Ok(game)
}

pub fn var_specify_multi_instance_indices(
    game_inst: &GameInstance,
    mut multi_instance_indices: MultiInstanceIndices,
) -> Result<MultiInstanceIndices, Infallible> {
    for i in 0..multi_instance_indices.forspecs.len() {
        println!("varspecify {i} {:?}", multi_instance_indices);
        multi_instance_indices.forspecs[i] = multi_instance_indices.forspecs[i]
            .map_identifiers(|id| -> Identifier { resolve_game_var(game_inst, id.ident_ref()) });
        println!("varspecify {i} {:?}", multi_instance_indices);
    }

    Ok(multi_instance_indices)
}
