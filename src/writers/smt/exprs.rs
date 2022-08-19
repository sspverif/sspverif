use std::io::{Result, Write};

use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::types::Type;

pub fn smt_to_string<T: Into<SmtExpr>>(t: T) -> String {
    let expr: SmtExpr = t.into();
    expr.to_string()
}

pub trait SmtFmt {
    fn write_smt_to<T: Write>(&self, write: &mut T) -> Result<()>;

    fn to_string(&self) -> String {
        let mut buf = vec![];
        self.write_smt_to(&mut buf)
            .expect("can't happen, we assume the buffer never errors");

        String::from_utf8(buf).expect("can't happen, we only write utf8")
    }
}

#[derive(Debug, Clone)]
pub enum SmtExpr {
    Comment(String),
    Atom(String),
    List(Vec<SmtExpr>),
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
                }
                write!(write, ")")
            }
        }
    }
}

impl From<Expression> for SmtExpr {
    fn from(expr: Expression) -> SmtExpr {
        match expr {
            Expression::Typed(t, inner) if *inner == Expression::EmptyTable => {
                if let Type::Table(idxtipe, valtipe) = t {
                    let idxtipe = *idxtipe;
                    SmtExpr::List(vec![
                        SmtExpr::List(vec![
                            SmtExpr::Atom("as".into()),
                            SmtExpr::Atom("const".into()),
                            SmtExpr::List(vec![
                                SmtExpr::Atom("Array".into()),
                                idxtipe.into(),
                                Type::Maybe(valtipe.clone()).into(),
                            ]),
                        ]),
                        SmtExpr::List(vec![
                            SmtExpr::Atom("as".into()),
                            SmtExpr::Atom("mk-none".into()),
                            Type::Maybe(valtipe).into(),
                        ]),
                    ])
                }
                else {
                    panic!("Empty table of type {:?}", t)
                }
            }
            Expression::Typed(_t, inner) => SmtExpr::from(*inner),
            Expression::Unwrap(_inner) => {
                panic!("unwrap expressions need to be on the right hand side of an assign!");
                // TODO find a better way to present that error to the user.
            }
            Expression::Some(inner) => {
                SmtExpr::List(vec![SmtExpr::Atom("mk-some".into()), SmtExpr::from(*inner)])
            }
            Expression::None(inner) => SmtExpr::List(vec![
                SmtExpr::Atom("as".into()),
                SmtExpr::Atom("mk-none".into()),
                Type::Maybe(Box::new(inner)).into(),
            ]),
            Expression::StringLiteral(litname) => SmtExpr::Atom(format!("\"{}\"", litname)),
            Expression::BooleanLiteral(litname) => SmtExpr::Atom(litname),
            Expression::IntegerLiteral(litname) => SmtExpr::Atom(litname),
            Expression::Equals(exprs) => {
                let mut acc = vec![SmtExpr::Atom("=".to_string())];
                for expr in exprs {
                    acc.push(expr.clone().into());
                }

                SmtExpr::List(acc)
            }
            Expression::Add(lhs, rhs) => SmtExpr::List(vec![
                SmtExpr::Atom("+".to_string()),
                SmtExpr::from(*lhs),
                SmtExpr::from(*rhs),
            ]),
            Expression::Sub(lhs, rhs) => SmtExpr::List(vec![
                SmtExpr::Atom("-".to_string()),
                SmtExpr::from(*lhs),
                SmtExpr::from(*rhs),
            ]),
            Expression::Mul(lhs, rhs) => SmtExpr::List(vec![
                SmtExpr::Atom("*".to_string()),
                SmtExpr::from(*lhs),
                SmtExpr::from(*rhs),
            ]),
            Expression::Div(lhs, rhs) => SmtExpr::List(vec![
                SmtExpr::Atom("/".to_string()),
                SmtExpr::from(*lhs),
                SmtExpr::from(*rhs),
            ]),
            Expression::Not(expr) => {
                SmtExpr::List(vec![SmtExpr::Atom("not".to_string()), (*expr).into()])
            }
            Expression::And(vals) => SmtExpr::List({
                let mut list = vec![SmtExpr::Atom("and".to_owned())];
                for val in vals {
                    list.push(SmtExpr::from(val))
                }
                list
            }),
            Expression::Or(vals) => SmtExpr::List({
                let mut list = vec![SmtExpr::Atom("or".to_owned())];
                for val in vals {
                    list.push(SmtExpr::from(val))
                }
                list
            }),
            Expression::Xor(vals) => SmtExpr::List({
                let mut list = vec![SmtExpr::Atom("xor".to_owned())];
                for val in vals {
                    list.push(SmtExpr::from(val))
                }
                list
            }),
            Expression::Identifier(Identifier::Scalar(identname)) => {
                panic! {"Found a scalar {:} which should have been removed by varspecify at this point", identname}
            }
            Expression::Identifier(Identifier::Local(identname)) => SmtExpr::Atom(identname),
            Expression::Identifier(Identifier::State {
                name: identname,
                pkgname,
                compname,
            }) => SmtExpr::List(vec![
                SmtExpr::Atom(format!("state-{}-{}-{}", compname, pkgname, identname)),
                SspSmtVar::SelfState.into(),
            ]),
            Expression::Bot => SmtExpr::Atom("bot".to_string()),
            Expression::TableAccess(table, index) => SmtExpr::List(vec![
                SmtExpr::Atom("select".into()),
                table.to_expression().into(),
                (*index).into(),
            ]),
            Expression::Tuple(exprs) => {
                let mut l = vec![SmtExpr::Atom(format!("mk-tuple{}", exprs.len()))];

                for expr in exprs {
                    l.push(expr.into())
                }

                SmtExpr::List(l)
            }
            Expression::FnCall(name, exprs) => {
                let mut call = vec![SmtExpr::Atom(name)];

                for expr in exprs {
                    call.push(expr.into());
                }

                SmtExpr::List(call)
            }
            Expression::List(inner) => {
                let t = if let Expression::Typed(t, _) = inner[0].clone() {
                    Some(t)
                } else {
                    None
                };

                let t = t.unwrap();

                let nil = SmtExpr::List(vec![
                    SmtExpr::Atom("as".to_owned()),
                    SmtExpr::Atom("nil".to_owned()),
                    SmtExpr::List(vec![SmtExpr::Atom("List".to_owned()), t.into()]),
                ]);

                let mut lst = nil;

                for el in inner.iter().rev() {
                    lst =
                        SmtExpr::List(vec![SmtExpr::Atom("insert".into()), el.clone().into(), lst])
                }

                lst
            }
            Expression::Set(inner) => {
                let t = if let Expression::Typed(t, _) = inner[0].clone() {
                    Some(t)
                } else {
                    None
                };

                let t = t.unwrap();

                let empty_set = SmtExpr::List(vec![
                    SmtExpr::List(vec![
                        SmtExpr::Atom("as".to_owned()),
                        SmtExpr::Atom("const".to_owned()),
                        SmtExpr::List(vec![SmtExpr::Atom("Set".to_owned()), t.into()]),
                    ]),
                    SmtExpr::Atom("false".to_string()),
                ]);

                let mut set = empty_set;

                for el in inner.iter().rev() {
                    set = SmtExpr::List(vec![
                        SmtExpr::Atom("store".into()),
                        set,
                        el.clone().into(),
                        SmtExpr::Atom("true".to_string()),
                    ])
                }

                set
            }
            _ => {
                panic!("not implemented: {:?}", expr);
            }
        }
    }
}

