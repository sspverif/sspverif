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

    // TODO Add
    // GameInstanceIdentifier(GameInstanceIdentifier),

    // get rid of the rest
    Local(String),
    // TODO add parameter identifiers for each place of definition (package/game/proof)
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

pub mod pkg_inst_ident {
    use crate::types::Type;

    use super::*;
    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub enum PackageInstanceIdentifier {
        State(PackageInstanceStateIdentifier),
        Local(PackageInstanceLocalIdentifier),
        OracleImport(PackageInstanceOracleImportIdentifier),
        OracleArg(PackageInstanceOracleArgIdentifier),
        ImportsLoopVar(PackageInstanceImportsLoopVarIdentifier),
        GameConst(PackageInstanceGameConstIdentifier),
        LoopVar(PackageInstanceGameLoopVarIdentifier),
    }

    impl PackageInstanceIdentifier {
        pub(crate) fn ident_ref(&self) -> &str {
            match self {
                PackageInstanceIdentifier::State(state_ident) => &state_ident.name,
                PackageInstanceIdentifier::Local(local_ident) => &local_ident.name,
                PackageInstanceIdentifier::OracleArg(arg_ident) => &arg_ident.name,
                PackageInstanceIdentifier::OracleImport(oracle_import) => &oracle_import.name,
                PackageInstanceIdentifier::ImportsLoopVar(loopvar) => &loopvar.name,
                PackageInstanceIdentifier::GameConst(game_const) => &game_const.name,
                PackageInstanceIdentifier::LoopVar(loopvar) => &loopvar.name,
            }
        }

        pub(crate) fn ident(&self) -> String {
            self.ident_ref().to_string()
        }

        pub(crate) fn get_type(&self) -> Type {
            match self {
                PackageInstanceIdentifier::State(state_ident) => state_ident.tipe.clone(),
                PackageInstanceIdentifier::Local(local_ident) => local_ident.tipe.clone(),
                PackageInstanceIdentifier::OracleArg(arg_ident) => arg_ident.tipe.clone(),
                PackageInstanceIdentifier::OracleImport(oracle_import) => {
                    oracle_import.return_type.clone()
                }
                PackageInstanceIdentifier::GameConst(const_ident) => const_ident.tipe.clone(),
                PackageInstanceIdentifier::ImportsLoopVar(_loopvar) => Type::Integer,
                PackageInstanceIdentifier::LoopVar(_local_ident) => Type::Integer,
            }
        }
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageInstanceGameConstIdentifier {
        pub game_name: String,
        pub pkg_inst_name: String,
        pub pkg_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageInstanceGameLoopVarIdentifier {
        pub game_name: String,
        pub pkg_inst_name: String,
        pub pkg_name: String,
        pub name: String,
        // tipe is always Integer
        pub start: Box<Expression>,
        pub end: Box<Expression>,
        pub start_comp: ForComp,
        pub end_comp: ForComp,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageInstanceStateIdentifier {
        pub game_name: String,
        pub pkg_inst_name: String,
        pub pkg_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageInstanceLocalIdentifier {
        pub game_name: String,
        pub pkg_inst_name: String,
        pub pkg_name: String,
        pub oracle_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageInstanceOracleArgIdentifier {
        pub game_name: String,
        pub pkg_inst_name: String,
        pub pkg_name: String,
        pub oracle_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageInstanceOracleImportIdentifier {
        pub game_name: String,
        pub pkg_inst_name: String,
        pub pkg_name: String,
        pub name: String,
        pub args: Vec<crate::types::Type>,
        pub return_type: crate::types::Type,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageInstanceImportsLoopVarIdentifier {
        pub game_name: String,
        pub pkg_inst_name: String,
        pub pkg_name: String,
        pub name: String,
        // tipe is always Integer
        pub start: Box<Expression>,
        pub end: Box<Expression>,
        pub start_comp: ForComp,
        pub end_comp: ForComp,
    }
}

//// old stuff /////

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd, Eq, Ord)]
pub struct PackageState {
    pub(crate) name: String,
    pub(crate) pkg_inst_name: String,
    pub(crate) game_inst_name: String,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd, Eq, Ord)]
pub struct PackageConst {
    pub(crate) name_in_pkg: String,
    pub(crate) pkgname: String,
    pub(crate) game_inst_name: String,
    pub(crate) name_in_comp: String,
    pub(crate) name_in_proof: String,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd, Eq, Ord)]
pub struct ComposeLoopVar {
    pub(crate) name_in_pkg: String,
    pub(crate) name_in_comp: String,
    pub(crate) pkgname: String,
    pub(crate) game_inst_name: String,
}

impl Identifier {
    // TODO implement correct converter trait to identifier expression
    pub fn to_expression(&self) -> Expression {
        Expression::Identifier(self.clone())
    }

    pub fn get_type(&self) -> Option<Type> {
        match self {
            Identifier::PackageIdentifier(pkg_ident) => Some(pkg_ident.get_type()),
            Identifier::GameIdentifier(game_ident) => Some(game_ident.get_type()),
            Identifier::ProofIdentifier(proof_ident) => Some(proof_ident.get_type()),
            _ => None,
        }
    }

    pub fn ident_ref(&self) -> &str {
        match self {
            Identifier::Local(name) => name,
            Identifier::PackageIdentifier(pkg_ident) => pkg_ident.ident_ref(),
            Identifier::GameIdentifier(game_ident) => game_ident.ident_ref(),
            Identifier::ProofIdentifier(proof_ident) => proof_ident.ident_ref(),
        }
    }

    pub fn ident(&self) -> String {
        match self {
            Identifier::Local(ident) => ident.clone(),
            Identifier::PackageIdentifier(pkg_ident) => pkg_ident.ident(),
            Identifier::GameIdentifier(game_ident) => game_ident.ident(),
            Identifier::ProofIdentifier(proof_ident) => proof_ident.ident(),
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
