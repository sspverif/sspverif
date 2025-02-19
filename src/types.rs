use crate::identifier::Identifier;

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
                Type::Empty
                | Type::Integer
                | Type::String
                | Type::Boolean
                | Type::Bits(_)
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
}

/// Describes the length of Bits types
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum CountSpec {
    Identifier(Identifier),
    Literal(u64),
    Any,
}
