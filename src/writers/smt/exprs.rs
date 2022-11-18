use std::fmt::Display;

use crate::types::Type;

pub fn smt_to_string<T: Into<SmtExpr>>(t: T) -> String {
    t.into().to_string()
}

#[derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq)]
pub enum SmtExpr {
    Comment(String),
    Atom(String),
    List(Vec<SmtExpr>),
}

impl From<Vec<SmtExpr>> for SmtExpr {
    fn from(lst: Vec<SmtExpr>) -> Self {
        SmtExpr::List(lst)
    }
}

impl From<&str> for SmtExpr {
    fn from(atom: &str) -> Self {
        SmtExpr::Atom(atom.to_string())
    }
}

impl From<String> for SmtExpr {
    fn from(atom: String) -> Self {
        SmtExpr::Atom(atom)
    }
}

impl<V: Into<SmtExpr>> From<SmtNot<V>> for SmtExpr {
    fn from(value: SmtNot<V>) -> Self {
        SmtExpr::List(vec!["not".into(), value.0.into()])
    }
}

impl From<SmtAnd> for SmtExpr {
    fn from(terms: SmtAnd) -> Self {
        let mut list: Vec<SmtExpr> = vec!["and".into()];
        list.extend(terms.0);
        SmtExpr::List(list)
    }
}

impl<PRE: Into<SmtExpr>, POST: Into<SmtExpr>> From<SmtImplies<PRE, POST>> for SmtExpr {
    fn from(value: SmtImplies<PRE, POST>) -> Self {
        SmtExpr::List(vec!["=>".into(), value.0.into(), value.1.into()])
    }
}

impl<B: Into<SmtExpr>> From<SmtAssert<B>> for SmtExpr {
    fn from(asrt: SmtAssert<B>) -> Self {
        SmtExpr::List(vec!["assert".into(), asrt.0.into()])
    }
}

impl<T1: Into<SmtExpr>, T2: Into<SmtExpr>> From<(T1, T2)> for SmtExpr {
    fn from(lst: (T1, T2)) -> SmtExpr {
        let (v1, v2) = lst;
        SmtExpr::List(vec![v1.into(), v2.into()])
    }
}

impl<T1: Into<SmtExpr>, T2: Into<SmtExpr>, T3: Into<SmtExpr>> From<(T1, T2, T3)> for SmtExpr {
    fn from(lst: (T1, T2, T3)) -> SmtExpr {
        let (v1, v2, v3) = lst;
        SmtExpr::List(vec![v1.into(), v2.into(), v3.into()])
    }
}

impl<T1: Into<SmtExpr>, T2: Into<SmtExpr>, T3: Into<SmtExpr>, T4: Into<SmtExpr>>
    From<(T1, T2, T3, T4)> for SmtExpr
{
    fn from(lst: (T1, T2, T3, T4)) -> SmtExpr {
        let (v1, v2, v3, v4) = lst;
        SmtExpr::List(vec![v1.into(), v2.into(), v3.into(), v4.into()])
    }
}

impl Display for SmtExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmtExpr::Comment(str) => write!(f, "; {}", str),
            SmtExpr::Atom(str) => write!(f, "{}", str),
            SmtExpr::List(lst) => {
                let mut peek = lst.iter().peekable();

                write!(f, "(")?;
                while let Some(elem) = peek.next() {
                    elem.fmt(f)?;

                    if peek.peek().is_some() {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")\n")
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
            Type::Empty => SmtExpr::Atom("Empty".to_string()),
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

impl From<&Type> for SmtExpr {
    fn from(t: &Type) -> SmtExpr {
        match t {
            Type::Bits(length) => {
                // TODO make sure we define this somewhere
                SmtExpr::Atom(format!("Bits_{}", length))
            }
            Type::Maybe(t) => SmtExpr::List(vec![SmtExpr::Atom("Maybe".into()), (&**t).into()]),
            Type::Boolean => SmtExpr::Atom("Bool".to_string()),
            Type::Integer => SmtExpr::Atom("Int".into()),
            Type::Table(t_idx, t_val) => SmtExpr::List(vec![
                SmtExpr::Atom("Array".into()),
                (&**t_idx).into(),
                Type::Maybe(t_val.clone()).into(),
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
            SspSmtVar::CompositionContext => "__global_state".into(),
            SspSmtVar::ContextLength => "__state_length".into(),
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

pub struct SmtAssert<B: Into<SmtExpr>>(pub B);

pub struct SmtNot<V: Into<SmtExpr>>(pub V);

pub struct SmtAnd(pub Vec<SmtExpr>);

pub struct SmtImplies<PRE, POST>(pub PRE, pub POST)
where
    PRE: Into<SmtExpr>,
    POST: Into<SmtExpr>;

pub enum SspSmtVar {
    CompositionContext,
    ContextLength,
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