impl From<Type> for SmtExpr {
    fn from(t: Type) -> SmtExpr {
        match t {
            Type::Bits(length) => {
                // TODO make sure we define this somewhere
                SmtExpr::Atom(format!("Bits_{}", length))
            }
            Type::Maybe(t) => SmtExpr::List(vec![SmtExpr::Atom("Maybe".into()), (*t).into()]),
            Type::Boolean => SmtExpr::Atom("Bool".to_string()),
            Type::Integer => SmtExpr::Atom("Int".into()),
            Type::Table(t_idx, t_val) => SmtExpr::List(vec![
                SmtExpr::Atom("Array".into()),
                (*t_idx).into(),
                Type::Maybe(t_val).into(),
            ]),
            Type::Tuple(types) => SmtExpr::List({
                let mut els = vec![SmtExpr::Atom(format!("Tuple{}", types.len()))];
                for t in types {
                    els.push(t.into());
                }
                els
            }),
            _ => {
                panic!("not implemented: {:?}", t)
            }
        }
    }
}

impl<C, T, E> From<SmtIte<C, T, E>> for SmtExpr
where
    C: Into<SmtExpr>,
    T: Into<SmtExpr>,
    E: Into<SmtExpr>,
{
    fn from(ite: SmtIte<C, T, E>) -> SmtExpr {
        SmtExpr::List(vec![
            SmtExpr::Atom("ite".into()),
            ite.cond.into(),
            ite.then.into(),
            ite.els.into(),
        ])
    }
}

