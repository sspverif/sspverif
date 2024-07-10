use crate::{
    package::{Composition, Export, SplitExport},
    proof::GameInstance,
    transforms::samplify::SampleInfo,
    types::Type,
    writers::smt::{
        exprs::{SmtExpr, SmtLet},
        names,
        partials::PartialsDatatype,
        patterns::{
            declare_datatype, DatastructurePattern, GameStateDeclareInfo, GameStatePattern,
            GameStateSelector, GameStateSort, GlobalStatePattern, VariablePattern,
        },
    },
};

use super::{GameInstanceContext, OracleContext, PackageInstanceContext, SplitOracleContext};

impl<'a> GameInstanceContext<'a> {
    pub fn new(game_inst: &'a GameInstance) -> Self {
        GameInstanceContext { game_inst }
    }

    pub fn game_inst(&self) -> &'a GameInstance {
        self.game_inst
    }

    pub fn game(&self) -> &'a Composition {
        self.game_inst().game()
    }

    pub fn smt_var_gamestate() -> SmtExpr {
        "__global_state".into()
    }

    /**
     *
     *
     * I finally need to address the problem of organizing the variables used inside oracle
     * function bodies, i.e.
     * - __global_state
     * - __self_state
     * - ???
     *
     * I feel like this is a new kind of pattern, but I am not sure. It seems somewhat close to the
     * proof constants pattern, but it is not close enough to put it in there. It does feel like I
     * should make a new thing for this, but it seems like there are not enough cases. I think we
     * only have __self_state and __game_state, where the former could even be part of the latter.
     *
     * Well, wait. We also have state variables, consts/params and local variables. We could make a
     * thing that basically manages all variables in a function. This would also mean that we might
     * be able to properly domain-separate them in the future, and we don't need to do the double
     * underscore convention (which means that the user can't call anything "__game_state" -- which
     * is an okay restriction, but we would have to consistently check for it, which we don't).
     *
     *
     *
     *
     */

    pub fn smt_with_new_gamestate<S: Into<SmtExpr>, B: Into<SmtExpr>>(
        state: S,
        body: B,
    ) -> SmtLet<B> {
        let global_state = &GlobalStatePattern;
        SmtLet {
            bindings: vec![(global_state.name(), state.into())],
            body,
        }
    }

    pub fn smt_sort_gamestate(&self) -> SmtExpr {
        GameStateSort {
            game_inst_name: self.game_inst.name(),
        }
        .into()
    }

    pub(crate) fn smt_declare_gamestate(&self, sample_info: &SampleInfo) -> SmtExpr {
        let game_state_pattern = GameStatePattern {
            game_inst_name: self.game_inst.name(),
        };
        let declare_info = GameStateDeclareInfo {
            game_inst: self.game_inst,
            sample_info: &sample_info,
        };

        let spec = game_state_pattern.datastructure_spec(&declare_info);
        declare_datatype(&game_state_pattern, &spec)
    }

    pub fn smt_access_gamestate_pkgstate<S: Into<SmtExpr>>(
        &self,
        state: S,
        pkg_inst_name: &str,
    ) -> Option<SmtExpr> {
        let game_state_pattern = GameStatePattern {
            game_inst_name: self.game_inst.name(),
        };
        let declare_info = GameStateDeclareInfo {
            game_inst: self.game_inst,
            // TODO/HACK: we don't have access to the sample info here, but we
            //            also don't really need it for accessing package state.
            //            we'll just pretend that the package doesn't sample.
            //            while the spec is "wrong" in that it won't contain the
            //            data based for sample instuctions, it should still behave
            //            correctly when queried for package state.
            sample_info: &SampleInfo {
                tipes: vec![],
                count: 0,
                positions: vec![],
            },
        };

        let spec = game_state_pattern.datastructure_spec(&declare_info);
        // if the requested package state does not exists, return none
        self.pkg_inst_ctx_by_name(pkg_inst_name)?;

        let _game_inst_name = self.game_inst.name();

        let selector = GameStateSelector::PackageInstance { pkg_inst_name };

        game_state_pattern.access(&spec, &selector, state)
    }

    pub fn game_state_pattern(&self) -> GameStatePattern {
        let game_inst_name = self.game_inst.name();
        GameStatePattern { game_inst_name }
    }

    fn game_state_declare_info(&self, sample_info: &'a SampleInfo) -> GameStateDeclareInfo {
        let game_inst = self.game_inst;
        GameStateDeclareInfo {
            game_inst,
            sample_info,
        }
    }

    pub fn smt_update_gamestate_pkgstate<S, V>(
        &self,
        gamestate: S,
        sample_info: &SampleInfo,
        target_name: &str,
        new_pkgstate: V,
    ) -> Option<SmtExpr>
    where
        S: Clone + Into<SmtExpr>,
        V: Clone + Into<SmtExpr>,
    {
        let game_state_pattern = self.game_state_pattern();
        let declare_info = self.game_state_declare_info(sample_info);
        let spec = game_state_pattern.datastructure_spec(&declare_info);

        let pkgstate_selector = GameStateSelector::PackageInstance {
            pkg_inst_name: target_name,
        };

        return game_state_pattern.update(&spec, &pkgstate_selector, gamestate, new_pkgstate);
    }

    pub fn smt_access_gamestate_const<S: Into<SmtExpr>>(
        &self,
        state: S,
        param_name: &str,
        sample_info: &SampleInfo,
    ) -> Option<SmtExpr> {
        let game_state_pattern = self.game_state_pattern();
        let declare_info = self.game_state_declare_info(sample_info);
        let spec = game_state_pattern.datastructure_spec(&declare_info);

        let tipe = self.param_type(param_name)?;
        let const_selector = GameStateSelector::Const {
            const_name: param_name,
            tipe,
        };

        return game_state_pattern.access(&spec, &const_selector, state);
    }

    pub fn smt_access_gamestate_rand<S: Into<SmtExpr>>(
        &self,
        sample_info: &SampleInfo,
        state: S,
        sample_id: usize,
    ) -> Option<SmtExpr> {
        let game_state_pattern = self.game_state_pattern();
        let declare_info = self.game_state_declare_info(sample_info);
        let spec = game_state_pattern.datastructure_spec(&declare_info);

        let rand_selector = GameStateSelector::Randomness { sample_id };

        return game_state_pattern.access(&spec, &rand_selector, state);
    }

    pub fn smt_update_gamestate_rand<S, V>(
        &self,
        state: S,
        sample_info: &SampleInfo,
        sample_id: usize,
        new_value: V,
    ) -> Option<SmtExpr>
    where
        S: Clone + Into<SmtExpr>,
        V: Clone + Into<SmtExpr>,
    {
        let game_state_pattern = self.game_state_pattern();
        let declare_info = self.game_state_declare_info(sample_info);
        let spec = game_state_pattern.datastructure_spec(&declare_info);

        let rand_selector = GameStateSelector::Randomness { sample_id };

        return game_state_pattern.update(&spec, &rand_selector, state, new_value);
    }

    // NOTE: This function could be implemented a bit more efficient. If the prover struggles, we could do that.
    // the more efficient implementation would look more like smt_update_gamestate_rand, but instead of new_value it would use (+ 1 old_value)
    // that way we don't query the old state twice
    // actually I don't think it makes a big difference, and also I don't think this function will be used anyway
    pub fn smt_increment_gamestate_rand<S>(
        &self,
        state: S,
        sample_info: &SampleInfo,
        target_sample_id: usize,
    ) -> Option<SmtExpr>
    where
        S: Clone + Into<SmtExpr>,
    {
        let old_value =
            self.smt_access_gamestate_rand(sample_info, state.clone(), target_sample_id)?;
        let new_value = ("+", 1, old_value);
        self.smt_update_gamestate_rand(state, sample_info, target_sample_id, new_value)
    }

    pub fn pkg_inst_ctx_by_name(&self, inst_name: &str) -> Option<PackageInstanceContext<'a>> {
        self.game_inst
            .game()
            .pkgs // we only want a single package, no sorting needed
            .iter()
            .position(|pkg| pkg.name == inst_name)
            .map(|inst_offs| PackageInstanceContext {
                game_ctx: self.clone(),
                inst_offs,
            })
    }

    pub fn pkg_inst_ctx_by_offs(&self, inst_offs: usize) -> Option<PackageInstanceContext<'a>> {
        if inst_offs >= self.game_inst.game().pkgs.len() {
            return None;
        }

        Some(PackageInstanceContext {
            game_ctx: self.clone(),
            inst_offs,
        })
    }

    pub fn exported_oracle_ctx_by_name(&self, oracle_name: &str) -> Option<OracleContext<'a>> {
        let Export(inst_offs, _) = *self
            .game_inst
            .game()
            .exports
            .iter()
            .find(|Export(_inst_offs, osig)| osig.name == oracle_name)?;

        let inst_ctx = PackageInstanceContext {
            game_ctx: self.clone(),
            inst_offs,
        };

        inst_ctx.oracle_ctx_by_name(oracle_name)
    }

    pub fn exported_split_oracle_ctx_by_name(
        &self,
        oracle_name: &str,
        partials: &'a PartialsDatatype,
    ) -> Option<SplitOracleContext<'a>> {
        let inst_ctx = self.pkg_inst_ctx_by_exported_split_oracle_name(oracle_name)?;

        inst_ctx.split_oracle_ctx_by_name(oracle_name, partials)
    }

    pub fn pkg_inst_ctx_by_exported_split_oracle_name(
        &self,
        oracle_name: &str,
    ) -> Option<PackageInstanceContext<'a>> {
        self.game_inst
            .game()
            .split_exports
            .iter()
            .find(|SplitExport(_inst_offs, osig)| osig.name == oracle_name)
            .map(|SplitExport(inst_offs, _osig)| PackageInstanceContext {
                game_ctx: self.clone(),
                inst_offs: *inst_offs,
            })
    }

    pub(super) fn param_type(&self, param_name: &str) -> Option<&'a Type> {
        self.game_inst
            .game()
            .consts
            .iter()
            .find(|(name, _tipe)| name == param_name)
            .map(|(_name, tipe)| tipe)
    }

    pub fn smt_eval_randfn<CTR: Into<SmtExpr>>(
        &self,
        sample_id: usize,
        ctr: CTR,
        tipe: &Type,
    ) -> SmtExpr {
        let rand_fn_name = names::fn_sample_rand_name(&self.game_inst.game().name, tipe);
        (rand_fn_name, sample_id, ctr).into()
    }
}
