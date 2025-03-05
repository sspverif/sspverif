use itertools::Itertools as _;

use crate::{
    expressions::Expression,
    identifier::{
        pkg_ident::{PackageConstIdentifier, PackageIdentifier},
        Identifier,
    },
    package::{OracleDef, OracleSig, Package},
    parser::package::MultiInstanceIndices,
    statement::Statement,
    types::{CountSpec, Type},
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
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pkg_name(&self) -> &str {
        &self.pkg.name
    }

    pub fn get_oracle_sigs(&self) -> Vec<OracleSig> {
        self.pkg
            .oracles
            .clone()
            .into_iter()
            .map(|d| d.sig)
            .collect()
    }

    pub fn params(&self) -> Vec<(&str, &Expression)> {
        let mut params: Vec<_> = self
            .params
            .iter()
            .map(|(name, expr)| (name.name.as_str(), expr))
            .collect();
        params.sort();
        params
    }

    /// instantiates the provided oraclae signature. this means that occurrences package parameters
    /// are replaced with the assigned values.
    ///
    /// this is needed for Bits(n), since the `n` is different in game and package.
    pub(crate) fn instantiate_oracle_signature(&self, sig: OracleSig) -> OracleSig {
        OracleSig {
            args: sig
                .args
                .into_iter()
                .map(|(name, ty)| (name, self.instantiate_type(ty)))
                .collect(),
            tipe: self.instantiate_type(sig.tipe),
            ..sig
        }
    }

    /// instantiates the provided type. this means that occurrences package parameters
    /// are replaced with the assigned values.
    ///
    /// This also means that the types somehow don't match 100%, this will just ignore that type and
    /// leave it as-is. But that shouldn't really happen, since we compare the types in the package
    /// params with the types in the code. But it could be the source of annoying-to-debug bugs.
    ///
    /// this is needed for Bits(n), since the `n` is different in game and package.
    pub(crate) fn instantiate_type(&self, ty: Type) -> Type {
        // we only want the ints, because the maybe be in Bits(n)
        let int_params = self
            .params
            .iter()
            .filter(|(_, expr)| matches!(expr.get_type(), Type::Integer))
            .map(|(ident, expr)| {
                let assigned_value = match expr {
                    Expression::Identifier(ident) => CountSpec::Identifier(ident.clone()),
                    Expression::IntegerLiteral(num) => CountSpec::Literal(*num as u64),
                    _ => unreachable!(),
                };

                (
                    Type::Bits(Box::new(crate::types::CountSpec::Identifier(
                        ident.clone().into(),
                    ))),
                    Type::Bits(Box::new(assigned_value)),
                )
            })
            .collect_vec();

        ty.rewrite_type(&int_params)
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

        let new_oracles = pkg
            .oracles
            .iter()
            .map(|oracle_def| inst_ctx.rewrite_oracle_def(oracle_def.clone()))
            .collect();

        // let new_split_oracles = pkg
        //     .split_oracles
        //     .iter()
        //     .map(|split_oracle_def| inst_ctx.rewrite_split_oracle_def(split_oracle_def.clone()))
        //     .collect();

        let pkg = Package {
            oracles: new_oracles,
            //split_oracles: new_split_oracles,
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
    use crate::{
        identifier::{
            game_ident::{GameConstIdentifier, GameIdentInstanciationInfo, GameIdentifier},
            pkg_ident::{
                PackageLocalIdentifier, PackageOracleArgIdentifier, PackageStateIdentifier,
            },
            proof_ident::ProofIdentInstanciationInfo,
        },
        statement::{CodeBlock, IfThenElse, InvokeOracleStatement},
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

        pub(crate) fn rewrite_oracle_def(&self, oracle_def: OracleDef) -> OracleDef {
            OracleDef {
                sig: self.rewrite_oracle_sig(oracle_def.sig),
                code: self.rewrite_oracle_code(oracle_def.code),
                file_pos: oracle_def.file_pos,
            }
        }

        pub(crate) fn rewrite_oracle_sig(&self, oracle_sig: OracleSig) -> OracleSig {
            {
                let type_rewrite_rules: Vec<_> = self
                    .type_assignments
                    .iter()
                    .map(|(name, tipe)| (Type::UserDefined(name.to_string()), tipe.clone()))
                    .collect();

                OracleSig {
                    name: oracle_sig.name,
                    multi_inst_idx: oracle_sig.multi_inst_idx,
                    args: oracle_sig
                        .args
                        .into_iter()
                        .map(|(name, tipe)| (name.clone(), tipe.rewrite_type(&type_rewrite_rules)))
                        .collect(),
                    tipe: oracle_sig.tipe.rewrite_type(&type_rewrite_rules),
                }
            }
        }

        pub(crate) fn rewrite_oracle_code(&self, code: CodeBlock) -> CodeBlock {
            CodeBlock(
                code.0
                    .into_iter()
                    .map(|stmt| self.rewrite_statement(stmt))
                    .collect(),
            )
        }

        // pub(crate) fn rewrite_split_oracle_sig(
        //     &self,
        //     split_oracle_sig: &SplitOracleSig,
        // ) -> SplitOracleSig {
        //     let type_rewrite_rules: Vec<_> = self
        //         .type_assignments
        //         .iter()
        //         .map(|(name, tipe)| (Type::UserDefined(name.to_string()), tipe.clone()))
        //         .collect();
        //
        //     SplitOracleSig {
        //         name: split_oracle_sig.name.clone(),
        //         args: split_oracle_sig
        //             .args
        //             .iter()
        //             .map(|(name, tipe)| (name.clone(), tipe.rewrite(&type_rewrite_rules)))
        //             .collect(),
        //         partial_vars: split_oracle_sig
        //             .partial_vars
        //             .iter()
        //             .map(|(name, tipe)| (name.clone(), tipe.rewrite(&type_rewrite_rules)))
        //             .collect(),
        //         path: SplitPath::new(
        //             split_oracle_sig
        //                 .path
        //                 .path()
        //                 .iter()
        //                 .map(|component| crate::split::SplitPathComponent {
        //                     pkg_inst_name: component.pkg_inst_name.clone(),
        //                     oracle_name: component.oracle_name.clone(),
        //                     split_type: match &component.split_type {
        //                         SplitType::Plain
        //                         | SplitType::IfBranch
        //                         | SplitType::ElseBranch
        //                         | SplitType::Invoc(_) => component.split_type.clone(),
        //                         SplitType::ForStep(ident, start, end) => SplitType::ForStep(
        //                             ident.clone(),
        //                             self.rewrite_expression(start),
        //                             self.rewrite_expression(end),
        //                         ),
        //                         SplitType::IfCondition(expr) => {
        //                             SplitType::IfCondition(self.rewrite_expression(expr))
        //                         }
        //                     },
        //                     split_range: component.split_range.clone(),
        //                 })
        //                 .collect(),
        //         ),
        //         tipe: split_oracle_sig.tipe.rewrite(&type_rewrite_rules),
        //     }
        // }
        //
        // pub(crate) fn rewrite_split_oracle_def(
        //     &self,
        //     split_oracle_def: SplitOracleDef,
        // ) -> SplitOracleDef {
        //     SplitOracleDef {
        //         sig: self.rewrite_split_oracle_sig(&split_oracle_def.sig),
        //         code: self.rewrite_oracle_code(split_oracle_def.code),
        //     }
        // }

        pub(crate) fn rewrite_statement(&self, stmt: Statement) -> Statement {
            let type_rewrite_rules: Vec<_> = self
                .type_assignments
                .iter()
                .map(|(name, tipe)| (Type::UserDefined(name.to_string()), tipe.clone()))
                .collect();

            match stmt {
                Statement::Abort(_) => stmt.clone(),
                Statement::Return(expr, pos) => {
                    Statement::Return(expr.clone().map(|expr| self.rewrite_expression(&expr)), pos)
                }

                Statement::Assign(ident, index, value, pos) => Statement::Assign(
                    self.rewrite_identifier(ident),
                    index.clone().map(|expr| self.rewrite_expression(&expr)),
                    self.rewrite_expression(&value),
                    pos,
                ),
                Statement::Parse(idents, expr, pos) => Statement::Parse(
                    idents
                        .into_iter()
                        .map(|ident| self.rewrite_identifier(ident))
                        .collect(),
                    self.rewrite_expression(&expr),
                    pos,
                ),
                Statement::Sample(ident, index, sample_id, tipe, pos) => Statement::Sample(
                    self.rewrite_identifier(ident),
                    index.clone().map(|expr| self.rewrite_expression(&expr)),
                    sample_id,
                    tipe.rewrite_type(&type_rewrite_rules),
                    pos,
                ),
                Statement::InvokeOracle(InvokeOracleStatement {
                    id,
                    opt_idx,
                    opt_dst_inst_idx,
                    name,
                    args,
                    target_inst_name,
                    tipe,
                    file_pos,
                }) => Statement::InvokeOracle(InvokeOracleStatement {
                    name,
                    target_inst_name,
                    file_pos,

                    id: self.rewrite_identifier(id),
                    opt_idx: opt_idx.clone().map(|expr| self.rewrite_expression(&expr)),
                    opt_dst_inst_idx: opt_dst_inst_idx
                        .clone()
                        .map(|expr| self.rewrite_expression(&expr)),
                    args: args
                        .into_iter()
                        .map(|expr| self.rewrite_expression(&expr))
                        .collect(),
                    tipe: tipe
                        .clone()
                        .map(|tipe| tipe.rewrite_type(&type_rewrite_rules)),
                }),

                Statement::IfThenElse(ite) => Statement::IfThenElse(IfThenElse {
                    cond: self.rewrite_expression(&ite.cond),
                    then_block: self.rewrite_oracle_code(ite.then_block),
                    else_block: self.rewrite_oracle_code(ite.else_block),
                    ..ite
                }),
                Statement::For(ident, start, end, code, pos) => Statement::For(
                    ident.clone(),
                    self.rewrite_expression(&start),
                    self.rewrite_expression(&end),
                    self.rewrite_oracle_code(code),
                    pos,
                ),
            }
        }
    }

    impl InstantiationContext<'_> {
        pub(crate) fn rewrite_expression(&self, expr: &Expression) -> Expression {
            expr.map(|expr| match (self.src, expr) {
                (_, Expression::Identifier(ident)) => {
                    Expression::Identifier(self.rewrite_identifier(ident))
                }

                // XXX: Are we sure these should all be unreachable???
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
                            let new_expr = replace.map(|mut expr| match expr {
                                Expression::Identifier(ref mut ident) => match self.src {
                                    InstantiationSource::Package { .. } => {
                                        ident.set_pkg_inst_info(
                                            self.inst_name.to_string(),
                                            self.parent_name.to_string(),
                                        );
                                        expr
                                    }
                                    InstantiationSource::Game { .. } => {
                                        ident.set_game_inst_info(
                                            self.inst_name.to_string(),
                                            self.parent_name.to_string(),
                                        );
                                        expr
                                    }
                                },
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

                // can only happen in oracle code, i.e. package code
                (_, Expression::TableAccess(ident, expr)) => {
                    Expression::TableAccess(self.rewrite_identifier(ident), expr)
                }
                (_, Expression::FnCall(ident, args)) => {
                    Expression::FnCall(self.rewrite_identifier(ident), args)
                }

                (_, expr) => expr,
            })
        }

        pub(crate) fn rewrite_identifier(&self, ident: Identifier) -> Identifier {
            let type_rewrite_rules: Vec<_> = self
                .type_assignments
                .iter()
                .map(|(name, tipe)| (Type::UserDefined(name.to_string()), tipe.clone()))
                .collect();

            // extend the identifier with the instance and parent names
            let ident = match (self.src, ident) {
                (
                    InstantiationSource::Package { .. },
                    Identifier::PackageIdentifier(mut pkg_ident),
                ) => {
                    pkg_ident.set_pkg_inst_info(
                        self.inst_name.to_string(),
                        self.parent_name.to_string(),
                    );
                    pkg_ident.into()
                }
                (
                    InstantiationSource::Game { .. },
                    Identifier::PackageIdentifier(mut pkg_ident),
                ) => {
                    pkg_ident.set_game_inst_info(
                        self.inst_name.to_string(),
                        self.parent_name.to_string(),
                    );
                    pkg_ident.into()
                }
                (InstantiationSource::Game { .. }, Identifier::GameIdentifier(mut game_ident)) => {
                    game_ident.set_game_inst_info(
                        self.inst_name.to_string(),
                        self.parent_name.to_string(),
                    );
                    game_ident.into()
                }

                (InstantiationSource::Package { .. }, ident @ Identifier::GameIdentifier(_))
                | (InstantiationSource::Package { .. }, ident @ Identifier::ProofIdentifier(_))
                | (InstantiationSource::Game { .. }, ident @ Identifier::ProofIdentifier(_)) => {
                    unreachable!(
                        "found\n    {ident:?}\n  when instantiating with context\n    {self:?}",
                        ident = ident,
                        self = self
                    )
                }
                (InstantiationSource::Package { .. }, ident @ Identifier::Generated(_, _))
                | (InstantiationSource::Game { .. }, ident @ Identifier::Generated(_, _)) => ident,
            };

            // rewrite the types
            #[allow(clippy::let_and_return)]
            let new_ident = match ident {
                Identifier::PackageIdentifier(pkg_ident) => {
                    let pkg_ident = match pkg_ident {
                        PackageIdentifier::Const(const_ident) => {
                            PackageIdentifier::Const(PackageConstIdentifier {
                                tipe: const_ident.tipe.rewrite_type(&type_rewrite_rules),
                                ..const_ident
                            })
                        }
                        PackageIdentifier::State(state_ident) => {
                            PackageIdentifier::State(PackageStateIdentifier {
                                tipe: state_ident.tipe.rewrite_type(&type_rewrite_rules),
                                ..state_ident
                            })
                        }
                        PackageIdentifier::Local(local_ident) => {
                            PackageIdentifier::Local(PackageLocalIdentifier {
                                tipe: local_ident.tipe.rewrite_type(&type_rewrite_rules),
                                ..local_ident
                            })
                        }
                        // XXX: A bit unclear whether we keep this variant... it conflicts witht he
                        // Oracle variant of Declaration, so we only need one and so far we use the
                        // other. Also this one doesn't seem to support multi instance
                        PackageIdentifier::OracleImport(_) => todo!(),

                        PackageIdentifier::OracleArg(arg_ident) => {
                            PackageIdentifier::OracleArg(PackageOracleArgIdentifier {
                                tipe: arg_ident.tipe.rewrite_type(&type_rewrite_rules),
                                ..arg_ident.clone()
                            })
                        }
                        // no types to rewrite here
                        ident @ PackageIdentifier::ImportsLoopVar(_) => ident,
                        ident @ PackageIdentifier::CodeLoopVar(_) => ident,
                    };

                    Identifier::PackageIdentifier(pkg_ident)
                }

                Identifier::GameIdentifier(_) => ident.clone(),

                Identifier::ProofIdentifier(_) => unreachable!(
                    "unexpected proof identifier when instantiating package: {:?}",
                    ident
                ),

                other => unreachable!("won't rewrite deprecated identifier {:?}", other),
            };

            new_ident
        }
    }
}
