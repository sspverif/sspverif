use std::collections::HashSet;
use std::convert::Infallible;

use crate::package::{Composition, Edge, Export};
use crate::statement::{CodeBlock, InvokeOracleStatement, Statement};
use crate::types::Type;

pub struct Transformation<'a>(pub &'a Composition);

impl super::Transformation for Transformation<'_> {
    type Err = Infallible;
    type Aux = HashSet<Type>;

    fn transform(&self) -> Result<(Composition, HashSet<Type>), Infallible> {
        let mut set = HashSet::new();

        // extract game const types
        set.extend(self.0.consts.iter().map(|(_, ty)| ty).cloned());

        // extract oracle arg an return types from edges
        set.extend(self.0.edges.iter().flat_map(|Edge(_, _, sig)| {
            sig.args
                .iter()
                .map(|(_, ty)| ty)
                .chain(Some(&sig.tipe))
                .cloned()
        }));

        // extract oracle arg an return types from exports
        set.extend(self.0.exports.iter().flat_map(|Export(_, sig)| {
            sig.args
                .iter()
                .map(|(_, ty)| ty)
                .chain(Some(&sig.tipe))
                .cloned()
        }));

        // extract types from package params, state, imported oracle signatures and defined oracle
        // seignatures.
        set.extend(self.0.pkgs.iter().flat_map(|pkg_inst| {
            let pkg = &pkg_inst.pkg;

            // first prepare iterators for all the relevant types that are extracted
            let params = pkg.params.iter().map(|(_, ty, _)| ty);
            let state = pkg.state.iter().map(|(_, ty, _)| ty);
            let oracle_imports = pkg
                .imports
                .iter()
                .flat_map(|(sig, _)| sig.args.iter().map(|(_, ty)| ty).chain(Some(&sig.tipe)));
            let oracle_definitions = pkg.oracles.iter().flat_map(|oracle_def| {
                let sig = &oracle_def.sig;
                sig.args.iter().map(|(_, ty)| ty).chain(Some(&sig.tipe))
            });

            // Then chain them and clone the items, because the set wants owned types
            params
                .chain(state)
                .chain(oracle_imports)
                .chain(oracle_definitions)
                .cloned()
        }));

        let insts = &self.0.pkgs.iter();
        let oracles = insts.clone().flat_map(|inst| inst.pkg.oracles.clone());

        for oracle in oracles {
            extract_types_from_codeblock(&mut set, oracle.code);
        }

        Ok((self.0.clone(), set))
    }
}

// This is a separate function so it's easier to inject debug printing that should happen in all
// cases.
fn record_type(set: &mut HashSet<Type>, ty: Type) {
    set.insert(ty);
}

fn extract_types_from_codeblock(set: &mut HashSet<Type>, cb: CodeBlock) {
    for stmt in cb.0 {
        match stmt {
            Statement::Abort(_) => {}
            Statement::Return(Some(expr), _) => {
                record_type(set, expr.get_type());
            }
            Statement::Return(None, _) => {}
            Statement::Assign(_, Some(expr_idx), expr_val, _) => {
                record_type(set, expr_idx.get_type());
                record_type(set, expr_val.get_type());
            }
            Statement::Assign(_, _, expr_val, _) => {
                record_type(set, expr_val.get_type());
            }
            Statement::Parse(_, expr, _) => {
                record_type(set, expr.get_type());
            }
            Statement::IfThenElse(ite) => {
                record_type(set, ite.cond.get_type());
                extract_types_from_codeblock(set, ite.then_block);
                extract_types_from_codeblock(set, ite.else_block);
            }
            Statement::For(_, lower_bound, upper_bound, body, _) => {
                record_type(set, lower_bound.get_type());
                record_type(set, upper_bound.get_type());
                extract_types_from_codeblock(set, body)
            }
            Statement::Sample(_, Some(expr_idx), _, ty, _) => {
                record_type(set, expr_idx.get_type());
                record_type(set, ty);
            }
            Statement::Sample(_, _, _, ty, _) => {
                record_type(set, ty);
            }
            Statement::InvokeOracle(InvokeOracleStatement {
                opt_idx,
                args,
                tipe,
                ..
            }) => {
                if let Some(expr) = opt_idx {
                    record_type(set, expr.get_type());
                }

                if let Some(ty) = tipe {
                    record_type(set, ty);
                }

                for arg in args {
                    record_type(set, arg.get_type());
                }
            }
        }
    }
}
