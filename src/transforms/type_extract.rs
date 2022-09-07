use std::collections::HashSet;

use crate::expressions::Expression;
use crate::package::Composition;
use crate::statement::{CodeBlock, Statement};
use crate::types::Type;

#[derive(Debug, Clone)]
pub struct Error(pub String);

pub struct Transformation<'a>(pub &'a Composition);

impl<'a> super::Transformation for Transformation<'a> {
    type Err = Error;
    type Aux = HashSet<Type>;

    fn transform(&self) -> Result<(Composition, HashSet<Type>), Error> {
        let mut set = HashSet::new();

        let insts = &self.0.pkgs.iter();
        let oracles = insts.clone().map(|inst| inst.pkg.oracles.clone()).flatten();
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
            Statement::Abort => {}
            Statement::Return(Some(expr)) => extract_types_from_expression(set, &expr),
            Statement::Return(None) => {}
            Statement::Assign(_, Some(expr_idx), expr_val) => {
                extract_types_from_expression(set, &expr_idx);
                extract_types_from_expression(set, &expr_val);
            }
            Statement::Assign(_, _, expr_val) => extract_types_from_expression(set, &expr_val),
            Statement::Parse(_, expr) => extract_types_from_expression(set, &expr),
            Statement::IfThenElse(cond, cb_left, cb_right) => {
                extract_types_from_expression(set, &cond);
                extract_types_from_codeblock(set, cb_left);
                extract_types_from_codeblock(set, cb_right);
            }
            Statement::Sample(_, Some(expr_idx), _, t) => {
                extract_types_from_expression(set, &expr_idx);
                set.insert(t);
            }
            Statement::Sample(_, _, _, t) => {
                set.insert(t);
            }
            Statement::InvokeOracle {
                opt_idx,
                args,
                tipe,
                ..
            } => {
                if let Some(expr) = opt_idx {
                    extract_types_from_expression(set, &expr);
                }

                if let Some(t) = tipe {
                    set.insert(t);
                }

                for arg in args {
                    extract_types_from_expression(set, &arg);
                }
            }
        }
    }
}

fn extract_types_from_expression(set: &mut HashSet<Type>, expr: &Expression) {
    match expr {
        Expression::Typed(t, _) => {
            set.insert(t.to_owned());
        }
        ex => {
            println!("found unexpected untyped expression {ex:?} during type extraction :(");
        }
    }
}
