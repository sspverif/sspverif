use crate::{
    expressions::Expression,
    identifier::{
        pkg_ident::{PackageConstIdentifier, PackageIdentifier},
        Identifier,
    },
    package::{OracleDef, OracleSig, Package},
    parser::package::MultiInstanceIndices,
    statement::Statement,
    types::Type,
};

use self::instantiate::InstantiationContext;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageInstance {
    pub name: String,

    // we need these two to compare whether two instances of a same package have the same
    // parameters (both constants and types).
    // We need to compare that when checking the mapping of game and an assumption:
    // A mapping is only valid if the package instances in the game and the assumption
    // are excatly the same, i.e. they are different instances of the same package with
    // the same parameters.
    // TODO: this should probably just be (String, Expression)
    pub params: Vec<(PackageConstIdentifier, Expression)>,
    pub types: Vec<(String, Type)>,

    // this is the package - it has been rewritten, though!
    pub pkg: Package,

    // These are probably deprecated?
    pub multi_instance_indices: MultiInstanceIndices,
}

impl PackageInstance {
    pub fn get_oracle_sigs(&self) -> Vec<OracleSig> {
        self.pkg
            .oracles
            .clone()
            .into_iter()
            .map(|d| d.sig)
            .collect()
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
        let inst_ctx: InstantiationContext =
            InstantiationContext::new_package_instantiation_context(
                pkg_inst_name,
                game_name,
                &params,
                &types,
            );

        let params_as_ident: Vec<(Identifier, Expression)> = params
            .iter()
            .map(|(ident, expr)| (ident.clone().into(), expr.clone()))
            .collect();

        let new_oracles = pkg
            .oracles
            .iter()
            .map(|oracle_def| inst_ctx.rewrite_oracle_def(oracle_def))
            .collect();

        let new_split_oracles = pkg
            .split_oracles
            .iter()
            .map(|split_oracle_def| inst_ctx.rewrite_split_oracle_def(split_oracle_def))
            .collect();

        let pkg = Package {
            types: vec![],
            params: vec![],
            oracles: new_oracles,
            split_oracles: new_split_oracles,
            ..pkg.clone()
        };

        PackageInstance {
            params,
            types,
            pkg,
            name: pkg_inst_name.to_string(),
            multi_instance_indices,
        }
    }

    /// This returns whether the package and the parameters inside the package are the same.
    /// TODO: also check that the type parameters are the same
    pub(crate) fn practically_equivalent(&self, other: &PackageInstance) -> bool {
        self.pkg.name == other.pkg.name && self.params == other.params
    }
}

pub(crate) mod instantiate {
    use std::marker::PhantomData;

    use crate::{
        identifier::{
            game_ident::{GameConstIdentifier, GameIdentInstanciationInfo, GameIdentifier},
            pkg_ident::{
                PackageImportsLoopVarIdentifier, PackageLocalIdentifier,
                PackageOracleArgIdentifier, PackageOracleCodeLoopVarIdentifier,
                PackageStateIdentifier,
            },
            proof_ident::ProofIdentInstanciationInfo,
        },
        package::Composition,
        proof::GameInstance,
        split::{SplitOracleDef, SplitOracleSig, SplitPath, SplitType},
        statement::CodeBlock,
    };

    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub(crate) enum InstantiationSource<'a> {
        Package {
            const_assignments: &'a [(PackageConstIdentifier, Expression)],
        },

