use crate::identifier::Identifier;
use crate::types::Type;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Expression {
    Typed(Type, Box<Expression>),
    Bot,
    Sample(Type),
    StringLiteral(String),
    IntegerLiteral(i64),
    BooleanLiteral(String),
    Identifier(Identifier),
    EmptyTable,
    TableAccess(Identifier, Box<Expression>),
    Tuple(Vec<Expression>),
    List(Vec<Expression>),
    Set(Vec<Expression>),
    FnCall(Identifier, Vec<Expression>),
    // or maybe at some point: FnCall(Box<Expression>, Vec<Expression>),
    None(Type),
    Some(Box<Expression>),
    Unwrap(Box<Expression>),

    // Scalar Operations:
    Not(Box<Expression>), // 1-x (not really, in fact: true \mapsto false; false \mapsto true)
    Neg(Box<Expression>), //  -x
    Inv(Box<Expression>), // 1/x

    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Pow(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),

    Equals(Vec<Expression>),
    And(Vec<Expression>),
    Or(Vec<Expression>),
    Xor(Vec<Expression>),

    // Set/List Operations:
    Sum(Box<Expression>),
    Prod(Box<Expression>),
    Any(Box<Expression>), // set/list or
    All(Box<Expression>), // set/list and
    Union(Box<Expression>),
    Cut(Box<Expression>),
    SetDiff(Box<Expression>),

    Concat(Vec<Expression>),
}

impl Expression {
    /*
    fn new_identifier(name: &str) -> Expression {
        Expression::Identifier(name.to_string())
    }
    */

    pub fn walk(&mut self, f: &mut impl FnMut(&mut Expression) -> bool) {
        if !f(self) {
            return;
        }

        match self {
            Expression::Bot
            | Expression::EmptyTable
            | Expression::None(_)
            | Expression::Sample(_)
            | Expression::StringLiteral(_)
            | Expression::IntegerLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::Identifier(_) => {}

            Expression::Not(expr)
            | Expression::Some(expr)
            | Expression::Unwrap(expr)
            | Expression::TableAccess(_, expr)
            | Expression::Typed(_, expr) => expr.as_mut().walk(f),

            Expression::Tuple(exprs)
            | Expression::Equals(exprs)
            | Expression::And(exprs)
            | Expression::Or(exprs)
            | Expression::FnCall(_, exprs)
            | Expression::List(exprs)
            | Expression::Set(exprs)
            | Expression::Xor(exprs) => {
                for expr in exprs {
                    expr.walk(f)
                }
            }

            Expression::Add(lhs, rhs)
            | Expression::Sub(lhs, rhs)
            | Expression::Mul(lhs, rhs)
            | Expression::Div(lhs, rhs) => {
                lhs.as_mut().walk(f);
                rhs.as_mut().walk(f)
            }

            _ => {
                panic!("Expression: not implemented: {:#?}", self)
            }
        }
    }