impl<C, E> From<SmtIs<C, E>> for SmtExpr
where
    C: Into<String>,
    E: Into<SmtExpr>,
{
    fn from(is: SmtIs<C, E>) -> SmtExpr {
        SmtExpr::List(vec![
            SmtExpr::List(vec![
                SmtExpr::Atom("_".into()),
                SmtExpr::Atom("is".into()),
                SmtExpr::Atom(is.con.into()),
            ]),
            is.expr.into(),
        ])
    }
}

impl<L, R> From<SmtEq2<L, R>> for SmtExpr
where
    L: Into<SmtExpr>,
    R: Into<SmtExpr>,
{
    fn from(eq: SmtEq2<L, R>) -> Self {
        SmtExpr::List(vec![
            SmtExpr::Atom("=".to_string()),
            eq.lhs.into(),
            eq.rhs.into(),
        ])
    }
}

impl<B> From<SmtLet<B>> for SmtExpr
where
    B: Into<SmtExpr>,
{
    fn from(l: SmtLet<B>) -> SmtExpr {
        if l.bindings.is_empty() {
            return l.body.into();
        }

        SmtExpr::List(vec![
            SmtExpr::Atom(String::from("let")),
            SmtExpr::List(
                l.bindings
                    .into_iter()
                    .map(|(id, expr)| SmtExpr::List(vec![SmtExpr::Atom(id), expr]))
                    .collect(),
            ),
            l.body.into(),
        ])
    }
}

impl From<SmtAs> for SmtExpr {
    fn from(smtas: SmtAs) -> Self {
        SmtExpr::List(vec![
            SmtExpr::Atom("as".to_string()),
            SmtExpr::Atom(smtas.name),
            smtas.tipe.into(),
        ])
    }
}

impl From<SspSmtVar> for SmtExpr {
    fn from(v: SspSmtVar) -> SmtExpr {
        match v {
            SspSmtVar::GlobalState => SmtExpr::Atom("__global_state".into()),
            SspSmtVar::SelfState => SmtExpr::Atom("__self_state".into()),
            SspSmtVar::ReturnValue => SmtExpr::Atom("__ret".into()),
            SspSmtVar::OracleReturnConstructor {
                compname,
                pkgname,
                oname,
            } => SmtExpr::Atom(format!("mk-return-{}-{}-{}", compname, pkgname, oname)),
            SspSmtVar::OracleAbort {
                compname,
                pkgname,
                oname,
            } => SmtExpr::Atom(format!("mk-abort-{}-{}-{}", compname, pkgname, oname)),
        }
    }
}

pub struct SmtLet<B>
where
    B: Into<SmtExpr>,
{
    pub bindings: Vec<(String, SmtExpr)>,
    pub body: B,
}

pub struct SmtEq2<L, R>
where
    L: Into<SmtExpr>,
    R: Into<SmtExpr>,
{
    pub lhs: L,
    pub rhs: R,
}

pub struct SmtAs {
    pub name: String,
    pub tipe: Type,
}

pub struct SmtIte<C, T, E>
where
    C: Into<SmtExpr>,
    T: Into<SmtExpr>,
    E: Into<SmtExpr>,
{
    pub cond: C,
    pub then: T,
    pub els: E,
}

pub struct SmtIs<C, E>
where
    C: Into<String>,
    E: Into<SmtExpr>,
{
    pub con: C,
    pub expr: E,
}

pub enum SspSmtVar {
    GlobalState,
    SelfState,
    ReturnValue,
    OracleReturnConstructor {
        compname: String,
        pkgname: String,
        oname: String,
    },
    OracleAbort {
        compname: String,
        pkgname: String,
        oname: String,
    },
}
