use crate::{
    package::Package,
    types::Type,
    writers::smt::patterns::{DatastructurePattern, DatastructureSpec},
};

#[derive(Debug)]
pub struct PackageConstsPattern<'a> {
    pub pkg_name: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PackageConstsSelector<'a> {
    pub(crate) name: &'a str,
    pub(crate) ty: &'a Type,
}

impl<'a> DatastructurePattern<'a> for PackageConstsPattern<'a> {
    type Constructor = ();

    type Selector = PackageConstsSelector<'a>;

    type DeclareInfo = Package;

    const CAMEL_CASE: &'static str = "PackageConsts";

    const KEBAB_CASE: &'static str = "pkg-consts";

    fn sort_name(&self) -> String {
        let camel_case = Self::CAMEL_CASE;
        let pkg_name = self.pkg_name;
        format!("<{camel_case}_{pkg_name}>")
    }

    fn constructor_name(&self, _cons: &Self::Constructor) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self { pkg_name } = self;

        format!("<mk-{kebab_case}-{pkg_name}>")
    }

    fn selector_name(&self, sel: &Self::Selector) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let const_name = sel.name;
        let Self { pkg_name } = self;

        format!("<{kebab_case}-{pkg_name}-{const_name}>")
    }

    fn selector_sort(&self, sel: &Self::Selector) -> crate::writers::smt::exprs::SmtExpr {
        sel.ty.into()
    }

    fn datastructure_spec(&self, info: &'a Self::DeclareInfo) -> DatastructureSpec<'a, Self> {
        let fields = info
            .params
            .iter()
            // function parameters are just declared as smtlib functions globally, so we don't
            // want them to be part of this datatype. This way we also stay compatible with
            // solvers that don't support higher-order functions.
            .filter(|(_name, ty, _)| !matches!(ty, crate::types::Type::Fn(_, _)))
            .map(|(name, ty, _)| PackageConstsSelector { name, ty })
            .collect();

        DatastructureSpec(vec![((), fields)])
    }

    fn matchfield_name(&self, sel: &Self::Selector) -> String {
        let const_name = sel.name;
        format!("<match-{const_name}>")
    }
}
