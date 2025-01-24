use crate::expressions::Expression;

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Type {
    Unknown,
    Empty,
    Integer,
    String,
    Boolean,
    Bits(String),        // Bits strings of length ...
    AddiGroupEl(String), // name of the group
    MultGroupEl(String), // name of the group
    List(Box<Type>),
    Set(Box<Type>),
    Tuple(Vec<Type>),
    Table(Box<Type>, Box<Type>),
    Maybe(Box<Type>),
    Fn(Vec<Type>, Box<Type>), // arg types, return type
    UserDefined(String),
}

impl Type {
    pub(crate) fn rewrite(&self, rules: &[(Type, Type)]) -> Self {
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

                Type::List(t) => Type::List(Box::new(t.rewrite(rules))),
                Type::Maybe(t) => Type::Maybe(Box::new(t.rewrite(rules))),
                Type::Set(t) => Type::Set(Box::new(t.rewrite(rules))),

                Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| t.rewrite(rules)).collect()),
                Type::Table(t1, t2) => {
                    Type::Table(Box::new(t1.rewrite(rules)), Box::new(t2.rewrite(rules)))
                }
                Type::Fn(ts, t) => Type::Fn(
                    ts.iter().map(|t| t.rewrite(rules)).collect(),
                    Box::new(t.rewrite(rules)),
                ),
                Type::Unknown => unreachable!(),
                Type::UserDefined(_) => unreachable!(),
            }
        }
    }
}
