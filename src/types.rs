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
    pub fn new_bits(length: &str) -> Type {
        Type::Bits(length.to_string())
    }

    #[allow(dead_code)]
    pub fn new_list(t: &Type) -> Type {
        Type::List(Box::new(t.clone()))
    }

    #[allow(dead_code)]
    pub fn new_set(t: &Type) -> Type {
        Type::Set(Box::new(t.clone()))
    }

    pub fn new_fn(args: Vec<Type>, ret: Type) -> Type {
        Type::Fn(args, Box::new(ret))
    }

    pub fn default_value(&self) -> Expression {
        match self {
            Type::Integer => Expression::IntegerLiteral(0),
            Type::String => Expression::StringLiteral("".to_string()),
            Type::Boolean => Expression::BooleanLiteral("false".to_string()),
            Type::List(_tipe) => Expression::List(vec![]),
            Type::Tuple(tipes) => {
                Expression::Tuple(tipes.iter().map(|tipe| tipe.default_value()).collect())
            }
            Type::Table(_, _) => Expression::EmptyTable(self.clone()),
            Type::Maybe(tipe) => Expression::None(*tipe.clone()),
            Type::Empty | Type::Fn(_, _) => {
                panic!("No default value for type {:?}", self)
            }
            _ => todo!("No default value for type {:?}", self),
        }
    }

    pub(crate) fn rewrite(&self, rules: &[(Type, Type)]) -> Self {
        match self {
            Type::UserDefined(_) => {
                if let Some((_, replace)) = rules.iter().find(|(search, _)| self == search) {
                    replace.clone()
                } else {
                    self.clone()
                }
            }

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
        }
    }
}
