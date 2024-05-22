use crate::{
    expressions::Expression,
    identifier::{
        pkg_ident::{PackageConstIdentifier, PackageIdentifier},
        Identifier,
    },
    package::{OracleDef, OracleSig, Package, PackageInstance},
    parser::package::MultiInstanceIndices,
    statement::Statement,
    types::Type,
};

use self::instantiate::rewrite_split_oracle_def;

mod instantiate {
    use crate::{
        identifier::{
            pkg_ident::{
                PackageImportsLoopVarIdentifier, PackageLocalIdentifier,
                PackageOracleArgIdentifier, PackageOracleCodeLoopVarIdentifier,
                PackageStateIdentifier,
            },
            pkg_inst_ident::PackageInstanceImportsLoopVarIdentifier,
        },
        split::{SplitOracleDef, SplitOracleSig, SplitPath, SplitType},
        statement::CodeBlock,
    };

    use super::*;

    pub(super) fn rewrite_oracle_def(
        pkg_inst_name: &str,
        game_name: &str,
        oracle_def: &OracleDef,
        const_assignments: &[(PackageConstIdentifier, Expression)],
        type_assignments: &[(String, Type)],
    ) -> OracleDef {
        OracleDef {
            sig: rewrite_oracle_sig(&oracle_def.sig, type_assignments),
            code: rewrite_oracle_code(
                pkg_inst_name,
                game_name,
                &oracle_def.code,
                const_assignments,
                type_assignments,
            ),
            file_pos: oracle_def.file_pos.clone(),
        }
    }

    pub(super) fn rewrite_split_oracle_def(
        pkg_inst_name: &str,
        game_name: &str,
        split_oracle_def: &SplitOracleDef,
        const_assignments: &[(PackageConstIdentifier, Expression)],
        type_assignments: &[(String, Type)],
    ) -> SplitOracleDef {
        SplitOracleDef {
            sig: rewrite_split_oracle_sig(
                pkg_inst_name,
                game_name,
                &split_oracle_def.sig,
                const_assignments,
                type_assignments,
            ),
            code: rewrite_oracle_code(
                pkg_inst_name,
                game_name,
                &split_oracle_def.code,
                const_assignments,
                type_assignments,
            ),
        }
    }

    pub(super) fn rewrite_oracle_sig(
        oracle_sig: &OracleSig,
        type_assignments: &[(String, Type)],
    ) -> OracleSig {
        let type_rewrite_rules: Vec<_> = type_assignments
            .iter()
            .map(|(name, tipe)| (Type::UserDefined(name.to_string()), tipe.clone()))
            .collect();

        OracleSig {
            name: oracle_sig.name.clone(),
            multi_inst_idx: oracle_sig.multi_inst_idx.clone(),
            args: oracle_sig
                .args
                .iter()
                .map(|(name, tipe)| (name.clone(), tipe.rewrite(&type_rewrite_rules)))
                .collect(),
            tipe: oracle_sig.tipe.rewrite(&type_rewrite_rules),
        }
    }

