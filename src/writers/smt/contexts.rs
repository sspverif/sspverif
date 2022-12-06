use crate::{
    package::{Composition, Export},
    transforms::samplify::SampleInfo,
};

use super::exprs::SmtExpr;

use super::{declare, names};

#[derive(Clone)]
pub struct GameContext<'a> {
    game: &'a Composition,
}

impl<'a> GameContext<'a> {
    pub fn new(game: &'a Composition) -> Self {
        Self { game }
    }

    pub fn pkg_inst_ctx_by_name(&self, inst_name: &str) -> Option<PackageInstanceContext<'a>> {
        self.game
            .pkgs // we only want a single package, no sorting needed
            .iter()
            .position(|pkg| pkg.name == inst_name)
            .map(|inst_offs| PackageInstanceContext {
                game_ctx: self.clone(),
                inst_offs,
            })
    }

    pub fn pkg_inst_ctx_by_offs(&self, inst_offs: usize) -> Option<PackageInstanceContext<'a>> {
        if inst_offs >= self.game.pkgs.len() {
            return None;
        }

        Some(PackageInstanceContext {
            game_ctx: self.clone(),
            inst_offs,
        })
    }

    pub fn exported_oracle_ctx_by_name(&self, oracle_name: &str) -> Option<OracleContext<'a>> {
        let Export(inst_offs, _) = *self
            .game
            .exports
            .iter()
            .find(|Export(_inst_offs, osig)| osig.name == oracle_name)?;

        let inst_ctx = PackageInstanceContext {
            game_ctx: self.clone(),
            inst_offs,
        };

        inst_ctx.oracle_ctx_by_name(oracle_name)
    }

    pub fn smt_sort_gamestate(&self) -> SmtExpr {
        let game_name: &str = &self.game.name;
        names::gamestate_sort_name(game_name).into()
    }

    pub fn smt_declare_gamestate(&self, sample_info: &SampleInfo) -> SmtExpr {
        let game_name: &str = &self.game.name;

        let pkgstate_fields = self.game.ordered_pkgs().into_iter().map(|inst| {
            let inst_name = &inst.name;
            (
                names::gamestate_selector_pkgstate_name(game_name, inst_name),
                names::pkgstate_sort_name(game_name, inst_name).into(),
            )
        });

        let const_fields = self.game.consts.iter().map(|(param_name, tipe)| {
            (
                names::gamestate_selector_param_name(game_name, param_name),
                tipe.into(),
            )
        });

        let rand_fields = (0..sample_info.count).map(|sample_id| {
            (
                names::gamestate_selector_rand_name(game_name, sample_id),
                crate::types::Type::Integer.into(),
            )
        });

        let fields = pkgstate_fields.chain(const_fields).chain(rand_fields);

        declare::declare_datatype(
            &names::gamestate_sort_name(game_name),
            &names::gamestate_constructor_name(game_name),
            fields,
        )
    }

    pub fn smt_access_gamestate_pkgstate<S: Into<SmtExpr>>(
        &self,
        state: S,
        inst_name: &str,
    ) -> Option<SmtExpr> {
        // if the requested package state does not exists, return none
        self.pkg_inst_ctx_by_name(inst_name)?;

        let game_name = &self.game.name;

        Some(
            (
                names::gamestate_selector_pkgstate_name(game_name, inst_name),
                state,
            )
                .into(),
        )
    }

    pub fn smt_update_gamestate_pkgstate<S, V>(
        &self,
        state: S,
        sample_info: &SampleInfo,
        target_name: &str,
        new_value: V,
    ) -> Option<SmtExpr>
    where
        S: Clone + Into<SmtExpr>,
        V: Clone + Into<SmtExpr>,
    {
        let game_name: &str = &self.game.name;
        let mut found = false;

        let pkgstate_fields = self.game.ordered_pkgs().into_iter().map(|inst| {
            let inst_name = &inst.name;

            (
                names::gamestate_selector_pkgstate_name(game_name, inst_name),
                if inst_name == target_name {
                    found = true;
                    new_value.clone().into()
                } else {
                    self.smt_access_gamestate_pkgstate(state.clone(), inst_name)
                        .unwrap()
                },
            )
                .into()
        });

        let const_fields = self.game.consts.iter().map(|(param_name, _tipe)| {
            (
                names::gamestate_selector_param_name(game_name, param_name),
                self.smt_access_gamestate_const(state.clone(), param_name)
                    .unwrap(),
            )
                .into()
        });

        let rand_fields = (0..sample_info.count).map(|sample_id| {
            (
                names::gamestate_selector_rand_name(game_name, sample_id),
                self.smt_access_gamestate_rand(sample_info, state.clone(), sample_id)
                    .unwrap(),
            )
                .into()
        });

        let fields = pkgstate_fields.chain(const_fields).chain(rand_fields);

        let mut fncall = vec![names::gamestate_constructor_name(game_name).into()];
        fncall.extend(fields);

        if !found {
            return None;
        }

        Some(SmtExpr::List(fncall))
    }

    pub fn smt_access_gamestate_const<S: Into<SmtExpr>>(
        &self,
        state: S,
        param_name: &str,
    ) -> Option<SmtExpr> {
        // if the requested constant does not exists, return none
        self.game
            .consts
            .iter()
            .position(|(name, _)| name == param_name)?;

        let game_name = &self.game.name;

        Some(
            (
                names::gamestate_selector_param_name(game_name, param_name),
                state,
            )
                .into(),
        )
    }

    pub fn smt_access_gamestate_rand<S: Into<SmtExpr>>(
        &self,
        sample_info: &SampleInfo,
        state: S,
        sample_id: usize,
    ) -> Option<SmtExpr> {
        // if the requested sample_id does not exists, return none
        if sample_id >= sample_info.count {
            return None;
        }

        let game_name = &self.game.name;

        Some(
            (
                names::gamestate_selector_rand_name(game_name, sample_id),
                state,
            )
                .into(),
        )
    }

    pub fn smt_update_gamestate_rand<S, V>(
        &self,
        state: S,
        sample_info: &SampleInfo,
        target_sample_id: usize,
        new_value: V,
    ) -> Option<SmtExpr>
    where
        S: Clone + Into<SmtExpr>,
        V: Clone + Into<SmtExpr>,
    {
        let game_name: &str = &self.game.name;
        let mut found = false;

        let pkgstate_fields = self.game.ordered_pkgs().into_iter().map(|inst| {
            let inst_name = &inst.name;

            (
                names::gamestate_selector_pkgstate_name(game_name, inst_name),
                self.smt_access_gamestate_pkgstate(state.clone(), inst_name)
                    .unwrap(),
            )
                .into()
        });

        let const_fields = self.game.consts.iter().map(|(param_name, _tipe)| {
            (
                names::gamestate_selector_param_name(game_name, param_name),
                self.smt_access_gamestate_const(state.clone(), param_name)
                    .unwrap(),
            )
                .into()
        });

        let rand_fields = (0..sample_info.count).map(|sample_id| {
            (
                names::gamestate_selector_rand_name(game_name, sample_id),
                if sample_id == target_sample_id {
                    found = true;
                    new_value.clone().into()
                } else {
                    self.smt_access_gamestate_rand(sample_info, state.clone(), sample_id)
                        .unwrap()
                },
            )
                .into()
        });

        let fields = pkgstate_fields.chain(const_fields).chain(rand_fields);

        let mut fncall = vec![names::gamestate_constructor_name(game_name).into()];
        fncall.extend(fields);

        if !found {
            return None;
        }

        Some(SmtExpr::List(fncall))
    }

    // NOTE: This function could be implemented a bit more efficient. If the prover struggles, we could do that.
    // the more efficient implementation would look more like smt_update_gamestate_rand, but instead of new_value it would use (+ 1 old_value)
    // that way we don't query the old state twice
    // actually I don't think it makes a big difference, and also I don't think this function will be used anyway
    pub fn smt_increment_gamestate_rand<S, V>(
        &self,
        state: S,
        sample_info: &SampleInfo,
        target_sample_id: usize,
    ) -> Option<SmtExpr>
    where
        S: Clone + Into<SmtExpr>,
        V: Clone + Into<SmtExpr>,
    {
        let old_value =
            self.smt_access_gamestate_rand(sample_info, state.clone(), target_sample_id)?;
        let new_value = ("+", 1, old_value);
        self.smt_update_gamestate_rand(state, sample_info, target_sample_id, new_value)
    }
}

