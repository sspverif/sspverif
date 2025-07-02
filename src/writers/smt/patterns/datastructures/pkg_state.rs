use crate::{
    expressions::Expression,
    identifier::pkg_ident::PackageConstIdentifier,
    package::Package,
    types::Type,
    writers::smt::{
        contexts::PackageInstanceContext,
        names,
        patterns::instance_names::{encode_params, only_ints, Separated},
    },
};

use super::{DatastructurePattern, DatastructureSpec, Datatype};

pub struct PackageStatePattern<'a> {
    pub pkg_name: &'a str,
    pub params: &'a [(PackageConstIdentifier, Expression)],
}

#[derive(PartialEq, Eq, Debug)]
pub struct PackageStateSelector<'a> {
    pub name: &'a str,
    pub ty: &'a Type,
}

pub(crate) struct PackageStateDatatype<'a>(PackageInstanceContext<'a>);

impl<'a> PackageInstanceContext<'a> {
    pub(crate) fn package_state_datatype(self) -> PackageStateDatatype<'a> {
        PackageStateDatatype(self)
    }
}

impl Datatype for PackageStateDatatype<'_> {
    type Constructor = ();

    type Selector = usize;

    const CAMEL_CASE: &'static str = "PackageState";

    const KEBAB_CASE: &'static str = "pkg-state";

    fn sort_symbol(&self) -> sspverif_smtlib::syntax::tokens::Symbol {
        let base_elems: &[&str] = &[Self::CAMEL_CASE, self.0.pkg_name()];
        let encoded_params = encode_params(only_ints(self.0.pkg_params()));
        let encoded_params_ref = encoded_params.as_ref().map(String::as_str);

        names::concat_camel_case(base_elems.iter().copied().chain(encoded_params_ref)).into()
    }

    fn sort_par_sort_symbols(&self) -> Vec<sspverif_smtlib::syntax::tokens::Symbol> {
        vec![]
    }

    fn constructors(&self) -> impl Iterator<Item = Self::Constructor> {
        Some(()).into_iter()
    }

    fn selectors(&self, _con: &Self::Constructor) -> impl Iterator<Item = Self::Selector> {
        0..self.0.pkg().state.len()
    }

    fn constructor_symbol(
        &self,
        _cons: &Self::Constructor,
    ) -> sspverif_smtlib::syntax::tokens::Symbol {
        let base_elems = &["mk", Self::KEBAB_CASE, self.0.pkg_name()];
        let encoded_params = encode_params(only_ints(self.0.pkg_params()));
        let encoded_params_ref = encoded_params.as_ref().map(String::as_str);

        names::concat_kebab_case(base_elems.iter().copied().chain(encoded_params_ref)).into()
    }

    fn selector_symbol(&self, sel: &Self::Selector) -> sspverif_smtlib::syntax::tokens::Symbol {
        let base_elems = &[Self::KEBAB_CASE, self.0.pkg_name()];
        let encoded_params = encode_params(only_ints(self.0.pkg_params()));
        let encoded_params_ref = encoded_params.as_ref().map(String::as_str);
        let (param_name, _, _) = &self.0.pkg().state[*sel];

        names::concat_kebab_case(
            base_elems
                .iter()
                .copied()
                .chain(encoded_params_ref)
                .chain(Some(param_name.as_str())),
        )
        .into()
    }

    fn selector_sort(&self, sel: &Self::Selector) -> sspverif_smtlib::syntax::sort::Sort {
        let (_, ty, _) = &self.0.pkg().state[*sel];
        ty.into()
    }
}

impl<'a> DatastructurePattern<'a> for PackageStatePattern<'a> {
    type Constructor = ();
    type Selector = PackageStateSelector<'a>;
    type DeclareInfo = Package;

    const CAMEL_CASE: &'static str = "PackageState";

    const KEBAB_CASE: &'static str = "pkg-state";

    fn sort_name(&self) -> String {
        let camel_case = Self::CAMEL_CASE;
        let Self { pkg_name, params } = self;

        let encoded_params = encode_params(only_ints(*params));
        let separated_params = Separated::new(encoded_params, "_");

        format!("<{camel_case}_{pkg_name}{separated_params}>")
    }

    fn constructor_name(&self, _cons: &Self::Constructor) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self { pkg_name, params } = self;
        let encoded_params = encode_params(only_ints(*params));
        let separated_params = Separated::new(encoded_params, "-");

        format!("<mk-{kebab_case}-{pkg_name}{separated_params}>")
    }

    fn selector_name(&self, sel: &Self::Selector) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self { pkg_name, params } = self;
        let encoded_params = encode_params(only_ints(*params));
        let separated_params = Separated::new(encoded_params, "-");

        let PackageStateSelector {
            name: field_name, ..
        } = sel;

        format!("<{kebab_case}-{pkg_name}{separated_params}-{field_name}>")
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
