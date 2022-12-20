use crate::{
    package::Composition,
    writers::smt::{exprs::SmtExpr, names},
};

pub struct GlobalContext;

impl GlobalContext {
    pub fn smt_latest_gamestate() -> SmtExpr {
        (
            "select",
            names::var_globalstate_name(),
            names::var_state_length_name(),
        )
            .into()
    }
}

mod game;
mod oracle;
mod pkg_inst;

#[derive(Clone)]
pub struct GameContext<'a> {
    game: &'a Composition,
}

pub struct OracleContext<'a> {
    game_ctx: GameContext<'a>,
    inst_offs: usize,
    oracle_offs: usize,
}

pub struct PackageInstanceContext<'a> {
    game_ctx: GameContext<'a>,
    inst_offs: usize,
}
