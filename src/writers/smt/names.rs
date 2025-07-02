use itertools::Itertools;

use super::exprs::SmtExpr;

pub(crate) fn concat_camel_case<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    format!("<{}>", parts.into_iter().join("_"))
}

pub(crate) fn concat_kebab_case<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    format!("<{}>", parts.into_iter().join("-"))
}

pub(crate) fn var_globalstate_name() -> String {
    "<game-state>".to_string()
}

pub(crate) fn var_ret_name() -> String {
    "__ret".to_string()
}

pub(crate) fn fn_sample_rand_name<S: Into<SmtExpr>>(game_name: &str, rand_sort: S) -> String {
    format!("__sample-rand-{}-{}", game_name, rand_sort.into())
}

pub(crate) fn oracle_nonsplit_arg_name(oracle_name: &str, arg_name: &str) -> String {
    format!("<arg-{oracle_name}-{arg_name}>")
}