pub struct PackageInstanceContext<'a> {
    game_ctx: GameContext<'a>,
    inst_offs: usize,
}

impl<'a> PackageInstanceContext<'a> {
    pub fn oracle_ctx_by_name(&self, oracle_name: &str) -> Option<OracleContext<'a>> {
        let inst_offs = self.inst_offs;
        let inst = &self.game_ctx.game.pkgs[inst_offs];
        let oracle_offs = inst
            .pkg
            .oracles
            .iter()
            .position(|odef| odef.sig.name == oracle_name)?;

        Some(OracleContext {
            game_ctx: self.game_ctx.clone(),
            inst_offs,
            oracle_offs,
        })
    }

    pub fn oracle_ctx_by_oracle_offs(&self, oracle_offs: usize) -> Option<OracleContext<'a>> {
        let oracle_count = self.game_ctx.game.pkgs[self.inst_offs].pkg.oracles.len();
        if oracle_offs >= oracle_count {
            return None;
        }

        let game_ctx = self.game_ctx.clone();
        let inst_offs = self.inst_offs;

        Some(OracleContext {
            game_ctx,
            inst_offs,
            oracle_offs,
        })
    }

    pub fn smt_sorts_return(&self) -> Vec<SmtExpr> {
        let oracle_count = self.game_ctx.game.pkgs[self.inst_offs].pkg.oracles.len();

        (0..oracle_count)
            .map(|i| {
                let octx = self.oracle_ctx_by_oracle_offs(i).unwrap();
                octx.smt_sort_return()
            })
            .collect()
    }

    pub fn smt_declare_pkgstate(&self) -> SmtExpr {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];

        let game_name = &game.name;
        let inst_name = &inst.name;

        let fields = inst
            .pkg
            .state
            .iter()
            .cloned()
            .map(|(name, tipe)| (name, tipe.into()));

        declare::declare_datatype(
            &names::pkgstate_sort_name(game_name, inst_name),
            &names::pkgstate_constructor_name(game_name, inst_name),
            fields,
        )
    }

    pub fn smt_access_pkgstate<S: Into<SmtExpr>>(
        &self,
        state: S,
        field_name: &str,
    ) -> Option<SmtExpr> {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];

        let game_name = &game.name;
        let inst_name = &inst.name;

        // return none if no field with that name exists
        inst.pkg
            .state
            .iter()
            .find(|(name, _tipe)| name == field_name)?;

        let access = (
            names::pkgstate_selector_name(game_name, inst_name, field_name),
            state,
        )
            .into();

        Some(access)
    }

    pub fn smt_update_pkgstate<S, V>(&self, state: S, field_name: &str, value: V) -> Option<SmtExpr>
    where
        S: Clone + Into<SmtExpr>,
        V: Clone + Into<SmtExpr>,
    {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];

        let game_name = &game.name;
        let inst_name = &inst.name;

        let mut found = false;

        // an iterator that returns either a selector on the current state
        // or the provided value, effectively updating the target field
        let args: Vec<SmtExpr> = inst
            .pkg
            .state
            .iter()
            .map(|(name, _tipe)| {
                if name == field_name {
                    found = true;
                    value.clone().into()
                } else {
                    // we can unwrap here because this only returns none
                    // if the field doesn't exist, which can't happen here
                    // because we iterate over the fields.
                    self.smt_access_pkgstate(state.clone(), name).unwrap()
                }
            })
            .collect();

        if !found {
            return None;
        }

        let mut constructor_fncall =
            vec![names::pkgstate_constructor_name(game_name, inst_name).into()];
        constructor_fncall.extend(args);

        Some(SmtExpr::List(constructor_fncall))
    }
}

