use crate::{
    split::{SplitOracleDef, SplitPath},
    types::Type,
    writers::smt::{
        exprs::SmtExpr,
        names,
        partials::PartialsDatatype,
        patterns::{oracle_args::{self, OracleArgPattern}, FunctionPattern, IntermediateStatePattern, PartialOraclePattern},
    },
};

use super::{
    GameInstanceContext, GenericOracleContext, PackageInstanceContext, SplitOracleContext,
};

impl<'a> SplitOracleContext<'a> {
    pub fn smt_arg_name(&self, arg_name: &str) -> SmtExpr {
        let game_inst = self.game_inst_ctx().game_inst();
        let game = self.game_inst_ctx().game();
        let inst = &game.pkgs[self.pkg_inst_offs];
        let odef = &inst.pkg.oracles[self.split_oracle_offs];

        names::oracle_split_arg_name(game_inst.name(), &odef.sig.name, arg_name).into()
    }

    pub fn oracle_def(&self) -> &'a SplitOracleDef {
        &self.pkg_inst_ctx().pkg_inst().pkg.split_oracles[self.split_oracle_offs]
    }

    pub fn partials_dtype(&self) -> &'a PartialsDatatype {
        self.partials
    }

    pub fn split_path(&self) -> &'a SplitPath {
        &self.oracle_def().sig.path
    }

    fn game_inst_ctx(&self) -> GameInstanceContext<'a> {
        self.game_inst_context.clone()
    }

    fn pkg_inst_ctx(&self) -> PackageInstanceContext<'a> {
        PackageInstanceContext {
            game_ctx: self.game_inst_ctx(),
            inst_offs: self.pkg_inst_offs,
        }
    }

    pub fn intermediate_state_pattern(&self) -> IntermediateStatePattern<'a> {
        let pkg_inst_ctx = self.pkg_inst_ctx();

        IntermediateStatePattern {
            pkg_name: pkg_inst_ctx.pkg_inst_name(),
            params: &pkg_inst_ctx.pkg_inst().params,
            oracle_name: &self.oracle_def().sig.name,
        }
    }

    // returns none if the wrong number of arguments were provided
    pub fn smt_invoke_oracle<GS, IS, ARGS>(
        &self,
        gamestate: GS,
        intermediate_state: IS,
        args: ARGS,
    ) -> Option<SmtExpr>
    where
        GS: Into<SmtExpr>,
        IS: Into<SmtExpr>,
        ARGS: Iterator<Item = SmtExpr>,
    {
        let game_inst = self.game_inst_ctx().game_inst();
        let game = self.game_inst_ctx().game();
        let pkg_inst = &game.pkgs[self.pkg_inst_offs];
        let osig = &pkg_inst.pkg.split_oracles[self.split_oracle_offs].sig;

        let game_name = game_inst.game_name();
        let game_params = &game_inst.consts;
        let pkg_name = &pkg_inst.pkg.name;
        let pkg_params = &pkg_inst.params;
        let oracle_name = &osig.name;
        let split_path = &osig.path;

        let pattern = PartialOraclePattern {
            game_name,
            game_params,
            pkg_name,
            pkg_params,
            oracle_name,
            split_path,
        };

        let expected_len = 3 + osig.args.len();

        let mut cmdline = Vec::with_capacity(expected_len);
        cmdline.push(pattern.function_name().into());
        cmdline.push(gamestate.into());
        cmdline.push(intermediate_state.into());
        cmdline.extend(args);

        if cmdline.len() != expected_len {
            return None;
        }

        Some(SmtExpr::List(cmdline))
    }
}

impl<'a> GenericOracleContext for SplitOracleContext<'a> {
    fn game_inst_ctx(&self) -> GameInstanceContext {
        self.game_inst_ctx()
    }

    fn pkg_inst_ctx(&self) -> PackageInstanceContext<'a> {
        self.pkg_inst_ctx()
    }

    fn oracle_name(&self) -> &str {
        &self.oracle_def().sig.name
    }

    fn oracle_args(&self) -> &[(String, Type)] {
        &self.oracle_def().sig.args
    }

    fn oracle_return_type(&self) -> &crate::types::Type {
        &self.oracle_def().sig.tipe
    }

    fn smt_game_state(&self) -> SmtExpr {
        "__global_state".into()
    }

    fn smt_write_back_state(
        &self,
        sample_info: &crate::transforms::samplify::SampleInfo,
    ) -> SmtExpr {
        let game_inst_ctx = self.game_inst_ctx();
        let pkg_inst_ctx = self.pkg_inst_ctx();
        let pkg_inst = self.pkg_inst_ctx().pkg_inst();

        let game_state = oracle_args::GameStatePattern {
            game_name: game_inst_ctx.game_name(),
            game_params: game_inst_ctx.game_params(),
        };

        game_inst_ctx
            .smt_update_gamestate_pkgstate(
                game_state.local_arg_name(),
                sample_info,
                &pkg_inst.name,
                pkg_inst_ctx.smt_update_pkgstate_from_locals().unwrap(),
            )
            .unwrap()
    }

    fn smt_construct_abort<S: Into<SmtExpr>>(&self, _game_state: S) -> SmtExpr {
        let game = self.game_inst_context.game();
        let game_name = &game.name;
        let inst_name = &self.pkg_inst_ctx().pkg_inst().name;
        let oracle_name_with_path = self.oracle_def().sig.name_with_path();

        SmtExpr::List(vec![names::return_constructor_abort_name(
            game_name,
            inst_name,
            &oracle_name_with_path,
        )
        .into()])
    }
}