    pub fn map<F>(&self, f: F) -> Expression
    where
        F: Fn(Expression) -> Expression + Copy,
    {
        f(match &self {
            Expression::Bot
            | Expression::EmptyTable
            | Expression::None(_)
            | Expression::Sample(_)
            | Expression::StringLiteral(_)
            | Expression::IntegerLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::Identifier(_) => self.clone(),

            Expression::Not(expr) => Expression::Not(Box::new(expr.map(f))),
            Expression::Some(expr) => Expression::Some(Box::new(expr.map(f))),
            Expression::Unwrap(expr) => Expression::Unwrap(Box::new(expr.map(f))),
            Expression::TableAccess(id, expr) => {
                Expression::TableAccess(id.clone(), Box::new(expr.map(f)))
            }
            Expression::Tuple(exprs) => {
                Expression::Tuple(exprs.iter().map(|expr| expr.map(f)).collect())
            }
            Expression::Equals(exprs) => {
                Expression::Equals(exprs.iter().map(|expr| expr.map(f)).collect())
            }
            Expression::Xor(exprs) => {
                Expression::Xor(exprs.iter().map(|expr| expr.map(f)).collect())
            }
            Expression::And(exprs) => {
                Expression::And(exprs.iter().map(|expr| expr.map(f)).collect())
            }
            Expression::Or(exprs) => Expression::Or(exprs.iter().map(|expr| expr.map(f)).collect()),
            Expression::Add(lhs, rhs) => {
                Expression::Add(Box::new(lhs.map(f)), Box::new(rhs.map(f)))
            }
            Expression::Sub(lhs, rhs) => {
                Expression::Sub(Box::new(lhs.map(f)), Box::new(rhs.map(f)))
            }
            Expression::Mul(lhs, rhs) => {
                Expression::Mul(Box::new(lhs.map(f)), Box::new(rhs.map(f)))
            }
            Expression::Div(lhs, rhs) => {
                Expression::Div(Box::new(lhs.map(f)), Box::new(rhs.map(f)))
            }
            Expression::FnCall(name, exprs) => {
                Expression::FnCall(name.clone(), exprs.iter().map(|expr| expr.map(f)).collect())
            }
            Expression::Typed(t, inner) => Expression::Typed(t.clone(), Box::new(inner.map(f))),
            Expression::List(exprs) => {
                Expression::List(exprs.iter().map(|expr| expr.map(f)).collect())
            }
            Expression::Set(exprs) => {
                Expression::Set(exprs.iter().map(|expr| expr.map(f)).collect())
            }
            _ => {
                panic!("Expression: not implemented: {:#?}", self)
            }
        })
    }

