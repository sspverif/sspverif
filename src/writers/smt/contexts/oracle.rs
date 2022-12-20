use crate::expressions::Expression;

use super::super::exprs::SmtExpr;
use super::super::{declare, names};

use super::{OracleContext, PackageInstanceContext};

impl<'a> OracleContext<'a> {
    pub fn pkg_inst_ctx(&self) -> PackageInstanceContext {
        PackageInstanceContext {
            game_ctx: self.game_ctx.clone(),
            inst_offs: self.inst_offs,
        }
    }
    pub fn smt_sort_return(&self) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        names::return_sort_name(game_name, inst_name, oracle_name).into()
    }

    pub fn smt_declare_return(&self) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        use crate::types::Type;

        let fields = vec![
            (
                names::return_selector_state_name(game_name, inst_name, oracle_name),
                self.game_ctx.smt_sort_gamestates(),
            ),
            (
                names::return_selector_state_length_name(game_name, inst_name, oracle_name),
                Type::Integer.into(),
            ),
            (
                names::return_selector_value_name(game_name, inst_name, oracle_name),
                Type::Maybe(Box::new(osig.tipe.clone())).into(),
            ),
            (
                names::return_selector_is_abort_name(game_name, inst_name, oracle_name),
                Type::Boolean.into(),
            ),
        ];

        declare::declare_datatype(
            &names::return_sort_name(game_name, inst_name, oracle_name),
            &names::return_constructor_name(game_name, inst_name, oracle_name),
            fields.into_iter(),
        )
    }

    pub fn smt_construct_return<S, SL, V, ISAB>(
        &self,
        state: S,
        state_len: SL,
        value: V,
        is_abort: ISAB,
    ) -> SmtExpr
    where
        S: Into<SmtExpr>,
        SL: Into<SmtExpr>,
        V: Into<SmtExpr>,
        ISAB: Into<SmtExpr>,
    {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_constructor_name(game_name, inst_name, oracle_name),
            state,
            state_len,
            value,
            is_abort,
        )
            .into()
    }

    pub fn smt_construct_abort<S, SL>(&self, state: S, state_len: SL) -> SmtExpr
    where
        S: Into<SmtExpr>,
        SL: Into<SmtExpr>,
    {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_constructor_name(game_name, inst_name, oracle_name),
            state,
            state_len,
            Expression::None(osig.tipe.clone()),
            "true",
        )
            .into()
    }

    pub fn smt_access_return_state<R>(&self, ret: R) -> SmtExpr
    where
        R: Into<SmtExpr>,
    {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_selector_state_name(game_name, inst_name, oracle_name),
            ret,
        )
            .into()
    }

    pub fn smt_select_return_state<R, L>(&self, ret: R, state_length: L) -> SmtExpr
    where
        R: Into<SmtExpr>,
        L: Into<SmtExpr>,
    {
        ("select", self.smt_access_return_state(ret), state_length).into()
    }

    pub fn smt_store_return_state<R: Into<SmtExpr>, L: Into<SmtExpr>, S: Into<SmtExpr>>(
        &self,
        ret: R,
        state_length: L,
        state: S,
    ) -> SmtExpr {
        (
            "store",
            self.smt_access_return_state(ret),
            state_length,
            state,
        )
            .into()
    }

    pub fn smt_access_return_state_length<R: Into<SmtExpr>>(&self, ret: R) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_selector_state_length_name(game_name, inst_name, oracle_name),
            ret,
        )
            .into()
    }

    pub fn smt_access_return_is_abort<R: Into<SmtExpr>>(&self, ret: R) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_selector_is_abort_name(game_name, inst_name, oracle_name),
            ret,
        )
            .into()
    }

    pub fn smt_access_return_value<R: Into<SmtExpr>>(&self, ret: R) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        (
            names::return_selector_value_name(game_name, inst_name, oracle_name),
            ret,
        )
            .into()
    }

    // returns none if the wrong number of arguments were provided
    pub fn smt_invoke_oracle<S, SLEN, ARGS>(
        &self,
        gamestates: S,
        state_length: SLEN,
        args: ARGS,
    ) -> Option<SmtExpr>
    where
        S: Into<SmtExpr>,
        SLEN: Into<SmtExpr>,
        ARGS: Iterator<Item = SmtExpr>,
    {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        let expected_len = 3 + osig.args.len();

        let mut cmdline = Vec::with_capacity(expected_len);
        cmdline.push(names::oracle_function_name(game_name, inst_name, oracle_name).into());
        cmdline.push(gamestates.into());
        cmdline.push(state_length.into());
        cmdline.extend(args);

        if cmdline.len() != expected_len {
            return None;
        }

        Some(SmtExpr::List(cmdline))
    }
}
