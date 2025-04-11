use crate::identifier::{pkg_ident::PackageIdentifier, Identifier};

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Type {
    Unknown,
    Empty,
    Integer,
    String,
    Boolean,
    Bits(Box<CountSpec>), // Bits strings of length ...
    AddiGroupEl(String),  // name of the group
    MultGroupEl(String),  // name of the group
    List(Box<Type>),
    Set(Box<Type>),
    Tuple(Vec<Type>),
    Table(Box<Type>, Box<Type>),
    Maybe(Box<Type>),
    Fn(Vec<Type>, Box<Type>), // arg types, return type
    UserDefined(String),
}

impl Type {
    pub(crate) fn rewrite_type(&self, rules: &[(Type, Type)]) -> Self {
        if let Some((_, replace)) = rules.iter().find(|(search, _)| self == search) {
            replace.clone()
        } else {
            match self {
                Type::Bits(count_spec) if matches!(count_spec.as_ref(), CountSpec::Identifier(Identifier::PackageIdentifier(PackageIdentifier::Const(pkg_const_ident ))) if &pkg_const_ident.name == "n" && pkg_const_ident.tipe == Type::Integer) => {
                    if rules.is_empty() {
                        print!(".");
                        panic!();
                    } else {
                        //println!("called the rewriter on the target type with rules {rules:#?}");
                    }
                    self.clone()
                }

                Type::Empty
                | Type::Integer
                | Type::String
                | Type::Boolean
                | Type::Bits(_) // NB: This is a fallthrough, the Identifier case is handled above
                | Type::AddiGroupEl(_)
                | Type::MultGroupEl(_) => self.clone(),

                Type::List(t) => Type::List(Box::new(t.rewrite_type(rules))),
                Type::Maybe(t) => Type::Maybe(Box::new(t.rewrite_type(rules))),
                Type::Set(t) => Type::Set(Box::new(t.rewrite_type(rules))),

                Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| t.rewrite_type(rules)).collect()),
                Type::Table(t1, t2) => Type::Table(
                    Box::new(t1.rewrite_type(rules)),
                    Box::new(t2.rewrite_type(rules)),
                ),
                Type::Fn(ts, t) => Type::Fn(
                    ts.iter().map(|t| t.rewrite_type(rules)).collect(),
                    Box::new(t.rewrite_type(rules)),
                ),
                Type::Unknown => unreachable!(),
                Type::UserDefined(_) => unreachable!(),
            }
        }
    }

    pub(crate) fn types_match(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Bits(l), Type::Bits(r)) => dbg!(l.countspecs_match(r.as_ref())),

            (Type::List(l), Type::List(r))
            | (Type::Set(l), Type::Set(r))
            | (Type::Maybe(l), Type::Maybe(r)) => l.types_match(r.as_ref()),

            (Type::Table(lk, lv), Type::Table(rk, rv)) => {
                lk.types_match(rk.as_ref()) && lv.types_match(rv)
            }

            (Type::Tuple(l), Type::Tuple(r)) => {
                l.iter().zip(r.iter()).all(|(l, r)| Type::types_match(l, r))
            }

            (Type::Fn(largs, lty), Type::Fn(rargs, rty)) => {
                largs
                    .iter()
                    .zip(rargs.iter())
                    .all(|(l, r)| Type::types_match(l, r))
                    && lty.types_match(rty.as_ref())
            }

            (lother, rother) => lother == rother,
        }
    }
}

/// Describes the length of Bits types
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum CountSpec {
    Identifier(Identifier),
    Literal(u64),
    Any,
}

impl CountSpec {
    pub(crate) fn countspecs_match(&self, other: &Self) -> bool {
        if let (Self::Identifier(l), Self::Identifier(r)) = (self, other) {
            l.identifiers_match(r)
        } else {
            self == other
        }
    }
}