pub struct OracleContext<'a> {
    game_ctx: GameContext<'a>,
    inst_offs: usize,
    oracle_offs: usize,
}

impl<'a> OracleContext<'a> {
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

        let fields = vec![
            (
                names::return_selector_state_name(game_name, inst_name, oracle_name),
                self.smt_sort_return(),
            ),
            (
                names::return_selector_state_length_name(game_name, inst_name, oracle_name),
                "Int".into(),
            ),
            (
                names::return_selector_is_abort_name(game_name, inst_name, oracle_name),
                "Bool".into(),
            ),
            (
                names::return_selector_value_name(game_name, inst_name, oracle_name),
                names::pkgstate_sort_name(game_name, inst_name).into(),
            ),
        ];

        declare::declare_datatype(
            &names::return_sort_name(game_name, inst_name, oracle_name),
            &names::return_constructor_name(game_name, inst_name, oracle_name),
            fields.into_iter(),
        )
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
    pub fn smt_invoke_oracle<I>(&self, args: I) -> Option<SmtExpr>
    where
        I: Iterator<Item = SmtExpr>,
    {
        let game = self.game_ctx.game;
        let inst = &game.pkgs[self.inst_offs];
        let osig = &inst.pkg.oracles[self.oracle_offs].sig;

        let game_name = &game.name;
        let inst_name = &inst.name;
        let oracle_name = &osig.name;

        let expected_len = 1 + osig.args.len();

        let mut cmdline = Vec::with_capacity(expected_len);
        cmdline.push(names::oracle_function_name(game_name, inst_name, oracle_name).into());
        cmdline.extend(args);

        if cmdline.len() != expected_len {
            return None;
        }

        Some(SmtExpr::List(cmdline))
    }
}