    pub(super) fn rewrite_split_oracle_sig(
        pkg_inst_name: &str,
        game_name: &str,
        split_oracle_sig: &SplitOracleSig,
        const_assignments: &[(PackageConstIdentifier, Expression)],
        type_assignments: &[(String, Type)],
    ) -> SplitOracleSig {
        let type_rewrite_rules: Vec<_> = type_assignments
            .iter()
            .map(|(name, tipe)| (Type::UserDefined(name.to_string()), tipe.clone()))
            .collect();

        SplitOracleSig {
            name: split_oracle_sig.name.clone(),
            args: split_oracle_sig
                .args
                .iter()
                .map(|(name, tipe)| (name.clone(), tipe.rewrite(&type_rewrite_rules)))
                .collect(),
            partial_vars: split_oracle_sig
                .partial_vars
                .iter()
                .map(|(name, tipe)| (name.clone(), tipe.rewrite(&type_rewrite_rules)))
                .collect(),
            path: SplitPath::new(
                split_oracle_sig
                    .path
                    .path()
                    .iter()
                    .map(|component| crate::split::SplitPathComponent {
                        pkg_inst_name: component.pkg_inst_name.clone(),
                        oracle_name: component.oracle_name.clone(),
                        split_type: match &component.split_type {
                            SplitType::Plain
                            | SplitType::IfBranch
                            | SplitType::ElseBranch
                            | SplitType::Invoc(_) => component.split_type.clone(),
                            SplitType::ForStep(ident, start, end) => SplitType::ForStep(
                                ident.clone(),
                                rewrite_expression(
                                    pkg_inst_name,
                                    game_name,
                                    start,
                                    const_assignments,
                                    type_assignments,
                                ),
                                rewrite_expression(
                                    pkg_inst_name,
                                    game_name,
                                    end,
                                    const_assignments,
                                    type_assignments,
                                ),
                            ),
                            SplitType::IfCondition(expr) => {
                                SplitType::IfCondition(rewrite_expression(
                                    pkg_inst_name,
                                    game_name,
                                    expr,
                                    const_assignments,
                                    type_assignments,
                                ))
                            }
                        },
                        split_range: component.split_range.clone(),
                    })
                    .collect(),
            ),
            tipe: split_oracle_sig.tipe.rewrite(&type_rewrite_rules),
        }
    }

    pub(super) fn rewrite_oracle_code(
        pkg_inst_name: &str,
        game_name: &str,
        code: &CodeBlock,
        const_assignments: &[(PackageConstIdentifier, Expression)],
        type_assignments: &[(String, Type)],
    ) -> CodeBlock {
        let type_rewrite_rules: Vec<_> = type_assignments
            .iter()
            .map(|(name, tipe)| (Type::UserDefined(name.to_string()), tipe.clone()))
            .collect();

        CodeBlock(
            code.0
                .iter()
                .map(|stmt| {
                    rewrite_statement(
                        pkg_inst_name,
                        game_name,
                        stmt,
                        const_assignments,
                        type_assignments,
                    )
                })
                .collect(),
        )
    }

