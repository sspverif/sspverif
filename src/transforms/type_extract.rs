use std::collections::HashSet;
use std::convert::Infallible;

use crate::package::Composition;
use crate::statement::{CodeBlock, InvokeOracleStatement, Statement};
use crate::types::Type;

pub struct Transformation<'a>(pub &'a Composition);

impl super::Transformation for Transformation<'_> {
    type Err = Infallible;
    type Aux = HashSet<Type>;

    fn transform(&self) -> Result<(Composition, HashSet<Type>), Infallible> {
        let mut set = HashSet::new();

        let insts = &self.0.pkgs.iter();
        let oracles = insts.clone().flat_map(|inst| inst.pkg.oracles.clone());
        let codeblocks = oracles.map(|odef| odef.code);

        for cb in codeblocks {
            extract_types_from_codeblock(&mut set, cb);
        }

        Ok((self.0.clone(), set))
    }
}

fn extract_types_from_codeblock(set: &mut HashSet<Type>, cb: CodeBlock) {
    for stmt in cb.0 {
        match stmt {
            Statement::Abort(_) => {}
            Statement::Return(Some(expr), _) => {
                set.insert(expr.get_type());
            }
            Statement::Return(None, _) => {}
            Statement::Assign(_, Some(expr_idx), expr_val, _) => {
                set.insert(expr_idx.get_type());
                set.insert(expr_val.get_type());
            }
            Statement::Assign(_, _, expr_val, _) => {
                set.insert(expr_val.get_type());
            }
            Statement::Parse(_, expr, _) => {
                set.insert(expr.get_type());
            }
            Statement::IfThenElse(ite) => {
                set.insert(ite.cond.get_type());
                extract_types_from_codeblock(set, ite.then_block);
                extract_types_from_codeblock(set, ite.else_block);
            }
            Statement::For(_, lower_bound, upper_bound, body, _) => {
                set.insert(lower_bound.get_type());
                set.insert(upper_bound.get_type());
                extract_types_from_codeblock(set, body)
            }
            Statement::Sample(_, Some(expr_idx), _, t, _) => {
                set.insert(expr_idx.get_type());
                set.insert(t);
            }
            Statement::Sample(_, _, _, t, _) => {
                set.insert(t);
            }
            Statement::InvokeOracle(InvokeOracleStatement {
                opt_idx,
                args,
                tipe,
                ..
            }) => {
                if let Some(expr) = opt_idx {
                    set.insert(expr.get_type());
                }

                if let Some(t) = tipe {
                    set.insert(t);
                }

                for arg in args {
                    set.insert(arg.get_type());
                }
            }
        }
    }
}
