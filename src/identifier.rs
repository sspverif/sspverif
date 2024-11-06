use game_ident::{GameIdentInstanciationInfo, GameIdentifier};
use proof_ident::ProofIdentInstanciationInfo;

use crate::{expressions::Expression, parser::package::ForComp, types::Type};

use self::{
    game_ident::GameConstIdentifier,
    pkg_ident::{PackageIdentifier, PackageOracleCodeLoopVarIdentifier},
};

#[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
pub enum Identifier {
    PackageIdentifier(pkg_ident::PackageIdentifier),
    GameIdentifier(game_ident::GameIdentifier),
    ProofIdentifier(proof_ident::ProofIdentifier),

    /// Denotes identifiers that were injected by transforms.
    /// Should only live inside oracle code
    Generated(String, Type),
}

impl From<GameConstIdentifier> for Identifier {
    fn from(value: GameConstIdentifier) -> Self {
        Identifier::GameIdentifier(game_ident::GameIdentifier::Const(value))
    }
}

impl From<PackageOracleCodeLoopVarIdentifier> for Identifier {
    fn from(value: PackageOracleCodeLoopVarIdentifier) -> Self {
        Identifier::PackageIdentifier(PackageIdentifier::CodeLoopVar(value))
    }
}

impl From<PackageIdentifier> for Identifier {
    fn from(value: PackageIdentifier) -> Self {
        Identifier::PackageIdentifier(value)
    }
}

impl From<GameIdentifier> for Identifier {
    fn from(value: GameIdentifier) -> Self {
        Identifier::GameIdentifier(value)
    }
}

pub mod pkg_ident {
    use crate::types::Type;

    use super::*;

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub enum PackageIdentifier {
        Const(PackageConstIdentifier),
        State(PackageStateIdentifier),
        Local(PackageLocalIdentifier),
        OracleImport(PackageOracleImportIdentifier),
        OracleArg(PackageOracleArgIdentifier),
        ImportsLoopVar(PackageImportsLoopVarIdentifier),
        CodeLoopVar(PackageOracleCodeLoopVarIdentifier),
    }

    impl PackageIdentifier {
        pub(crate) fn ident_ref(&self) -> &str {
            match self {
                PackageIdentifier::Const(const_ident) => &const_ident.name,
                PackageIdentifier::State(state_ident) => &state_ident.name,
                PackageIdentifier::Local(local_ident) => &local_ident.name,
                PackageIdentifier::OracleArg(arg_ident) => &arg_ident.name,
                PackageIdentifier::OracleImport(oracle_import) => &oracle_import.name,
                PackageIdentifier::ImportsLoopVar(loopvar) => &loopvar.name,
                PackageIdentifier::CodeLoopVar(loopvar) => &loopvar.name,
            }
        }

        pub(crate) fn ident(&self) -> String {
            self.ident_ref().to_string()
        }

        pub(crate) fn get_type(&self) -> Type {
            match self {
                PackageIdentifier::Const(const_ident) => const_ident.tipe.clone(),
                PackageIdentifier::State(state_ident) => state_ident.tipe.clone(),
                PackageIdentifier::Local(local_ident) => local_ident.tipe.clone(),
                PackageIdentifier::OracleArg(arg_ident) => arg_ident.tipe.clone(),
                PackageIdentifier::OracleImport(oracle_import) => oracle_import.return_type.clone(),
                PackageIdentifier::ImportsLoopVar(_loopvar) => Type::Integer,
                PackageIdentifier::CodeLoopVar(_loopvar) => Type::Integer,
            }
        }

        pub(crate) fn type_mut(&mut self) -> &mut Type {
            match self {
                PackageIdentifier::Const(const_ident) => &mut const_ident.tipe,
                PackageIdentifier::State(state_ident) => &mut state_ident.tipe,
                PackageIdentifier::Local(local_ident) => &mut local_ident.tipe,
                PackageIdentifier::OracleArg(arg_ident) => &mut arg_ident.tipe,
                PackageIdentifier::OracleImport(oracle_import) => &mut oracle_import.return_type,
                PackageIdentifier::ImportsLoopVar(_) | PackageIdentifier::CodeLoopVar(_) => {
                    panic!("cannot provide a mutable reference to the type of a loopvar")
                }
            }
        }

