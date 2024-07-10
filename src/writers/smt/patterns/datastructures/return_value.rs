use crate::{types::Type, writers::smt::sorts::SmtReturnValue};

use super::{DatastructurePattern, DatastructureSpec};

#[derive(PartialEq, Eq)]
pub enum ReturnValueConstructor {
    Return,
    Abort,
}

#[derive(PartialEq, Eq)]
pub struct ReturnValueSelector;

#[derive(PartialEq, Eq)]
pub struct ReturnValue<'a> {
    pub inner_type: &'a Type,
}

impl<'a> ReturnValue<'a> {
    pub fn new(inner_type: &'a Type) -> Self {
        Self { inner_type }
    }
}

impl<'a> DatastructurePattern<'a> for ReturnValue<'a> {
    type Sort = SmtReturnValue<Type>;

    type Constructor = ReturnValueConstructor;

    type Selector = ReturnValueSelector;

    type DeclareInfo = ();

    const CAMEL_CASE: &'static str = "ReturnValue";

    const KEBAB_CASE: &'static str = "return-value";

    fn sort(&self) -> Self::Sort {
        SmtReturnValue {
            inner_sort: self.inner_type.clone(),
        }
    }

    fn constructor_name(&self, cons: &Self::Constructor) -> String {
        match cons {
            ReturnValueConstructor::Return => "mk-return-value",
            ReturnValueConstructor::Abort => "mk-abort",
        }
        .to_string()
    }

    fn selector_name(&self, _sel: &Self::Selector) -> String {
        "return-value".to_string()
    }

    fn selector_sort(&self, _sel: &Self::Selector) -> crate::writers::smt::exprs::SmtExpr {
        self.inner_type.into()
    }

    fn datastructure_spec(
        &self,
        _info: &'a Self::DeclareInfo,
    ) -> super::DatastructureSpec<'a, Self> {
        DatastructureSpec(vec![
            (ReturnValueConstructor::Return, vec![ReturnValueSelector]),
            (ReturnValueConstructor::Abort, vec![]),
        ])
    }

    fn matchfield_name(&self, _sel: &Self::Selector) -> String {
        "returnvalue".to_string()
    }
}
