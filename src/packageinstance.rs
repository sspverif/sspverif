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
        split::{SplitOracleDef, SplitOracleSig, SplitPath, SplitType},
        statement::CodeBlock,
    };

    use super::*;

    pub(super) fn rewrite_oracle_def(
        oracle_def: &OracleDef,
        const_assignments: &[(PackageConstIdentifier, Expression)],
        type_assignments: &[(String, Type)],
    ) -> OracleDef {
        OracleDef {
            sig: rewrite_oracle_sig(&oracle_def.sig, const_assignments, type_assignments),
            code: rewrite_oracle_code(&oracle_def.code, const_assignments, type_assignments),
            file_pos: oracle_def.file_pos.clone(),
        }
    }

    pub(super) fn rewrite_split_oracle_def(
        split_oracle_def: &SplitOracleDef,
        const_assignments: &[(PackageConstIdentifier, Expression)],
        type_assignments: &[(String, Type)],
    ) -> SplitOracleDef {
        SplitOracleDef {
            sig: rewrite_split_oracle_sig(
                &split_oracle_def.sig,
                const_assignments,
                type_assignments,
            ),
            code: rewrite_oracle_code(&split_oracle_def.code, const_assignments, type_assignments),
        }
    }

    pub(super) fn rewrite_oracle_sig(
        oracle_sig: &OracleSig,
        const_assignments: &[(PackageConstIdentifier, Expression)],
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
                                rewrite_expression(&start, const_assignments, type_assignments),
                                rewrite_expression(&end, const_assignments, type_assignments),
                            ),
                            SplitType::IfCondition(expr) => SplitType::IfCondition(
                                rewrite_expression(expr, const_assignments, type_assignments),
                            ),
                        },
                        split_range: component.split_range.clone(),
                    })
                    .collect(),
            ),
            tipe: split_oracle_sig.tipe.rewrite(&type_rewrite_rules),
        }
    }

    pub(super) fn rewrite_oracle_code(
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
                .map(|stmt| rewrite_statement(stmt, const_assignments, type_assignments))
                .collect(),
        )
    }

    pub(super) fn rewrite_statement(
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
                expr.clone()
                    .map(|expr| rewrite_expression(&expr, const_assignments, type_assignments)),
                pos.clone(),
            ),
            Statement::Assign(ident, index, value, pos) => Statement::Assign(
                ident.clone(),
                index
                    .clone()
                    .map(|expr| rewrite_expression(&expr, const_assignments, type_assignments)),
                rewrite_expression(value, const_assignments, type_assignments),
                pos.clone(),
            ),
            Statement::Parse(idents, expr, pos) => Statement::Parse(
                idents.clone(),
                rewrite_expression(&expr, const_assignments, type_assignments),
                pos.clone(),
            ),
            Statement::IfThenElse(cond, ifblock, elseblock, pos) => Statement::IfThenElse(
                rewrite_expression(cond, const_assignments, type_assignments),
                rewrite_oracle_code(ifblock, const_assignments, type_assignments),
                rewrite_oracle_code(elseblock, const_assignments, type_assignments),
                pos.clone(),
            ),
            Statement::Sample(ident, index, sample_id, tipe, pos) => Statement::Sample(
                ident.clone(),
                index
                    .clone()
                    .map(|expr| rewrite_expression(&expr, const_assignments, type_assignments)),
                sample_id.clone(),
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
                id: id.clone(),
                opt_idx: opt_idx
                    .clone()
                    .map(|expr| rewrite_expression(&expr, const_assignments, type_assignments)),
                opt_dst_inst_idx: opt_dst_inst_idx
                    .clone()
                    .map(|expr| rewrite_expression(&expr, const_assignments, type_assignments)),
                name: name.clone(),
                args: args
                    .iter()
                    .map(|expr| rewrite_expression(expr, const_assignments, type_assignments))
                    .collect(),
                target_inst_name: target_inst_name.clone(),
                tipe: tipe.clone().map(|tipe| tipe.rewrite(&type_rewrite_rules)),
                file_pos: file_pos.clone(),
            },
            Statement::For(ident, start, end, code, pos) => Statement::For(
                ident.clone(),
                rewrite_expression(start, const_assignments, type_assignments),
                rewrite_expression(end, const_assignments, type_assignments),
                rewrite_oracle_code(code, const_assignments, type_assignments),
                pos.clone(),
            ),
        }
    }

    pub(super) fn rewrite_expression(
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
                    // we can't compare equiality directly, because the value we look up already
                    // has the game value set, so it is not equal and we wouldn't find it.
                    if search.eq_except_game_assignment(&pkg_const_ident) {
                        Some(replace.clone())
                    } else {
                        None
                    }
                })
                .unwrap(),
            other => other.clone(),
        })
    }
}

impl PackageInstance {
    pub(crate) fn new(
        name: String,
        pkg: &Package,
        multi_instance_indices: MultiInstanceIndices,
        params: Vec<(PackageConstIdentifier, Expression)>,
        types: Vec<(String, Type)>,
    ) -> PackageInstance {
        let new_oracles = pkg
            .oracles
            .iter()
            .map(|oracle_def| instantiate::rewrite_oracle_def(oracle_def, &params, &types))
            .collect();

        let new_split_oracles = pkg
            .split_oracles
            .iter()
            .map(|split_oracle_def| rewrite_split_oracle_def(split_oracle_def, &params, &types))
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
            name,
            multi_instance_indices,
        }
    }
}
