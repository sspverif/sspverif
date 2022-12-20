use super::exprs::SmtExpr;

pub fn oracle_function_name(game_name: &str, inst_name: &str, oracle_name: &str) -> String {
    format!("oracle-{game_name}-{inst_name}-{oracle_name}")
}

pub fn return_sort_name(game_name: &str, inst_name: &str, oracle_name: &str) -> String {
    format!("Return_{game_name}_{inst_name}_{oracle_name}")
}

pub fn return_constructor_name(game_name: &str, inst_name: &str, oracle_name: &str) -> String {
    format!("mk-return-{game_name}-{inst_name}-{oracle_name}")
}

pub fn return_selector_state_name(game_name: &str, inst_name: &str, oracle_name: &str) -> String {
    format!("return-{game_name}-{inst_name}-{oracle_name}-state")
}

pub fn return_selector_state_sort(game_name: &str) -> SmtExpr {
    ("Array", "Int", gamestate_sort_name(game_name)).into()
}

pub fn return_selector_state_length_name(
    game_name: &str,
    inst_name: &str,
    oracle_name: &str,
) -> String {
    format!("return-{game_name}-{inst_name}-{oracle_name}-state-length")
}

pub fn return_selector_is_abort_name(
    game_name: &str,
    inst_name: &str,
    oracle_name: &str,
) -> String {
    format!("return-{game_name}-{inst_name}-{oracle_name}-is-abort")
}

pub fn return_selector_value_name(game_name: &str, inst_name: &str, oracle_name: &str) -> String {
    format!("return-{game_name}-{inst_name}-{oracle_name}-value")
}

pub fn gamestate_sort_name(game_name: &str) -> String {
    format!("CompositionState-{game_name}")
}

pub fn gamestate_constructor_name(game_name: &str) -> String {
    format!("mk-composition-state-{game_name}")
}

pub fn gamestate_selector_pkgstate_name(game_name: &str, inst_name: &str) -> String {
    format!("composition-pkgstate-{game_name}-{inst_name}")
}

pub fn gamestate_selector_param_name(game_name: &str, param_name: &str) -> String {
    format!("composition-param-{game_name}-{param_name}")
}

pub fn gamestate_selector_rand_name(game_name: &str, sample_id: usize) -> String {
    format!("composition-rand-{game_name}-{sample_id}")
}

pub fn pkgstate_sort_name(game_name: &str, inst_name: &str) -> String {
    format!("State_{game_name}_{inst_name}")
}

pub fn pkgstate_selector_name(game_name: &str, inst_name: &str, field_name: &str) -> String {
    format!("state-{game_name}-{inst_name}-{field_name}")
}

pub fn pkgstate_constructor_name(game_name: &str, inst_name: &str) -> String {
    format!("mk-state-{game_name}-{inst_name}")
}

pub fn var_selfstate_name() -> String {
    "__self_state".to_string()
}

pub fn var_globalstate_name() -> String {
    "__global_state".to_string()
}

pub fn var_state_length_name() -> String {
    "__state_length".to_string()
}

pub fn var_ret_name() -> String {
    "__ret".to_string()
}

pub fn fn_sample_rand_name<S: Into<SmtExpr>>(game_name: &str, rand_sort: S) -> String {
    format!("__sample-rand-{}-{}", game_name, rand_sort.into())
}
