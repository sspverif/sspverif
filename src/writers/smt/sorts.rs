use crate::types::Type;

use super::exprs::SmtExpr;

pub trait SmtSort: Into<SmtExpr> {}

impl SmtSort for Type {}

pub trait SmtPlainSort: SmtSort {
    fn sort_name(&self) -> String;
}

#[macro_export]
macro_rules! impl_Into_for_PlainSort {
    ($a:lifetime, $plain_sort:ty) => {
        impl<$a> From<$plain_sort> for $crate::writers::smt::exprs::SmtExpr {
            fn from(value: $plain_sort) -> $crate::writers::smt::exprs::SmtExpr {
                value.sort_name().into()
            }
        }
        impl<$a> $crate::writers::smt::sorts::SmtSort for $plain_sort {}
    };
    ($plain_sort:ty) => {
        impl From<$plain_sort> for $crate::writers::smt::exprs::SmtExpr {
            fn from(value: $plain_sort) -> $crate::writers::smt::exprs::SmtExpr {
                value.sort_name().into()
            }
        }
        impl<$a> $crate::writers::smt::sorts::SmtSort for $plain_sort {}
    };
}

pub struct Array<K, V>
where
    K: Into<SmtExpr>,
    V: Into<SmtExpr>,
{
    pub key: K,
    pub value: V,
}

impl<K, V> From<Array<K, V>> for SmtExpr
where
    K: Into<SmtExpr>,
    V: Into<SmtExpr>,
{
    fn from(t: Array<K, V>) -> Self {
        let Array { key, value } = t;
        ("Array", key, value).into()
    }
}

pub struct SmtReturnValue<T: SmtSort> {
    pub inner_sort: T,
}

impl<T: SmtSort> From<SmtReturnValue<T>> for SmtExpr {
    fn from(value: SmtReturnValue<T>) -> Self {
        ("ReturnValue", value.inner_sort).into()
    }
}

impl<T: SmtSort> SmtSort for SmtReturnValue<T> {}

pub struct SmtBool;

impl From<SmtBool> for SmtExpr {
    fn from(_value: SmtBool) -> Self {
        SmtExpr::Atom("Bool".to_string())
    }
}

impl SmtSort for SmtBool {}

pub struct SmtInt;

impl From<SmtInt> for SmtExpr {
    fn from(_: SmtInt) -> Self {
        SmtExpr::Atom("Int".to_string())
    }
}

impl SmtSort for SmtInt {}
