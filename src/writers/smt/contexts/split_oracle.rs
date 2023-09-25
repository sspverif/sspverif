use crate::{
    split::{SplitOracleDef, SplitPath},
    writers::smt::{
        exprs::SmtExpr, names, partials::PartialsDatatype, patterns::IntermediateStatePattern,
    },
};

use super::{GameContext, GenericOracleContext, PackageInstanceContext, SplitOracleContext};

impl<'a> SplitOracleContext<'a> {
    pub fn smt_arg_name(&self, arg_name: &str) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let odef = &inst.pkg.oracles[self.split_oracle_offs];

        names::oracle_split_arg_name(&game.name, &odef.sig.name, arg_name).into()
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

    fn game_ctx(&self) -> GameContext<'a> {
        self.game_ctx.clone()
    }

    fn pkg_inst_ctx(&self) -> PackageInstanceContext<'a> {
        PackageInstanceContext {
            game_ctx: self.game_ctx.clone(),
            inst_offs: self.inst_offs,
        }
    }

    pub fn intermediate_state_pattern(&self) -> IntermediateStatePattern<'a> {
        let pkg_inst_ctx = self.pkg_inst_ctx();
        let game_ctx = self.game_ctx();

        IntermediateStatePattern {
            game_name: &game_ctx.game.name,
            pkg_inst_name: pkg_inst_ctx.pkg_inst_name(),
            oracle_name: &self.oracle_def().sig.name,
        }
    }
}

impl<'a> GenericOracleContext for SplitOracleContext<'a> {
    fn game_ctx(&self) -> GameContext<'a> {
        self.game_ctx()
    }

    fn pkg_inst_ctx(&self) -> PackageInstanceContext<'a> {
        self.pkg_inst_ctx()
    }

    fn oracle_name(&self) -> &str {
        &self.oracle_def().sig.name
    }

    fn oracle_return_type(&self) -> &crate::types::Type {
        &self.oracle_def().sig.tipe
    }

    fn smt_game_state(&self) -> SmtExpr {
        "__global_state".into()
    }

    fn smt_construct_abort(&self) -> SmtExpr {
        let game = self.game_ctx.game();
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
