// SPDX-License-Identifier: Apache-2.0

use sspverif_smtlib::{
    syntax::{command::Command, term::Term},
    theories,
};

use crate::types::Type;

use super::ConstantPattern;

pub struct ProofConstant<'a> {
    pub proof_name: &'a str,
    pub ident_name: &'a str,
    pub ty: &'a Type,
}

pub struct GameInstanceConstant<'a> {
    pub proof_name: &'a str,
    pub game_inst_name: &'a str,
    pub ident_name: &'a str,
    pub ty: &'a Type,
}

pub struct PackageInstanceConstant<'a> {
    pub proof_name: &'a str,
    pub game_inst_name: &'a str,
    pub pkg_inst_name: &'a str,
    pub ident_name: &'a str,
    pub ty: &'a Type,
}

pub struct GameInstanceConstantAssigment<'a> {
    pub proof_const: ProofConstant<'a>,
    pub game_inst_const: GameInstanceConstant<'a>,
}

pub struct PackageInstanceConstantAssigment<'a> {
    pub game_inst_const: GameInstanceConstant<'a>,
    pub pkg_inst_const: PackageInstanceConstant<'a>,
}

impl super::ConstantPattern for ProofConstant<'_> {
    fn name(&self) -> String {
        let Self {
            proof_name,
            ident_name,
            ..
        } = self;
        format!("<$const-proof-{proof_name}-{ident_name}$>")
    }

    fn sort(&self) -> crate::writers::smt::sorts::Sort {
        self.ty.clone().into()
    }
}

impl super::ConstantPattern for GameInstanceConstant<'_> {
    fn name(&self) -> String {
        let Self {
            proof_name,
            game_inst_name,
            ident_name,
            ..
        } = self;
        format!("<$const-gameinst-{proof_name}-{game_inst_name}-{ident_name}$>")
    }

    fn sort(&self) -> crate::writers::smt::sorts::Sort {
        self.ty.clone().into()
    }
}

impl super::ConstantPattern for PackageInstanceConstant<'_> {
    fn name(&self) -> String {
        let Self {
            proof_name,
            game_inst_name,
            pkg_inst_name,
            ident_name,
            ..
        } = self;
        format!("<$const-pkginst-{proof_name}-{game_inst_name}-{pkg_inst_name}-{ident_name}$>")
    }

    fn sort(&self) -> crate::writers::smt::sorts::Sort {
        self.ty.clone().into()
    }
}

impl From<GameInstanceConstantAssigment<'_>> for Command {
    fn from(value: GameInstanceConstantAssigment<'_>) -> Self {
        let proof_ident = Term::Base(value.proof_const.name().into(), vec![]);
        let game_inst_ident = Term::Base(value.game_inst_const.name().into(), vec![]);

        Command::Assert(theories::core::eq(vec![proof_ident, game_inst_ident]))
    }
}

impl From<PackageInstanceConstantAssigment<'_>> for Command {
    fn from(value: PackageInstanceConstantAssigment<'_>) -> Self {
        let game_inst_ident = Term::Base(value.game_inst_const.name().into(), vec![]);
        let pkg_inst_ident = Term::Base(value.pkg_inst_const.name().into(), vec![]);

        Command::Assert(theories::core::eq(vec![game_inst_ident, pkg_inst_ident]))
    }
}