    pub(super) fn rewrite_statement(
        pkg_inst_name: &str,
        game_name: &str,
        stmt: &Statement,
        const_assignments: &[(PackageConstIdentifier, Expression)],
        type_assignments: &[(String, Type)],
    ) -> Statement {
        let type_rewrite_rules: Vec<_> = type_assignments
            .iter()
            .map(|(name, tipe)| (Type::UserDefined(name.to_string()), tipe.clone()))
            .collect();

        match stmt {
            Statement::Abort(_) => stmt.clone(),
            Statement::Return(expr, pos) => Statement::Return(
                expr.clone().map(|expr| {
                    rewrite_expression(
                        pkg_inst_name,
                        game_name,
                        &expr,
                        const_assignments,
                        type_assignments,
                    )
                }),
                pos.clone(),
            ),

            Statement::Assign(ident, index, value, pos) => Statement::Assign(
                rewrite_identifier(pkg_inst_name, game_name, ident, type_assignments),
                index.clone().map(|expr| {
                    rewrite_expression(
                        pkg_inst_name,
                        game_name,
                        &expr,
                        const_assignments,
                        type_assignments,
                    )
                }),
                rewrite_expression(
                    pkg_inst_name,
                    game_name,
                    value,
                    const_assignments,
                    type_assignments,
                ),
                pos.clone(),
            ),
            Statement::Parse(idents, expr, pos) => Statement::Parse(
                idents
                    .iter()
                    .map(|ident| {
                        rewrite_identifier(pkg_inst_name, game_name, ident, type_assignments)
                    })
                    .collect(),
                rewrite_expression(
                    pkg_inst_name,
                    game_name,
                    expr,
                    const_assignments,
                    type_assignments,
                ),
                pos.clone(),
            ),
            Statement::Sample(ident, index, sample_id, tipe, pos) => Statement::Sample(
                rewrite_identifier(pkg_inst_name, game_name, ident, type_assignments),
                index.clone().map(|expr| {
                    rewrite_expression(
                        pkg_inst_name,
                        game_name,
                        &expr,
                        const_assignments,
                        type_assignments,
                    )
                }),
                *sample_id,
                tipe.rewrite(&type_rewrite_rules),
                pos.clone(),
            ),
            Statement::InvokeOracle {
                id,
                opt_idx,
                opt_dst_inst_idx,
                name,
                args,
                target_inst_name,
                tipe,
                file_pos,
            } => Statement::InvokeOracle {
                id: rewrite_identifier(pkg_inst_name, game_name, id, type_assignments),
                opt_idx: opt_idx.clone().map(|expr| {
                    rewrite_expression(
                        pkg_inst_name,
                        game_name,
                        &expr,
                        const_assignments,
                        type_assignments,
                    )
                }),
                opt_dst_inst_idx: opt_dst_inst_idx.clone().map(|expr| {
                    rewrite_expression(
                        pkg_inst_name,
                        game_name,
                        &expr,
                        const_assignments,
                        type_assignments,
                    )
                }),
                name: name.clone(),
                args: args
                    .iter()
                    .map(|expr| {
                        rewrite_expression(
                            pkg_inst_name,
                            game_name,
                            expr,
                            const_assignments,
                            type_assignments,
                        )
                    })
                    .collect(),
                target_inst_name: target_inst_name.clone(),
                tipe: tipe.clone().map(|tipe| tipe.rewrite(&type_rewrite_rules)),
                file_pos: file_pos.clone(),
            },

            Statement::IfThenElse(cond, ifblock, elseblock, pos) => Statement::IfThenElse(
                rewrite_expression(
                    pkg_inst_name,
                    game_name,
                    cond,
                    const_assignments,
                    type_assignments,
                ),
                rewrite_oracle_code(
                    pkg_inst_name,
                    game_name,
                    ifblock,
                    const_assignments,
                    type_assignments,
                ),
                rewrite_oracle_code(
                    pkg_inst_name,
                    game_name,
                    elseblock,
                    const_assignments,
                    type_assignments,
                ),
                pos.clone(),
            ),
            Statement::For(ident, start, end, code, pos) => Statement::For(
                ident.clone(),
                rewrite_expression(
                    pkg_inst_name,
                    game_name,
                    start,
                    const_assignments,
                    type_assignments,
                ),
                rewrite_expression(
                    pkg_inst_name,
                    game_name,
                    end,
                    const_assignments,
                    type_assignments,
                ),
                rewrite_oracle_code(
                    pkg_inst_name,
                    game_name,
                    code,
                    const_assignments,
                    type_assignments,
                ),
                pos.clone(),
            ),
        }
    }

    pub(super) fn rewrite_expression(
        pkg_inst_name: &str,
        game_name: &str,
        expr: &Expression,
        const_assignments: &[(PackageConstIdentifier, Expression)],
        type_assignments: &[(String, Type)],
    ) -> Expression {
        expr.map(|expr| match expr {
            Expression::Identifier(Identifier::PackageIdentifier(PackageIdentifier::Const(
                pkg_const_ident,
            ))) => const_assignments
                .iter()
                .find_map(|(search, replace)| {
                    if search == &pkg_const_ident {
                        Some(Expression::Identifier(Identifier::PackageIdentifier(
                            PackageIdentifier::Const(PackageConstIdentifier {
                                game_assignment: Some(Box::new(replace.clone())),
                                ..pkg_const_ident.clone()
                            }),
                        )))
                    } else {
                        None
                    }
                })
                .unwrap(),
            Expression::Identifier(other_ident) => Expression::Identifier(rewrite_identifier(
                pkg_inst_name,
                game_name,
                &other_ident,
                type_assignments,
            )),
            other => other.clone(),
        })
    }

