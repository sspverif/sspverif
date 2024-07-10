use crate::package::{Composition, PackageInstance};
use crate::proof::GameInstance;
use crate::split::SplitPath;
use crate::writers::smt::partials::PartialsDatatype;
use crate::writers::smt::patterns::{declare_datatype, PackageStateSelector};
use crate::writers::smt::{
    contexts::{GameInstanceContext, OracleContext, PackageInstanceContext, SplitOracleContext},
    exprs::SmtExpr,
    patterns::{DatastructurePattern, PackageStatePattern},
};

impl<'a> PackageInstanceContext<'a> {
    pub fn game_inst_ctx(&self) -> GameInstanceContext<'a> {
        self.game_ctx.clone()
    }

    pub fn game_inst(&self) -> &'a GameInstance {
        self.game_ctx.game_inst
    }

    pub fn game(&self) -> &'a Composition {
        self.game_inst().game()
    }

    pub fn split_oracle_ctx_by_name(
        &self,
        oracle_name: &str,
        partials: &'a PartialsDatatype,
    ) -> Option<SplitOracleContext<'a>> {
        let inst_offs = self.inst_offs;
        let inst = &self.game().pkgs[inst_offs];
        let split_oracle_offs = inst
            .pkg
            .split_oracles
            .iter()
            .position(|odef| odef.sig.name == oracle_name)?;

        Some(SplitOracleContext {
            game_inst_context: self.game_ctx.clone(),
            pkg_inst_offs: inst_offs,
            split_oracle_offs,
            partials,
        })
    }

    pub fn oracle_ctx_by_name(&self, oracle_name: &str) -> Option<OracleContext<'a>> {
        let inst_offs = self.inst_offs;
        let inst = &self.game().pkgs[inst_offs];
        let oracle_offs = inst
            .pkg
            .oracles
            .iter()
            .position(|odef| odef.sig.name == oracle_name)?;

        Some(OracleContext {
            game_inst_context: self.game_ctx.clone(),
            pkg_inst_offs: inst_offs,
            oracle_offs,
        })
    }

    pub fn split_oracle_ctx_by_name_and_path(
        &self,
        oracle_name: &str,
        oracle_path: &SplitPath,
        partials: &'a PartialsDatatype,
    ) -> Option<SplitOracleContext<'a>> {
        let inst_offs = self.inst_offs;
        let inst = &self.game().pkgs[inst_offs];
        let split_oracle_offs = inst
            .pkg
            .split_oracles
            .iter()
            .position(|odef| odef.sig.name == oracle_name && &odef.sig.path == oracle_path)?;

        Some(SplitOracleContext {
            game_inst_context: self.game_ctx.clone(),
            pkg_inst_offs: inst_offs,
            split_oracle_offs,
            partials,
        })
    }

    pub fn oracle_ctx_by_oracle_offs(&self, oracle_offs: usize) -> Option<OracleContext<'a>> {
        let oracle_count = self.game().pkgs[self.inst_offs].pkg.oracles.len();
        if oracle_offs >= oracle_count {
            return None;
        }

        let game_ctx = self.game_ctx.clone();
        let inst_offs = self.inst_offs;

        Some(OracleContext {
            game_inst_context: game_ctx,
            pkg_inst_offs: inst_offs,
            oracle_offs,
        })
    }

    pub fn pkg_inst_name(&self) -> &'a str {
        &self.game().pkgs[self.inst_offs].name
    }

    pub fn pkg_inst(&self) -> &'a PackageInstance {
        &self.game().pkgs[self.inst_offs]
    }

    pub fn pkg_state_pattern(&self) -> PackageStatePattern<'a> {
        let game_inst_ctx = self.game_inst_ctx();
        let game_inst_name = game_inst_ctx.game_inst.name();
        let pkg_inst_name = self.pkg_inst_name();

        PackageStatePattern {
            game_inst_name,
            pkg_inst_name,
        }
    }

    pub fn smt_sorts_return(&self) -> Vec<SmtExpr> {
        let oracle_count = self.game().pkgs[self.inst_offs].pkg.oracles.len();

        (0..oracle_count)
            .map(|i| {
                let octx = self.oracle_ctx_by_oracle_offs(i).unwrap();
                octx.smt_sort_return()
            })
            .collect()
    }

    pub fn smt_declare_pkgstate(&self) -> SmtExpr {
        let pkg = &self.pkg_inst().pkg;
        let pattern = self.pkg_state_pattern();
        let spec = pattern.datastructure_spec(&pkg);

        return declare_datatype(&pattern, &spec);
    }

    pub fn smt_access_pkgstate<S: Into<SmtExpr>>(
        &self,
        pkg_state: S,
        field_name: &str,
    ) -> Option<SmtExpr> {
        let pkg = &self.game().pkgs[self.inst_offs].pkg;

        let pkg_state_pattern = self.pkg_state_pattern();
        let pkg_state_spec = pkg_state_pattern.datastructure_spec(pkg);
        let pkg_state_field = self.field_selector(field_name)?;

        pkg_state_pattern.access(&pkg_state_spec, &pkg_state_field, pkg_state)
    }

    fn field_selector(&self, field_name: &'a str) -> Option<PackageStateSelector<'a>> {
        let pkg_inst = &self.game().pkgs[self.inst_offs];

        // return none if no field with that name exists
        let (_, tipe, _) = pkg_inst
            .pkg
            .state
            .iter()
            .find(|(name, _tipe, _file_pos)| name == field_name)?;

        Some(PackageStateSelector {
            name: field_name,
            tipe,
        })
    }

    pub fn smt_update_pkgstate<S, V>(&self, state: S, field_name: &str, value: V) -> Option<SmtExpr>
    where
        S: Clone + Into<SmtExpr>,
        V: Clone + Into<SmtExpr>,
    {
        let game = self.game();
        let pkg_inst = &game.pkgs[self.inst_offs];
        let pkg = &pkg_inst.pkg;

        let state: SmtExpr = state.into();

        let pkg_state_pattern = self.pkg_state_pattern();
        let pkg_state_spec = pkg_state_pattern.datastructure_spec(pkg);
        pkg_state_pattern.call_constructor(&pkg_state_spec, &(), |sel| {
            let PackageStateSelector { name, .. } = sel;

            Some(if name == &field_name {
                value.clone().into()
            } else {
                self.smt_access_pkgstate(state.clone(), name)?
            })
        })
    }
}
