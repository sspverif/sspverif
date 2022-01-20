use std::io::{Result, Write};
//use std::io::prelude::*;

use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::types::Type;

pub trait SmtFmt {
    fn write_smt_to<T: Write>(&self, write: &mut T) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum SmtExpr {
    Comment(String),
    Atom(String),
    List(Vec<SmtExpr>)
}

impl SmtFmt for SmtExpr {
    fn write_smt_to<T: Write>(&self, write: &mut T) -> Result<()> {
        match self {
            SmtExpr::Comment(str) => write!(write, "; {}", str),
            SmtExpr::Atom(str) => write!(write, "{}", str),
            SmtExpr::List(lst) => {
                let mut peek = lst.iter().peekable();
                
                write!(write, "(")?;
                while let Some(elem) = peek.next() {
                    elem.write_smt_to(write)?;

                    if peek.peek().is_some() {
                    write!(write, " ")?;
                    }
                };
                write!(write, ")")
            }
        }
    }
}


pub fn statevarname() -> SmtExpr {
    /*
    SmtExpr::List(vec![
        SmtExpr::Atom("'".to_string()),
        SmtExpr::Atom("sspds-rs".to_string()),
        SmtExpr::Atom("state".to_string()),
    ])
    */

    SmtExpr::Atom(String::from("sspds-rs-state"))
}


impl Into<SmtExpr> for Expression {
    fn into(self) -> SmtExpr {
        match self {
            Expression::BooleanLiteral(litname) => {
                SmtExpr::Atom(litname)
            },
            Expression::Equals(exprs) => {
                let mut acc = vec![];
                acc.push(SmtExpr::Atom("=".to_string()));
                for expr in exprs {
                    acc.push(expr.clone().into());
                }
                SmtExpr::List(acc)
            },
            Expression::Identifier(Identifier::Scalar(identname)) => {
                SmtExpr::Atom(identname)
            },
            Expression::Identifier(Identifier::Local(identname)) => {
                SmtExpr::Atom(identname)
            },
            Expression::Identifier(Identifier::State{name:identname, pkgname}) => {
                SmtExpr::List(vec![SmtExpr::Atom(format!("state-{}-{}", pkgname, identname)), statevarname()])
            },
            Expression::Bot => {
                SmtExpr::Atom("bot".to_string())
            },
            Expression::Sample(tipe) => {
                // TODO: fix this later! This is generally speaking not correct!
                SmtExpr::Atom("rand".to_string())
            },
            Expression::FnCall(name, exprs) => {
                let mut call = vec![
                    SmtExpr::Atom(name),
                ];

                for expr in exprs {
                    call.push(expr.into());
                }

                SmtExpr::List(call)
            },
            _ => { panic!("not implemented: {:?}", self); }
        }
    }
}

impl Into<SmtExpr> for Type {
    fn into(self) -> SmtExpr {
        match &self {
            Type::Bits(length) => {
                // TODO make sure we define this somewhere
                SmtExpr::Atom(format!("Bits_{}", length))
            },
            Type::Boolean => {
                SmtExpr::Atom("Bool".to_string())
            },
            _ => {panic!("not implemented!")}
        }
    }
}