    pub(super) fn rewrite_identifier(
        pkg_inst_name: &str,
        game_name: &str,
        ident: &Identifier,
        type_assignments: &[(String, Type)],
    ) -> Identifier {
        let type_rewrite_rules: Vec<_> = type_assignments
            .iter()
            .map(|(name, tipe)| (Type::UserDefined(name.to_string()), tipe.clone()))
            .collect();

        match ident {
            Identifier::PackageIdentifier(pkg_ident) => {
                let pkg_ident = match pkg_ident {
                    PackageIdentifier::Const(_) => {
                        // This code can only be reached when rewriting a left-hand-side of an
                        // assign-like statement (which is forbidden for consts). it can't occur
                        // when rewriting expressions, as that already resolves these
                        unreachable!("no const identifiers allowed here")
                    }
                    PackageIdentifier::State(state_ident) => {
                        PackageIdentifier::State(PackageStateIdentifier {
                            pkg_inst_name: Some(pkg_inst_name.to_string()),
                            game_name: Some(game_name.to_string()),
                            tipe: state_ident.tipe.rewrite(&type_rewrite_rules),
                            ..state_ident.clone()
                        })
                    }
                    PackageIdentifier::Local(local_ident) => {
                        PackageIdentifier::Local(PackageLocalIdentifier {
                            pkg_inst_name: Some(pkg_inst_name.to_string()),
                            game_name: Some(game_name.to_string()),
                            tipe: local_ident.tipe.rewrite(&type_rewrite_rules),
                            ..local_ident.clone()
                        })
                    }
                    // XXX: A bit unclear whether we keep this variant... it conflicts witht he
                    // Oracle variant of Declaration, so we only need one and so far we use the
                    // other. Also this one doesn't seem to support multi instance
                    PackageIdentifier::OracleImport(_) => todo!(),

                    PackageIdentifier::OracleArg(arg_ident) => {
                        PackageIdentifier::OracleArg(PackageOracleArgIdentifier {
                            pkg_inst_name: Some(pkg_inst_name.to_string()),
                            game_name: Some(game_name.to_string()),
                            tipe: arg_ident.tipe.rewrite(&type_rewrite_rules),
                            ..arg_ident.clone()
                        })
                    }
                    PackageIdentifier::ImportsLoopVar(loopvar_ident) => {
                        PackageIdentifier::ImportsLoopVar(PackageImportsLoopVarIdentifier {
                            pkg_inst_name: Some(pkg_inst_name.to_string()),
                            game_name: Some(game_name.to_string()),
                            ..loopvar_ident.clone()
                        })
                    }
                    PackageIdentifier::CodeLoopVar(loopvar_ident) => {
                        PackageIdentifier::CodeLoopVar(PackageOracleCodeLoopVarIdentifier {
                            pkg_inst_name: Some(pkg_inst_name.to_string()),
                            game_name: Some(game_name.to_string()),
                            ..loopvar_ident.clone()
                        })
                    }
                };

                Identifier::PackageIdentifier(pkg_ident)
            }

            Identifier::GameIdentifier(_) => ident.clone(),

            Identifier::ProofIdentifier(_) => unreachable!(
                "unexpected proof identifier when instantiating package: {:?}",
                ident
            ),

            other => unreachable!("won't rewrite deprecated identifier {:?}", other),
        }
    }
}

impl PackageInstance {
    pub(crate) fn new(
        pkg_inst_name: &str,
        game_name: &str,
        pkg: &Package,
        multi_instance_indices: MultiInstanceIndices,
        params: Vec<(PackageConstIdentifier, Expression)>,
        types: Vec<(String, Type)>,
    ) -> PackageInstance {
        let new_oracles = pkg
            .oracles
            .iter()
            .map(|oracle_def| {
                instantiate::rewrite_oracle_def(
                    pkg_inst_name,
                    game_name,
                    oracle_def,
                    &params,
                    &types,
                )
            })
            .collect();

        let new_split_oracles = pkg
            .split_oracles
            .iter()
            .map(|split_oracle_def| {
                rewrite_split_oracle_def(
                    pkg_inst_name,
                    game_name,
                    split_oracle_def,
                    &params,
                    &types,
                )
            })
            .collect();

        let pkg = Package {
            types: vec![],
            params: vec![],
            name: pkg.name.clone(),
            state: pkg.state.clone(),
            imports: pkg.imports.clone(),
            oracles: new_oracles,
            split_oracles: new_split_oracles,
        };

        PackageInstance {
            params,
            types,
            pkg,
            name: pkg_inst_name.to_string(),
            multi_instance_indices,
        }
    }
}