        pub(crate) fn set_pkg_inst_info(&mut self, pkg_inst_name: String, game_name: String) {
            match self {
                PackageIdentifier::Const(id) => id.set_pkg_inst_info(pkg_inst_name, game_name),
                PackageIdentifier::State(id) => id.set_pkg_inst_info(pkg_inst_name, game_name),
                PackageIdentifier::Local(id) => id.set_pkg_inst_info(pkg_inst_name, game_name),
                PackageIdentifier::OracleImport(id) => {
                    id.set_pkg_inst_info(pkg_inst_name, game_name)
                }
                PackageIdentifier::OracleArg(id) => id.set_pkg_inst_info(pkg_inst_name, game_name),
                PackageIdentifier::ImportsLoopVar(id) => {
                    id.set_pkg_inst_info(pkg_inst_name, game_name)
                }
                PackageIdentifier::CodeLoopVar(id) => {
                    id.set_pkg_inst_info(pkg_inst_name, game_name)
                }
            }
        }

        pub(crate) fn set_game_inst_info(&mut self, game_inst_name: String, proof_name: String) {
            match self {
                PackageIdentifier::Const(id) => id.set_game_inst_info(game_inst_name, proof_name),
                PackageIdentifier::State(id) => id.set_game_inst_info(game_inst_name, proof_name),
                PackageIdentifier::Local(id) => id.set_game_inst_info(game_inst_name, proof_name),
                PackageIdentifier::OracleImport(id) => {
                    id.set_game_inst_info(game_inst_name, proof_name)
                }
                PackageIdentifier::OracleArg(id) => {
                    id.set_game_inst_info(game_inst_name, proof_name)
                }
                PackageIdentifier::ImportsLoopVar(id) => {
                    id.set_game_inst_info(game_inst_name, proof_name)
                }
                PackageIdentifier::CodeLoopVar(id) => {
                    id.set_game_inst_info(game_inst_name, proof_name)
                }
            }
        }
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageConstIdentifier {
        pub pkg_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
        pub game_assignment: Option<Box<Expression>>,
        pub pkg_inst_name: Option<String>,
        pub game_name: Option<String>,
        pub game_inst_name: Option<String>,
        pub proof_name: Option<String>,
    }

    impl From<PackageConstIdentifier> for Identifier {
        fn from(value: PackageConstIdentifier) -> Self {
            Identifier::PackageIdentifier(PackageIdentifier::Const(value))
        }
    }

    impl PackageConstIdentifier {
        pub(crate) fn new(name: String, pkg_name: String, ty: Type) -> Self {
            Self {
                pkg_name,
                name,
                tipe: ty,
                game_assignment: None,
                pkg_inst_name: None,
                game_name: None,
                game_inst_name: None,
                proof_name: None,
            }
        }

        pub(crate) fn ident(&self) -> String {
            self.name.clone()
        }

        pub(crate) fn ident_ref(&self) -> &str {
            &self.name
        }

        pub(crate) fn eq_except_game_assignment(&self, other: &Self) -> bool {
            let left = Self {
                game_assignment: None,
                ..self.clone()
            };

            let right = Self {
                game_assignment: None,
                ..other.clone()
            };

            left == right
        }
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageStateIdentifier {
        pub pkg_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
        pub pkg_inst_name: Option<String>,
        pub game_name: Option<String>,
        pub game_inst_name: Option<String>,
        pub proof_name: Option<String>,
    }

    impl From<PackageStateIdentifier> for Identifier {
        fn from(value: PackageStateIdentifier) -> Self {
            Identifier::PackageIdentifier(PackageIdentifier::State(value))
        }
    }

    macro_rules! impl_set_inst_info {
        ($idtype:ty) => {
            impl $idtype {
                pub(crate) fn set_pkg_inst_info(
                    &mut self,
                    pkg_inst_name: String,
                    game_name: String,
                ) {
                    self.pkg_inst_name = Some(pkg_inst_name);
                    self.game_name = Some(game_name);
                }

                pub(crate) fn set_game_inst_info(
                    &mut self,
                    game_inst_name: String,
                    proof_name: String,
                ) {
                    self.game_inst_name = Some(game_inst_name);
                    self.proof_name = Some(proof_name);
                }
            }
        };
    }

