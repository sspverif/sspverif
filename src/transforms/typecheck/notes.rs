use std::collections::HashMap;

use crate::writers::smt::exprs::{SmtAnd, SmtExpr, SmtLt, SmtLte, SmtOr};

pub enum Error {
    Undefined(Ident),
    NotANumber(Set),
}

#[derive(Clone, Debug)]
pub enum Comp {
    Lt,
    Lte,
}

#[derive(Clone, Debug)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
}

#[derive(Clone, Debug)]
pub enum Number {
    Numeral(u64),
    GameConst(String),
    PkgConst(String),
    Arith {
        op: ArithOp,
        left: Box<Number>,
        right: Box<Number>,
    },
}

impl From<Number> for SmtExpr {
    fn from(value: Number) -> Self {
        match value {
            Number::Numeral(num) => (num as usize).into(),
            Number::GameConst(name) => format!("game-const-{name}").into(),
            Number::PkgConst(name) => format!("pkg-const-{name}").into(),
            Number::Arith { op, left, right } => {
                let op = match op {
                    ArithOp::Add => "+",
                    ArithOp::Sub => "-",
                    ArithOp::Mul => "*",
                };

                (op, *left, *right).into()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Range {
    pub start: Number,
    pub end: Number,
    pub start_comp: Comp,
    pub end_comp: Comp,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum IdentType {
    Unknown,
    LoopVar,
    PkgConst,
    GameConst,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Ident {
    pub id: String,
    pub tipe: IdentType,
}

#[derive(Clone, Debug)]
pub enum Set {
    Numeral(u64),
    Range(Range),
    Arith {
        op: ArithOp,
        left: Box<Set>,
        right: Box<Set>,
    },
    Ident(Ident),
    Union(Vec<Set>),
}

#[derive(Clone, Debug)]
pub struct Env(pub HashMap<Ident, Set>);

impl Range {
    pub fn contains(&self, num: &Number) -> SmtExpr {
        SmtAnd(vec![
            match self.start_comp {
                Comp::Lt => SmtLt(self.start.clone(), num.clone()).into(),
                Comp::Lte => SmtLte(self.start.clone(), num.clone()).into(),
            },
            match self.start_comp {
                Comp::Lt => SmtLt(num.clone(), self.end.clone()).into(),
                Comp::Lte => SmtLte(num.clone(), self.end.clone()).into(),
            },
        ])
        .into()
    }
}

impl Number {
    pub fn simplify(self, env: &Env) -> Result<Self, Error> {
        match self {
            Number::Numeral(_) | Number::GameConst(_) => Ok(self),
            Number::PkgConst(name) => {
                let ident = Ident {
                    id: name,
                    tipe: IdentType::PkgConst,
                };

                env.0
                    .get(&ident)
                    .ok_or(Error::Undefined(ident))?
                    .clone()
                    .simplify(env)?
                    .try_into_num()
            }
            Number::Arith { op, left, right } => Ok(Self::Arith {
                op,
                left: Box::new(left.simplify(env)?),
                right: Box::new(right.simplify(env)?),
            }),
        }
    }
}

impl Set {
    pub fn simplify(self, env: &Env) -> Result<Self, Error> {
        match self {
            Set::Numeral(_)
            | Set::Ident(Ident {
                tipe: IdentType::GameConst,
                ..
            }) => Ok(self),
            Set::Range(range) => {
                let start = range.start.simplify(env)?;
                let end = range.end.simplify(env)?;
                Ok(Set::Range(Range {
                    start,
                    end,
                    ..range
                }))
            }
            Set::Arith { op, left, right } => Ok(Self::Arith {
                op,
                left: Box::new(left.simplify(env)?),
                right: Box::new(right.simplify(env)?),
            }),
            Set::Ident(ident) => env
                .0
                .get(&ident)
                .ok_or(Error::Undefined(ident))?
                .clone()
                .simplify(env),
            Set::Union(sets) => {
                let simplified: Result<Vec<_>, _> =
                    sets.into_iter().map(|set| set.simplify(env)).collect();
                Ok(Set::Union(simplified?))
            }
        }
    }

    pub fn try_into_num(self) -> Result<Number, Error> {
        match self {
            Set::Numeral(num) => Ok(Number::Numeral(num)),
            Set::Ident(Ident {
                tipe: IdentType::GameConst,
                id,
            }) => Ok(Number::GameConst(id)),
            Set::Range(_) => Err(Error::NotANumber(self)),
            Set::Arith { op, left, right } => {
                let left = Box::new(left.try_into_num()?);
                let right = Box::new(right.try_into_num()?);

                Ok(Number::Arith { op, left, right })
            }
            Set::Ident(Ident {
                tipe: IdentType::PkgConst,
                id,
            }) => Ok(Number::PkgConst(id)),
            Set::Union(_) | Set::Ident(_) => Err(Error::NotANumber(self)),
        }
    }

    pub fn contains(&self, target: &Number) -> SmtExpr {
        match self {
            Set::Numeral(num) => ("=", *num as usize, target.clone()).into(),
            Set::Range(range) => range.contains(target),
            Set::Ident(Ident {
                tipe: IdentType::GameConst,
                id,
            }) => ("=", target.clone(), format!("game-const-{id}")).into(),
            Set::Arith {
                op: ArithOp::Add,
                left,
                right,
            } => {
                if let Ok(left) = left.clone().try_into_num() {
                    right.contains(&Number::Arith {
                        op: ArithOp::Sub,
                        left: Box::new(target.clone()),
                        right: Box::new(left),
                    })
                } else if let Ok(right) = right.clone().try_into_num() {
                    left.contains(&Number::Arith {
                        op: ArithOp::Sub,
                        left: Box::new(target.clone()),
                        right: Box::new(right),
                    })
                } else {
                    unreachable!("one of the arguemnts needs to be a number")
                }
            }
            Set::Arith {
                op: ArithOp::Sub,
                left,
                right,
            } => {
                if let Ok(left) = left.clone().try_into_num() {
                    right.contains(&Number::Arith {
                        op: ArithOp::Sub,
                        left: Box::new(left),
                        right: Box::new(target.clone()),
                    })
                } else if let Ok(right) = right.clone().try_into_num() {
                    left.contains(&Number::Arith {
                        op: ArithOp::Add,
                        left: Box::new(target.clone()),
                        right: Box::new(right),
                    })
                } else {
                    unreachable!("one of the arguemnts needs to be a number")
                }
            }
            Set::Arith {
                op: ArithOp::Mul, ..
            } => {
                todo!()
            }
            Set::Ident(ident) => unreachable!(
                "other identifiers should have been resolved by now. {:?}",
                ident
            ),
            Set::Union(sets) => {
                SmtOr(sets.into_iter().map(|set| set.contains(target)).collect()).into()
            }
        }
    }
}
