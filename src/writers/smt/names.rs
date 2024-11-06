use super::exprs::SmtExpr;

pub(crate) fn oracle_function_name(game_name: &str, inst_name: &str, oracle_name: &str) -> String {
    format!("oracle-{game_name}-{inst_name}-{oracle_name}")
}

pub(crate) fn return_constructor_abort_name(
    game_name: &str,
    inst_name: &str,
    oracle_name: &str,
) -> String {
    format!("mk-abort-{game_name}-{inst_name}-{oracle_name}")
}

pub(crate) fn gamestate_sort_name(game_name: &str) -> String {
    format!("CompositionState-{game_name}")
}

pub(crate) fn var_selfstate_name() -> String {
    "__self_state".to_string()
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

pub(crate) fn oracle_split_arg_name(game_name: &str, oracle_name: &str, arg_name: &str) -> String {
    format!("<arg-{game_name}-{oracle_name}-{arg_name}>")
}