    pub fn mapfold<F, Ac>(&self, init: Ac, f: F) -> (Ac, Expression)
    where
        F: Fn(Ac, Expression) -> (Ac, Expression) + Copy,
        Ac: Clone,
    {
        let (ac, ex) = match &self {
            Expression::Bot
            | Expression::EmptyTable
            | Expression::None(_)
            | Expression::Sample(_)
            | Expression::StringLiteral(_)
            | Expression::IntegerLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::Identifier(_) => (init, self.clone()),

            Expression::Not(expr) => {
                let (ac, e) = expr.mapfold(init, f);
                (ac, Expression::Not(Box::new(e)))
            }
            Expression::Some(expr) => {
                let (ac, e) = expr.mapfold(init, f);
                (ac, Expression::Some(Box::new(e)))
            }
            Expression::Unwrap(expr) => {
                let (ac, e) = expr.mapfold(init, f);
                (ac, Expression::Unwrap(Box::new(e)))
            }
            Expression::TableAccess(id, expr) => {
                let (ac, e) = expr.mapfold(init, f);
                (ac, Expression::TableAccess(id.clone(), Box::new(e)))
            }
            Expression::Tuple(exprs) => {
                let mut ac = init;
                let newexprs = exprs
                    .iter()
                    .map(|expr| {
                        let (newac, e) = expr.mapfold(ac.clone(), f);
                        ac = newac;
                        e
                    })
                    .collect();
                (ac, Expression::Tuple(newexprs))
            }
            Expression::Xor(exprs) => {
                let mut ac = init;
                let newexprs = exprs
                    .iter()
                    .map(|expr| {
                        let (newac, e) = expr.mapfold(ac.clone(), f);
                        ac = newac;
                        e
                    })
                    .collect();
                (ac, Expression::Xor(newexprs))
            }
            Expression::Equals(exprs) => {
                let mut ac = init;
                let newexprs = exprs
                    .iter()
                    .map(|expr| {
                        let (newac, e) = expr.mapfold(ac.clone(), f);
                        ac = newac;
                        e
                    })
                    .collect();
                (ac, Expression::Equals(newexprs))
            }
            Expression::And(exprs) => {
                let mut ac = init;
                let newexprs = exprs
                    .iter()
                    .map(|expr| {
                        let (newac, e) = expr.mapfold(ac.clone(), f);
                        ac = newac;
                        e
                    })
                    .collect();
                (ac, Expression::And(newexprs))
            }
            Expression::Or(exprs) => {
                let mut ac = init;
                let newexprs = exprs
                    .iter()
                    .map(|expr| {
                        let (newac, e) = expr.mapfold(ac.clone(), f);
                        ac = newac;
                        e
                    })
                    .collect();
                (ac, Expression::Or(newexprs))
            }
            Expression::Add(lhs, rhs) => {
                let ac = init;
                let (ac, newlhs) = lhs.mapfold(ac, f);
                let (ac, newrhs) = rhs.mapfold(ac, f);
                (ac, Expression::Add(Box::new(newlhs), Box::new(newrhs)))
            }
            Expression::Sub(lhs, rhs) => {
                let ac = init;
                let (ac, newlhs) = lhs.mapfold(ac, f);
                let (ac, newrhs) = rhs.mapfold(ac, f);
                (ac, Expression::Sub(Box::new(newlhs), Box::new(newrhs)))
            }
            Expression::Mul(lhs, rhs) => {
                let ac = init;
                let (ac, newlhs) = lhs.mapfold(ac, f);
                let (ac, newrhs) = rhs.mapfold(ac, f);
                (ac, Expression::Mul(Box::new(newlhs), Box::new(newrhs)))
            }
            Expression::Div(lhs, rhs) => {
                let ac = init;
                let (ac, newlhs) = lhs.mapfold(ac, f);
                let (ac, newrhs) = rhs.mapfold(ac, f);
                (ac, Expression::Div(Box::new(newlhs), Box::new(newrhs)))
            }
            Expression::FnCall(name, exprs) => {
                let mut ac = init;
                let newexprs = exprs
                    .iter()
                    .map(|expr| {
                        let (newac, e) = expr.mapfold(ac.clone(), f);
                        ac = newac;
                        e
                    })
                    .collect();

                (ac, Expression::FnCall(name.clone(), newexprs))
            }
            Expression::Typed(t, inner) => {
                let (ac, e) = inner.mapfold(init, f);
                (ac, Expression::Typed(t.clone(), Box::new(e)))
            }
            Expression::List(inner) => {
                let mut ac = init;
                let newexprs = inner
                    .iter()
                    .map(|expr| {
                        let (newac, e) = expr.mapfold(ac.clone(), f);
                        ac = newac;
                        e
                    })
                    .collect();
                (ac, Expression::List(newexprs))
            }
            Expression::Set(inner) => {
                let mut ac = init;
                let newexprs = inner
                    .iter()
                    .map(|expr| {
                        let (newac, e) = expr.mapfold(ac.clone(), f);
                        ac = newac;
                        e
                    })
                    .collect();
                (ac, Expression::Set(newexprs))
            }
            _ => {
                panic!("Expression: not implemented: {:#?}", self)
            }
        };
        f(ac, ex)
    }

    pub fn new_equals(exprs: Vec<&Expression>) -> Expression {
        Expression::Equals(exprs.into_iter().cloned().collect())
    }

    pub fn get_type(&self) -> Option<&Type> {
        match self {
            Expression::Typed(t, _expr) => Some(t),
            _ => None,
        }
    }
}

#[macro_export]
macro_rules! tuple {
    ( $($e:expr),* ) => {
        {
            Expression::Tuple(vec![ $( $e.clone(), )* ])
        }
    };
}

#[macro_export]
macro_rules! list {
    ( $($e:expr),* ) => {
        {
            Expression::List(vec![ $( $e.clone(), )* ])
        }
    };
}

#[macro_export]
macro_rules! oracleinvoc {
    ( $name:expr, $($e:expr),* ) => {
        {
            Expression::OracleInvoc(
                $name.to_string(),
                vec![ $( $e.clone(), )* ],
            )
        }
    };
}

#[macro_export]
macro_rules! fncall {
    ( $name:expr, $($e:expr),* ) => {
        {
            Expression::FnCall(
                Identifier::Scalar($name.to_string()),
                vec![ $( $e.clone(), )* ],
            )
        }
    };
}