        Game {
            const_assignments: &'a [(GameConstIdentifier, Expression)],
        },
    }

    #[derive(Debug, Clone, Copy)]
    pub(crate) struct InstantiationContext<'a> {
        src: InstantiationSource<'a>,

        inst_name: &'a str,
        parent_name: &'a str,

        type_assignments: &'a [(String, Type)],
    }

    impl<'a> InstantiationContext<'a> {
        pub(crate) fn new_package_instantiation_context(
            inst_name: &'a str,
            parent_name: &'a str,
            consts: &'a [(PackageConstIdentifier, Expression)],
            types: &'a [(String, Type)],
        ) -> Self {
            Self {
                src: InstantiationSource::Package {
                    const_assignments: consts,
                },
                inst_name,
                parent_name,
                type_assignments: types,
            }
        }

        pub(crate) fn new_game_instantiation_context(
            inst_name: &'a str,
            parent_name: &'a str,
            consts: &'a [(GameConstIdentifier, Expression)],
            types: &'a [(String, Type)],
        ) -> Self {
            Self {
                src: InstantiationSource::Game {
                    const_assignments: consts,
                },
                inst_name,
                parent_name,
                type_assignments: types,
            }
        }

        pub(crate) fn rewrite_oracle_def(&self, oracle_def: &OracleDef) -> OracleDef {
            OracleDef {
                sig: self.rewrite_oracle_sig(&oracle_def.sig),
                code: self.rewrite_oracle_code(&oracle_def.code),
                file_pos: oracle_def.file_pos,
            }
        }

        pub(crate) fn rewrite_oracle_sig(&self, oracle_sig: &OracleSig) -> OracleSig {
            {
                let type_rewrite_rules: Vec<_> = self
                    .type_assignments
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
        }

        pub(crate) fn rewrite_oracle_code(&self, code: &CodeBlock) -> CodeBlock {
            CodeBlock(
                code.0
                    .iter()
                    .map(|stmt| self.rewrite_statement(stmt))
                    .collect(),
            )
        }

        pub(crate) fn rewrite_split_oracle_sig(
            &self,
            split_oracle_sig: &SplitOracleSig,
        ) -> SplitOracleSig {
            let type_rewrite_rules: Vec<_> = self
                .type_assignments
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
                                    self.rewrite_expression(start),
                                    self.rewrite_expression(end),
                                ),
                                SplitType::IfCondition(expr) => {
                                    SplitType::IfCondition(self.rewrite_expression(expr))
                                }
                            },
                            split_range: component.split_range.clone(),
                        })
                        .collect(),
                ),
                tipe: split_oracle_sig.tipe.rewrite(&type_rewrite_rules),
            }
        }

        pub(crate) fn rewrite_split_oracle_def(
            &self,
            split_oracle_def: &SplitOracleDef,
        ) -> SplitOracleDef {
            SplitOracleDef {
                sig: self.rewrite_split_oracle_sig(&split_oracle_def.sig),
                code: self.rewrite_oracle_code(&split_oracle_def.code),
            }
        }

        pub(crate) fn rewrite_statement(&self, stmt: &Statement) -> Statement {
            let type_rewrite_rules: Vec<_> = self
                .type_assignments
                .iter()
                .map(|(name, tipe)| (Type::UserDefined(name.to_string()), tipe.clone()))
                .collect();

            match stmt {
                Statement::Abort(_) => stmt.clone(),
                Statement::Return(expr, pos) => Statement::Return(
                    expr.clone().map(|expr| self.rewrite_expression(&expr)),
                    *pos,
                ),

                Statement::Assign(ident, index, value, pos) => Statement::Assign(
                    rewrite_identifier(
                        self.inst_name,
                        self.parent_name,
                        ident,
                        self.type_assignments,
                    ),
                    index.clone().map(|expr| self.rewrite_expression(&expr)),
                    self.rewrite_expression(value),
                    *pos,
                ),
                Statement::Parse(idents, expr, pos) => Statement::Parse(
                    idents
                        .iter()
                        .map(|ident| {
                            rewrite_identifier(
                                self.inst_name,
                                self.parent_name,
                                ident,
                                self.type_assignments,
                            )
                        })
                        .collect(),
                    self.rewrite_expression(expr),
                    *pos,
                ),
                Statement::Sample(ident, index, sample_id, tipe, pos) => Statement::Sample(
                    rewrite_identifier(
                        self.inst_name,
                        self.parent_name,
                        ident,
                        self.type_assignments,
                    ),
                    index.clone().map(|expr| self.rewrite_expression(&expr)),
                    *sample_id,
                    tipe.rewrite(&type_rewrite_rules),
                    *pos,
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
                    id: rewrite_identifier(
                        self.inst_name,
                        self.parent_name,
                        id,
                        self.type_assignments,
                    ),
                    opt_idx: opt_idx.clone().map(|expr| self.rewrite_expression(&expr)),
                    opt_dst_inst_idx: opt_dst_inst_idx
                        .clone()
                        .map(|expr| self.rewrite_expression(&expr)),
                    name: name.clone(),
                    args: args
                        .iter()
                        .map(|expr| self.rewrite_expression(expr))
                        .collect(),
                    target_inst_name: target_inst_name.clone(),
                    tipe: tipe.clone().map(|tipe| tipe.rewrite(&type_rewrite_rules)),
                    file_pos: *file_pos,
                },

                Statement::IfThenElse(cond, ifblock, elseblock, pos) => Statement::IfThenElse(
                    self.rewrite_expression(cond),
                    self.rewrite_oracle_code(ifblock),
                    self.rewrite_oracle_code(elseblock),
                    *pos,
                ),
                Statement::For(ident, start, end, code, pos) => Statement::For(
                    ident.clone(),
                    self.rewrite_expression(start),
                    self.rewrite_expression(end),
                    self.rewrite_oracle_code(code),
                    *pos,
                ),
            }
        }
    }

    impl<'a> InstantiationContext<'a> {
        pub(crate) fn rewrite_expression(&self, expr: &Expression) -> Expression {
            expr.map(|expr| match (self.src, expr) {
                (
                    InstantiationSource::Package { const_assignments },
                    Expression::Identifier(Identifier::PackageIdentifier(
                        PackageIdentifier::Const(pkg_const_ident),
                    )),
                ) => const_assignments
                    .iter()
                    .find_map(|(search, replace)| {
                        if search.name == pkg_const_ident.name {
                            let new_expr = replace.map(|expr| match expr {
                                Expression::Identifier(Identifier::GameIdentifier(game_ident)) => {
                                    let inst_info = GameIdentInstanciationInfo {
                                        lower: pkg_const_ident.clone(),
                                        pkg_inst_name: self.inst_name.to_string(),
                                    };

                                    Expression::Identifier(Identifier::GameIdentifier(
                                        game_ident.with_instance_info(inst_info),
                                    ))
                                }
                                other => other,
                            });
                            Some(new_expr)
                        } else {
                            None
                        }
                    })
                    .unwrap(),
                (
                    InstantiationSource::Game { const_assignments },
                    Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::Const(
                        game_const_ident,
                    ))),
                ) => const_assignments
                    .iter()
                    .find_map(|(search, replace)| {
                        if search.name == game_const_ident.name {
                            let new_expr = replace.map(|expr| match expr {
                                Expression::Identifier(Identifier::ProofIdentifier(
                                    proof_ident,
                                )) => {
                                    let inst_info = ProofIdentInstanciationInfo {
                                        lower: game_const_ident.clone(),
                                        game_inst_name: self.inst_name.to_string(),
                                    };

                                    Expression::Identifier(Identifier::ProofIdentifier(
                                        proof_ident.with_instance_info(inst_info),
                                    ))
                                }
                                other => other,
                            });
                            Some(new_expr)
                        } else {
                            None
                        }
                    })
                    .unwrap(),

                (_, expr) => expr,
            })
        }
    }

    pub(crate) fn rewrite_expression(
        inst_name: &str,
        game_name: &str,
        expr: &Expression,
        const_assignments: &[(Identifier, Expression)],
        type_assignments: &[(String, Type)],
    ) -> Expression {
        expr.map(|expr| match expr {
            Expression::Identifier(Identifier::PackageIdentifier(PackageIdentifier::Const(
                pkg_const_ident,
            ))) => const_assignments
                .iter()
                .find_map(|(search, replace)| match (search, replace) {
                    (Identifier::PackageIdentifier(PackageIdentifier::Const(search)), new_expr) => {
                        if search.name == pkg_const_ident.name {
                            let new_expr = new_expr.map(|expr| match expr {
                                Expression::Identifier(Identifier::GameIdentifier(game_ident)) => {
                                    let inst_info = GameIdentInstanciationInfo {
                                        lower: pkg_const_ident.clone(),
                                        pkg_inst_name: inst_name.to_string(),
                                    };

                                    Expression::Identifier(Identifier::GameIdentifier(
                                        game_ident.with_instance_info(inst_info),
                                    ))
                                }
                                other => other,
                            });
                            Some(new_expr)
                        } else {
                            None
                        }
                    }

                    other => {
                        unreachable!(
                            "expected a game identifier when rewriting, got {other:?}",
                            other = other
                        )
                    }
                })
                .unwrap(),

            Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::Const(
                game_const_ident,
            ))) => const_assignments
                .iter()
                .find_map(|(search, replace)| match (search, replace) {
                    (Identifier::GameIdentifier(GameIdentifier::Const(search)), new_expr) => {
                        if search.name == game_const_ident.name {
                            let new_expr = new_expr.map(|expr| match expr {
                                Expression::Identifier(Identifier::ProofIdentifier(
                                    proof_ident,
                                )) => {
                                    let inst_info = ProofIdentInstanciationInfo {
                                        lower: game_const_ident.clone(),
                                        game_inst_name: inst_name.to_string(),
                                    };

                                    Expression::Identifier(Identifier::ProofIdentifier(
                                        proof_ident.with_instance_info(inst_info),
                                    ))
                                }
                                other => other,
                            });
                            Some(new_expr)
                        } else {
                            None
                        }
                    }

                    other => {
                        unreachable!(
                            "expected a proof identifier when rewriting, got {other:?}",
                            other = other
                        )
                    }
                })
                .unwrap_or_else(||  panic!("tried finding a value for {game_const_ident:?}, but that was not in the list of declared values {const_assignments:?}", game_const_ident = game_const_ident, const_assignments = const_assignments)),

            Expression::Identifier(other_ident) => Expression::Identifier(rewrite_identifier(
                inst_name,
                game_name,
                &other_ident,
                type_assignments,
            )),
            Expression::TableAccess(ident, expr) => {
                Expression::TableAccess(rewrite_identifier(inst_name, game_name, &ident, type_assignments), expr)
            },
            other => other.clone(),
        })
    }

    pub(crate) fn rewrite_identifier(
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
