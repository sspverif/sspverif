use crate::{
    expressions::Expression,
    identifier::pkg_ident::PackageConstIdentifier,
    package::Package,
    types::Type,
    writers::smt::{
        patterns::instance_names::{encode_params, only_non_function_expression},
        sorts::SmtPlainSort,
    },
};

use super::{DatastructurePattern, DatastructureSpec};

pub struct PackageStatePattern<'a> {
    pub pkg_name: &'a str,
    pub params: &'a [(PackageConstIdentifier, Expression)],
}

impl<'a> PackageStatePattern<'a> {
    fn name(&self) -> String {
        let pkg_name = &self.pkg_name;

        if self.params.is_empty() {
            pkg_name.to_string()
        } else {
            let encoded_params = encode_params(only_non_function_expression(self.params));
            format!("{pkg_name}-{encoded_params}")
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct PackageStateSelector<'a> {
    pub name: &'a str,
    pub ty: &'a Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PackageStateSort<'a> {
    pub pkg_name: &'a str,
    pub params: &'a [(PackageConstIdentifier, Expression)],
}

use crate::impl_Into_for_PlainSort;
impl_Into_for_PlainSort!('a, PackageStateSort<'a>);

impl<'a> SmtPlainSort for PackageStateSort<'a> {
    fn sort_name(&self) -> String {
        let camel_case = PackageStatePattern::CAMEL_CASE;
        let Self { pkg_name, params } = self;

        let encoded_params = encode_params(only_non_function_expression(*params));

        format!("<{camel_case}_{pkg_name}_{encoded_params}>")
    }
}

impl<'a> DatastructurePattern<'a> for PackageStatePattern<'a> {
    type Constructor = ();
    type Selector = PackageStateSelector<'a>;
    type DeclareInfo = Package;
    type Sort = PackageStateSort<'a>;

    const CAMEL_CASE: &'static str = "State";

    const KEBAB_CASE: &'static str = "state";

    fn sort(&self) -> PackageStateSort<'a> {
        let PackageStatePattern { pkg_name, params } = self;
        PackageStateSort { pkg_name, params }
    }

    fn constructor_name(&self, _cons: &Self::Constructor) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self { pkg_name, params } = self;
        let encoded_params = encode_params(only_non_function_expression(*params));

        format!("<mk-{kebab_case}-{pkg_name}-{encoded_params}>")
    }

    fn selector_name(&self, sel: &Self::Selector) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self { pkg_name, params } = self;
        let encoded_params = encode_params(only_non_function_expression(*params));

        let PackageStateSelector {
            name: field_name, ..
        } = sel;

        format!("<{kebab_case}-{pkg_name}-{encoded_params}-{field_name}>")
    }

    fn matchfield_name(&self, sel: &Self::Selector) -> String {
        let PackageStateSelector {
            name: field_name, ..
        } = sel;

        format!("<match-{field_name}>")
    }

    fn selector_sort(&self, sel: &Self::Selector) -> crate::writers::smt::exprs::SmtExpr {
        let PackageStateSelector { ty: tipe, .. } = sel;
        (*tipe).into()
    }

    fn datastructure_spec(&self, pkg: &'a Self::DeclareInfo) -> DatastructureSpec<'a, Self> {
        let selectors = pkg
            .state
            .iter()
            .map(|(name, tipe, _file_pos)| PackageStateSelector { name, ty: tipe });

        DatastructureSpec(vec![((), selectors.collect())])
    }
}
