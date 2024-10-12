use crate::package::OracleDef;
use crate::transforms::samplify::SampleInfo;
use crate::types::Type;
use crate::writers::smt::patterns::oracle_args::OracleArgPattern;
use crate::writers::smt::patterns::FunctionPattern;
use crate::writers::smt::patterns::{
    declare_datatype, oracle_args, DatastructurePattern, DispatchOraclePattern, OraclePattern,
    ReturnConstructor, ReturnPattern, ReturnSelector, ReturnValue, ReturnValueConstructor,
};

use super::super::exprs::SmtExpr;
use super::super::names;

use super::{GameInstanceContext, GenericOracleContext, OracleContext, PackageInstanceContext};

// Patterns
impl<'a> OracleContext<'a> {
    pub(crate) fn oracle_pattern(&'a self) -> OraclePattern<'a> {
        let game_inst = &self.game_inst_context.game_inst;
        let pkg_inst = &game_inst.game().pkgs[self.pkg_inst_offs];

        let game_name = self.game_inst_context.game_inst().game_name();
        let pkg_name = self.pkg_inst_ctx().pkg_name();
        let oracle_name = self.oracle_name();
        let oracle_args = self.oracle_args();

        let game_params = &game_inst.consts;
        let pkg_params = &pkg_inst.params;

        OraclePattern {
            game_name,
            pkg_name,
            oracle_name,
            oracle_args,
            game_params,
            pkg_params,
        }
    }

    pub(crate) fn dispatch_oracle_pattern(&'a self) -> DispatchOraclePattern<'a> {
        let game_inst = &self.game_inst_context.game_inst;
        let pkg_inst = &game_inst.game().pkgs[self.pkg_inst_offs];

        let game_name = self.game_inst_context.game_inst().game_name();
        let pkg_name = self.pkg_inst_ctx().pkg_name();
        let oracle_sig = &self.oracle_def().sig;

        let game_params = &game_inst.consts;
        let pkg_params = &pkg_inst.params;

        DispatchOraclePattern {
            game_name,
            game_params,
            pkg_name,
            pkg_params,
            oracle_sig,
        }
    }

    pub(crate) fn return_pattern(&self) -> ReturnPattern {
        let game_inst = &self.game_inst_context.game_inst;
        let pkg_inst = &game_inst.game().pkgs[self.pkg_inst_offs];

        let game_name = self.game_inst_context.game_inst().game_name();
        let pkg_name = self.pkg_inst_ctx().pkg_name();
        let oracle_name = self.oracle_name();

        let game_params = &game_inst.consts;
        let pkg_params = &pkg_inst.params;

        ReturnPattern {
            game_name,
            pkg_name,
            oracle_name,
            game_params,
            pkg_params,
        }
    }

    pub(crate) fn return_value_pattern(&self) -> ReturnValue {
        ReturnValue {
            inner_type: self.return_type(),
        }
    }

    pub(crate) fn oracle_arg_game_consts_pattern(&self) -> oracle_args::GameConstsPattern {
        oracle_args::GameConstsPattern {
            game_name: self.game_inst_ctx().game().name(),
        }
    }
    pub(crate) fn oracle_arg_game_state_pattern(&self) -> oracle_args::GameStatePattern {
        oracle_args::GameStatePattern {
            game_name: self.game_inst_ctx().game().name(),
            game_params: &self.game_inst_ctx().game_inst().consts,
        }
    }

    // TODO: (2024-10-09): oracle arg patterns for value args
}

// Getters
impl<'a> OracleContext<'a> {
    fn return_type(&self) -> &Type {
        &self.oracle_def().sig.tipe
    }

    pub(crate) fn oracle_def(&self) -> &'a OracleDef {
        &self.game_inst_context.game_inst.game().pkgs[self.pkg_inst_offs]
            .pkg
            .oracles[self.oracle_offs]
    }
}
// SMT Code Generation
impl<'a> OracleContext<'a> {
    pub(crate) fn smt_arg_name(&self, arg_name: &str) -> SmtExpr {
        let game = self.game_inst_context.game_inst.game();
        let pkg_inst = &game.pkgs[self.pkg_inst_offs];
        let odef = &pkg_inst.pkg.oracles[self.oracle_offs];

        names::oracle_nonsplit_arg_name(&odef.sig.name, arg_name).into()
    }

    pub(crate) fn smt_declare_return(&self) -> SmtExpr {
        let return_type = self.return_type();
        let pattern = self.return_pattern();
        let spec = pattern.datastructure_spec(&return_type);

        declare_datatype(&pattern, &spec)
    }

    pub(crate) fn smt_construct_return<S, V>(&self, state: S, value: V) -> SmtExpr
    where
        S: Into<SmtExpr>,
        V: Into<SmtExpr>,
    {
        let game_inst = self.game_inst_context.game_inst;
        let pkg_inst = &game_inst.game().pkgs[self.pkg_inst_offs];
        let odef = &pkg_inst.pkg.oracles[self.oracle_offs];
        let osig = &odef.sig;
        let return_type = &osig.tipe;

        let return_value_pattern = ReturnValue {
            inner_type: return_type,
        };
        let return_value_spec = return_value_pattern.datastructure_spec(&());

        // we do this here so we can clone the smt expression in the closure below
        // and don't have to require a `Clone` constraint from `value_smt`.
        let value_smt: SmtExpr = value.into();

        let return_value = return_value_pattern
            .call_constructor(&return_value_spec, &ReturnValueConstructor::Return, |_| {
                Some(value_smt.clone())
            })
            .unwrap();

        let return_pattern = self.return_pattern();

        let return_spec = return_pattern.datastructure_spec(&return_type);

        let state_smt: SmtExpr = state.into();

        return_pattern
            .call_constructor(&return_spec, &ReturnConstructor, |sel: &ReturnSelector| {
                Some(match sel {
                    ReturnSelector::GameState => state_smt.clone(),
                    ReturnSelector::ReturnValueOrAbort {
                        return_type: spec_return_type,
                    } => {
                        assert_eq!(*spec_return_type, return_type);
                        return_value.clone()
                    }
                })
            })
            .unwrap()
    }