    impl PackageStateIdentifier {
        pub(crate) fn new(name: String, pkg_name: String, ty: Type) -> Self {
            Self {
                pkg_name,
                name,
                tipe: ty,
                pkg_inst_name: None,
                game_name: None,
                game_inst_name: None,
                proof_name: None,
            }
        }
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageLocalIdentifier {
        pub pkg_name: String,
        pub oracle_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
        pub pkg_inst_name: Option<String>,
        pub game_name: Option<String>,
        pub game_inst_name: Option<String>,
        pub proof_name: Option<String>,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageOracleArgIdentifier {
        pub pkg_name: String,
        pub oracle_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
        pub pkg_inst_name: Option<String>,
        pub game_name: Option<String>,
        pub game_inst_name: Option<String>,
        pub proof_name: Option<String>,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageOracleImportIdentifier {
        pub pkg_name: String,
        pub name: String,
        pub args: Vec<crate::types::Type>,
        pub return_type: crate::types::Type,
        pub pkg_inst_name: Option<String>,
        pub game_name: Option<String>,
        pub game_inst_name: Option<String>,
        pub proof_name: Option<String>,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageImportsLoopVarIdentifier {
        pub pkg_name: String,
        pub name: String,
        // tipe is always Integer
        pub start: Box<Expression>,
        pub end: Box<Expression>,
        pub start_comp: ForComp,
        pub end_comp: ForComp,
        pub pkg_inst_name: Option<String>,
        pub game_name: Option<String>,
        pub game_inst_name: Option<String>,
        pub proof_name: Option<String>,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageOracleCodeLoopVarIdentifier {
        pub pkg_name: String,
        pub name: String,
        // tipe is always Integer
        pub start: Box<Expression>,
        pub end: Box<Expression>,
        pub start_comp: ForComp,
        pub end_comp: ForComp,
        pub pkg_inst_name: Option<String>,
        pub game_name: Option<String>,
        pub game_inst_name: Option<String>,
        pub proof_name: Option<String>,
    }

    impl_set_inst_info!(PackageConstIdentifier);
    impl_set_inst_info!(PackageStateIdentifier);
    impl_set_inst_info!(PackageLocalIdentifier);
    impl_set_inst_info!(PackageOracleArgIdentifier);
    impl_set_inst_info!(PackageOracleImportIdentifier);
    impl_set_inst_info!(PackageImportsLoopVarIdentifier);
    impl_set_inst_info!(PackageOracleCodeLoopVarIdentifier);
}

/*
*
* - im code soll der identifier stehen, der beschreibt wo der wert deklariert wird
*
* - pkg instanziieren:
*   - pkg const ident   -> aufloesen
*   - pkg loopvar ident -> anreichern
*
* - game instanziieren:
*   - game const ident   -> aufloesen
*   - game loopvar ident -> anreichern
*
*
*
* */

pub mod game_ident {
    use crate::types::Type;

    use self::pkg_ident::PackageConstIdentifier;

    use super::*;

    impl GameIdentifier {
        pub(crate) fn ident_ref(&self) -> &str {
            match self {
                GameIdentifier::Const(const_ident) => &const_ident.name,
                GameIdentifier::LoopVar(loopvar) => &loopvar.name,
            }
        }

        pub(crate) fn ident(&self) -> String {
            self.ident_ref().to_string()
        }

        pub(crate) fn get_type(&self) -> Type {
            match self {
                GameIdentifier::Const(const_ident) => const_ident.tipe.clone(),
                GameIdentifier::LoopVar(_local_ident) => Type::Integer,
            }
        }
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub enum GameIdentifier {
        Const(GameConstIdentifier),
        LoopVar(GameLoopVarIdentifier),
    }

    impl GameIdentifier {
        pub fn game_name(&self) -> &str {
            match self {
                GameIdentifier::Const(c) => &c.game_name,
                GameIdentifier::LoopVar(l) => &l.game_name,
            }
        }

        pub fn with_instance_info(self, inst_info: GameIdentInstanciationInfo) -> Self {
            match self {
                GameIdentifier::Const(c) => Self::Const(GameConstIdentifier {
                    inst_info: Some(inst_info),
                    ..c
                }),
                GameIdentifier::LoopVar(l) => Self::LoopVar(GameLoopVarIdentifier {
                    inst_info: Some(inst_info),
                    ..l
                }),
            }
        }
    }

    impl GameIdentifier {
        pub(crate) fn set_game_inst_info(&mut self, game_inst_name: String, proof_name: String) {
            match self {
                GameIdentifier::Const(id) => id.set_game_inst_info(game_inst_name, proof_name),
                GameIdentifier::LoopVar(id) => id.set_game_inst_info(game_inst_name, proof_name),
            }
        }
    }
    impl GameConstIdentifier {
        pub(crate) fn set_game_inst_info(&mut self, game_inst_name: String, proof_name: String) {
            self.game_inst_name = Some(game_inst_name);
            self.proof_name = Some(proof_name);
        }
    }
    impl GameLoopVarIdentifier {
        pub(crate) fn set_game_inst_info(&mut self, game_inst_name: String, proof_name: String) {
            self.game_inst_name = Some(game_inst_name);
            self.proof_name = Some(proof_name);
        }
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct GameIdentInstanciationInfo {
        pub lower: PackageConstIdentifier,
        pub pkg_inst_name: String,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct GameConstIdentifier {
        pub game_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
        pub game_inst_name: Option<String>,
        pub proof_name: Option<String>,
        pub inst_info: Option<GameIdentInstanciationInfo>,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct GameLoopVarIdentifier {
        pub game_name: String,
        pub name: String,
        // tipe is always Integer
        pub start: Box<Expression>,
        pub end: Box<Expression>,
        pub start_comp: ForComp,
        pub end_comp: ForComp,
        pub game_inst_name: Option<String>,
        pub proof_name: Option<String>,
        pub inst_info: Option<GameIdentInstanciationInfo>,
    }

    impl From<GameLoopVarIdentifier> for Identifier {
        fn from(value: GameLoopVarIdentifier) -> Self {
            Identifier::GameIdentifier(GameIdentifier::LoopVar(value))
        }
    }
}

pub mod proof_ident {
    use crate::types::Type;

    use super::*;

    impl ProofIdentifier {
        pub(crate) fn ident_ref(&self) -> &str {
            match self {
                ProofIdentifier::Const(const_ident) => &const_ident.name,
                ProofIdentifier::LoopVar(loopvar) => &loopvar.name,
            }
        }

        pub(crate) fn ident(&self) -> String {
            self.ident_ref().to_string()
        }

        pub(crate) fn get_type(&self) -> Type {
            match self {
                ProofIdentifier::Const(const_ident) => const_ident.tipe.clone(),
                ProofIdentifier::LoopVar(_local_ident) => Type::Integer,
            }
        }
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub enum ProofIdentifier {
        Const(ProofConstIdentifier),
        LoopVar(ProofLoopVarIdentifier),
    }

    impl ProofIdentifier {
        pub fn with_instance_info(self, inst_info: ProofIdentInstanciationInfo) -> Self {
            match self {
                ProofIdentifier::Const(c) => Self::Const(ProofConstIdentifier {
                    inst_info: Some(inst_info),
                    ..c
                }),
                ProofIdentifier::LoopVar(l) => Self::LoopVar(ProofLoopVarIdentifier {
                    inst_info: Some(inst_info),
                    ..l
                }),
            }
        }

        pub fn proof_name(&self) -> &str {
            match self {
                ProofIdentifier::Const(c) => &c.proof_name,
                ProofIdentifier::LoopVar(l) => &l.proof_name,
            }
        }
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct ProofIdentInstanciationInfo {
        pub lower: GameConstIdentifier,
        pub game_inst_name: String,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct ProofConstIdentifier {
        pub proof_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
        pub inst_info: Option<ProofIdentInstanciationInfo>,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct ProofLoopVarIdentifier {
        pub proof_name: String,
        pub name: String,
        // tipe is always Integer
        pub start: Box<Expression>,
        pub end: Box<Expression>,
        pub start_comp: ForComp,
        pub end_comp: ForComp,
        pub inst_info: Option<ProofIdentInstanciationInfo>,
    }

    impl From<ProofConstIdentifier> for Identifier {
        fn from(value: ProofConstIdentifier) -> Self {
            Identifier::ProofIdentifier(ProofIdentifier::Const(value))
        }
    }

    impl From<ProofLoopVarIdentifier> for Identifier {
        fn from(value: ProofLoopVarIdentifier) -> Self {
            Identifier::ProofIdentifier(ProofIdentifier::LoopVar(value))
        }
    }
}

impl From<Identifier> for Expression {
    fn from(value: Identifier) -> Self {
        Expression::Identifier(value)
    }
}

impl Identifier {
    pub fn get_type(&self) -> Type {
        match self {
            Identifier::PackageIdentifier(pkg_ident) => pkg_ident.get_type(),
            Identifier::GameIdentifier(game_ident) => game_ident.get_type(),
            Identifier::ProofIdentifier(proof_ident) => proof_ident.get_type(),
            Identifier::Generated(_, ty) => ty.clone(),
        }
    }

    pub fn ident_ref(&self) -> &str {
        match self {
            Identifier::Generated(name, _) => name,
            Identifier::PackageIdentifier(pkg_ident) => pkg_ident.ident_ref(),
            Identifier::GameIdentifier(game_ident) => game_ident.ident_ref(),
            Identifier::ProofIdentifier(proof_ident) => proof_ident.ident_ref(),
        }
    }

    pub fn ident(&self) -> String {
        match self {
            Identifier::Generated(ident, _) => ident.clone(),
            Identifier::PackageIdentifier(pkg_ident) => pkg_ident.ident(),
            Identifier::GameIdentifier(game_ident) => game_ident.ident(),
            Identifier::ProofIdentifier(proof_ident) => proof_ident.ident(),
        }
    }

    pub(crate) fn set_pkg_inst_info(&mut self, pkg_inst_name: String, game_name: String) {
        match self {
            Identifier::PackageIdentifier(id) => id.set_pkg_inst_info(pkg_inst_name, game_name),
            Identifier::GameIdentifier(_) => {}
            Identifier::ProofIdentifier(_) => {}
            Identifier::Generated(_, _) => {}
        }
    }

    pub(crate) fn set_game_inst_info(&mut self, game_inst_name: String, proof_name: String) {
        match self {
            Identifier::PackageIdentifier(id) => id.set_game_inst_info(game_inst_name, proof_name),
            Identifier::GameIdentifier(id) => id.set_game_inst_info(game_inst_name, proof_name),
            Identifier::ProofIdentifier(_) => {}
            Identifier::Generated(_, _) => {}
        }
    }

    pub(crate) fn with_instance_info(
        &mut self,
        game_ident_inst_info: Option<GameIdentInstanciationInfo>,
        proof_ident_inst_info: Option<ProofIdentInstanciationInfo>,
    ) -> Self {
        match (self, game_ident_inst_info, proof_ident_inst_info) {
            (ident @ Identifier::PackageIdentifier(_), _, _) => ident.clone(),
            (ident @ Identifier::Generated(_, _), _, _) => ident.clone(),
            (Identifier::GameIdentifier(game_ident), Some(inst_info), None) => {
                Identifier::GameIdentifier(game_ident.clone().with_instance_info(inst_info))
            }
            (Identifier::ProofIdentifier(proof_ident), None, Some(inst_info)) => {
                Identifier::ProofIdentifier(proof_ident.clone().with_instance_info(inst_info))
            }
            (other, _, _) => unreachable!("{other:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    /* while we should test equality of identifiers, this one is not interesting because it tests variants that we want to get rid of
    use super::{GameInstanceConst, Identifier, PackageConst};

    #[test]
    fn identifier_equality() {
        let left = Identifier::Parameter(PackageConst {
            name_in_pkg: "d".to_string(),
            pkgname: "Mod".to_string(),
            game_inst_name: "MODSec0_inst".to_string(),
            name_in_comp: "d".to_string(),
            name_in_proof: "d".to_string(),
        });

        let right = Identifier::GameInstanceConst(GameInstanceConst {
            game_inst_name: "MODSec0_inst".to_string(),
            name_in_comp: "d".to_string(),
            name_in_proof: "d".to_string(),
        });

        assert_eq!(left, right)
        }
    */
}
