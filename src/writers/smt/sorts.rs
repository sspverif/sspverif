use crate::types::Type;

use super::exprs::SmtExpr;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Sort {
    Int,
    Bool,
    String,
    Array(Box<(Self, Self)>),
    Parameter(String),
    Other(String, Vec<Self>), /* TODO: sheould we put the datatype spec here? */
}

impl From<Type> for Sort {
    fn from(value: Type) -> Self {
        match value {
            Type::Bits(length) => {
                // TODO make sure we define this somewhere
                Sort::Other(format!("Bits_{}", length), vec![])
            }
            Type::Maybe(t) => Sort::Other("Maybe".to_string(), vec![(*t).into()]),
            Type::Boolean => Sort::Bool,
            Type::Empty => Sort::Other("Empty".to_string(), vec![]),
            Type::Integer => Sort::Int,
            Type::Table(t_idx, t_val) => {
                Sort::Array(Box::new(((*t_idx).into(), Type::Maybe(t_val).into())))
            }
            Type::Tuple(types) => Sort::Other(
                format!("Tuple{}", types.len()),
                types.into_iter().map(|ty| ty.into()).collect(),
            ),
            Type::Unknown => todo!(),
            Type::String => todo!(),
            Type::AddiGroupEl(_) => todo!(),
            Type::MultGroupEl(_) => todo!(),
            Type::List(_) => todo!(),
            Type::Set(_) => todo!(),
            Type::Fn(_, _) => todo!(),
            Type::UserDefined(_) => todo!(),
        }
    }
}

impl From<Sort> for SmtExpr {
    fn from(value: Sort) -> Self {
        match value {
            Sort::Int => "Int".into(),
            Sort::Bool => "Bool".into(),
            Sort::String => "String".into(),
            Sort::Array(types) => {
                let (k, v) = *types;
                ("Array", k, v).into()
            }
            Sort::Other(name, params) => {
                if params.is_empty() {
                    name.into()
                } else {
                    SmtExpr::List(
                        Some(name)
                            .into_iter()
                            .map(SmtExpr::Atom)
                            .chain(params.into_iter().map(|sort| sort.into()))
                            .collect(),
                    )
                }
            }
            Sort::Parameter(name) => name.into(),
        }
    }
}
