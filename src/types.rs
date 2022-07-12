use crate::expressions::Expression;

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Type {
    Empty,
    Integer,
    String,
    Boolean,
    Bits(String), // Bits strings of length ...
    AddiGroupEl(String), // name of the group
    MultGroupEl(String), // name of the group
    List(Box<Type>),
    Set(Box<Type>),
    Tuple(Vec<Type>),
    Table(Box<Type>, Box<Type>),
    Maybe(Box<Type>),
    Fn(Vec<Type>, Box<Type>),     // arg types, return type
    Oracle(Vec<Type>, Box<Type>), // arg types, return type
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
            Type::Integer => Expression::IntegerLiteral("0".to_string()),
            Type::String => Expression::StringLiteral("".to_string()),
            Type::Boolean => Expression::BooleanLiteral("false".to_string()),
            Type::List(tipe) => Expression::List(vec![]),
            Type::Tuple(tipes) => Expression::Tuple(tipes.iter().map(|tipe| tipe.default_value()).collect()),
            Type::Table(_,_) => Expression::EmptyTable,
            Type::Maybe(tipe) => Expression::None(*tipe.clone()),
            Type::Empty |
            Type::Fn(_,_) |
            Type::Oracle(_,_) => panic!("No default value for type {:?}", self),
            _ => todo!("No default value for type {:?}", self),
        }
    }
}