    pub(crate) fn smt_access_return_state<R>(&self, ret: R) -> SmtExpr
    where
        R: Into<SmtExpr>,
    {
        let game_inst = self.game_inst_context.game_inst;
        let pkg_inst = &game_inst.game().pkgs[self.pkg_inst_offs];
        let osig = &pkg_inst.pkg.oracles[self.oracle_offs].sig;
        let return_type = &osig.tipe;

        let pattern = self.return_pattern();
        let spec = pattern.datastructure_spec(&return_type);

        pattern
            .access(&spec, &ReturnSelector::GameState, ret)
            .unwrap()
    }

    pub(crate) fn smt_access_return_is_abort<R: Into<SmtExpr>>(&self, ret: R) -> SmtExpr {
        let return_type = &self.return_type();
        let return_pattern = self.return_pattern();
        let return_spec = return_pattern.datastructure_spec(return_type);
        let return_value = return_pattern
            .access(
                &return_spec,
                &ReturnSelector::ReturnValueOrAbort { return_type },
                ret,
            )
            .unwrap();

        ("=", return_value, self.smt_construct_abort_return_value()).into()
    }

    pub(crate) fn smt_construct_abort_return_value(&self) -> SmtExpr {
        let pattern = self.return_value_pattern();
        let spec = pattern.datastructure_spec(&());
        pattern
            .call_constructor(&spec, &ReturnValueConstructor::Abort, |_| None)
            .unwrap()
    }

    pub(crate) fn smt_access_return_value<R: Into<SmtExpr>>(&self, ret: R) -> SmtExpr {
        let game_inst = self.game_inst_context.game_inst;
        let pkg_inst = &game_inst.game().pkgs[self.pkg_inst_offs];
        let osig = &pkg_inst.pkg.oracles[self.oracle_offs].sig;
        let return_type = &osig.tipe;

        let pattern = self.return_pattern();
        let spec = pattern.datastructure_spec(&return_type);

        pattern
            .access(
                &spec,
                &ReturnSelector::ReturnValueOrAbort { return_type },
                ret,
            )
            .unwrap()
    }

    /// writes the changes we made to local package state variables back into the package and game state
    pub(crate) fn smt_write_back_state(&self, sample_info: &SampleInfo) -> SmtExpr {
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
}

// Contexts
impl<'a> GenericOracleContext for OracleContext<'a> {
    fn game_inst_ctx(&self) -> GameInstanceContext<'a> {
        self.game_inst_context.clone()
    }

    fn pkg_inst_ctx(&self) -> PackageInstanceContext<'a> {
        PackageInstanceContext {
            game_ctx: self.game_inst_context.clone(),
            inst_offs: self.pkg_inst_offs,
        }
    }

    fn oracle_name(&self) -> &'a str {
        &self.oracle_def().sig.name
    }

    fn oracle_args(&self) -> &'a [(String, Type)] {
        &self.oracle_def().sig.args
    }

    fn oracle_return_type(&self) -> &Type {
        &self.oracle_def().sig.tipe
    }

    fn smt_write_back_state(&self, sample_info: &SampleInfo) -> SmtExpr {
        self.smt_write_back_state(sample_info)
    }

    fn smt_game_state(&self) -> SmtExpr {
        "<game-state>".into()
    }

    fn smt_construct_abort<S: Into<SmtExpr>>(&self, game_state: S) -> SmtExpr {
        let game_inst = self.game_inst_context.game_inst;
        let pkg_inst = &game_inst.game().pkgs[self.pkg_inst_offs];
        let osig = &pkg_inst.pkg.oracles[self.oracle_offs].sig;

        let return_type = self.return_type();

        let return_value_pattern = ReturnValue {
            inner_type: &osig.tipe,
        };
        let return_value_spec = return_value_pattern.datastructure_spec(&());

        let abort = return_value_pattern
            .call_constructor(
                &return_value_spec,
                &ReturnValueConstructor::Abort,
                |_| unreachable!(),
            )
            .unwrap();

        let return_pattern = self.return_pattern();

        let return_spec = return_pattern.datastructure_spec(&return_type);

        let game_state = game_state.into();
        return_pattern
            .call_constructor(&return_spec, &ReturnConstructor, |sel: &ReturnSelector| {
                Some(match sel {
                    ReturnSelector::GameState => game_state.clone(),
                    ReturnSelector::ReturnValueOrAbort { .. } => abort.clone(),
                })
            })
            .unwrap()
    }

    // returns none if the wrong number of arguments were provided
    fn smt_call_oracle_fn<
        GameState: Into<SmtExpr>,
        GameConsts: Into<SmtExpr>,
        Args: IntoIterator<Item = SmtExpr>,
    >(
        &self,
        game_state: GameState,
        game_consts: GameConsts,
        args: Args,
    ) -> Option<SmtExpr> {
        let pattern = self.oracle_pattern();

        let base_args = [game_state.into(), game_consts.into()].into_iter();
        let call_args: Vec<_> = base_args.chain(args).collect();

        pattern.call(&call_args)
    }
}
