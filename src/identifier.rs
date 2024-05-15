use crate::{expressions::Expression, parser::package::ForComp, types::Type};

use self::{
    game_ident::GameConstIdentifier,
    pkg_ident::{PackageIdentifier, PackageOracleCodeLoopVarIdentifier},
};

// TODO: remove the Parameter and GameInstanceConst variants so we can derive PartialEq again. Then
//       we can also remove the linter exception
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord)]
pub enum Identifier {
    PackageIdentifier(pkg_ident::PackageIdentifier),
    GameIdentifier(game_ident::GameIdentifier),

    // this is likely not needed. We added an Option<GameIdentifier> to the package const type,
    // which will contain the resolved identifer.
    PackageInstanceIdentifier(pkg_inst_ident::PackageInstanceIdentifier),
    // TODO Add
    // GameInstanceIdentifier(GameInstanceIdentifier),

    // get rid of the rest
    Scalar(String),
    State(PackageState),
    Parameter(PackageConst),
    ComposeLoopVar(ComposeLoopVar),
    Local(String),
    GameInstanceConst(GameInstanceConst),
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

// later we can do something like this, not entirely sure about the semantics.
//
// ```
// fn resolve_packageInstanceIdentifier: package_identifier x pkg_inst_name x game -> package_instance_identifier
// ```
//
// the point is we can see the context in which the identifier is valid from the outermost enum
// variant

pub mod pkg_ident {
    use crate::types::Type;

    use self::game_ident::GameIdentifier;

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
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageConstIdentifier {
        pub pkg_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
        pub game_assignment: Option<Box<Expression>>,
    }

    impl PackageConstIdentifier {
        pub(crate) fn ident(&self) -> String {
            self.name.clone()
        }

        pub(crate) fn ident_ref(&self) -> &str {
            &self.name
        }

        pub(crate) fn eq_except_game_assignment(&self, other: &Self) -> bool {
            let Self {
                pkg_name,
                name,
                tipe,
                game_assignment: _unused,
            } = self;

            pkg_name == &other.pkg_name && name == &other.name && tipe == &other.tipe
        }
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageStateIdentifier {
        pub pkg_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageLocalIdentifier {
        pub pkg_name: String,
        pub oracle_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageOracleArgIdentifier {
        pub pkg_name: String,
        pub oracle_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct PackageOracleImportIdentifier {
        pub pkg_name: String,
        pub name: String,
        pub args: Vec<crate::types::Type>,
        pub return_type: crate::types::Type,
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
    }
}

pub mod game_ident {
    use crate::types::Type;

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
                GameIdentifier::LoopVar(local_ident) => Type::Integer,
            }
        }
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub enum GameIdentifier {
        Const(GameConstIdentifier),
        LoopVar(GameLoopVarIdentifier),
    }

    #[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord, PartialEq)]
    pub struct GameConstIdentifier {
        pub game_name: String,
        pub name: String,
        pub tipe: crate::types::Type,
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
                PackageInstanceIdentifier::ImportsLoopVar(loopvar) => Type::Integer,
                PackageInstanceIdentifier::LoopVar(local_ident) => Type::Integer,
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

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd, Eq, Ord)]
pub struct GameInstanceConst {
    pub(crate) game_inst_name: String,
    pub(crate) name_in_comp: String,
    pub(crate) name_in_proof: String,
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Scalar(x), Self::Scalar(y)) => x == y,
            (Self::Local(x), Self::Local(y)) => x == y,
            (Self::State(l), Self::State(r)) => l == r,
            (Self::Parameter(l), Self::Parameter(r)) => l == r,
            (Self::ComposeLoopVar(l), Self::ComposeLoopVar(r)) => l == r,
            (Self::GameInstanceConst(l), Self::GameInstanceConst(r)) => l == r,

            (Self::Parameter(pkg_param), Self::GameInstanceConst(game_inst_const))
            | (Self::GameInstanceConst(game_inst_const), Self::Parameter(pkg_param)) => {
                pkg_param.game_inst_name == game_inst_const.game_inst_name
                    && pkg_param.name_in_comp == game_inst_const.name_in_comp
                    && pkg_param.name_in_proof == game_inst_const.name_in_proof
            }
            (Self::GameIdentifier(x), Self::GameIdentifier(y)) => x == y,
            (Self::PackageIdentifier(x), Self::PackageIdentifier(y)) => x == y,
            (Self::PackageInstanceIdentifier(x), Self::PackageInstanceIdentifier(y)) => x == y,
            _ => false,
        }
    }
}

impl Identifier {
    pub fn new_scalar(name: &str) -> Identifier {
        Identifier::Scalar(name.to_string())
    }

    // TODO implement correct converter trait to identifier expression
    pub fn to_expression(&self) -> Expression {
        Expression::Identifier(self.clone())
    }

    pub fn get_type(&self) -> Option<Type> {
        match self {
            Identifier::PackageIdentifier(pkg_ident) => Some(pkg_ident.get_type()),
            Identifier::PackageInstanceIdentifier(pkg_inst_ident) => {
                Some(pkg_inst_ident.get_type())
            }
            Identifier::GameIdentifier(game_ident) => Some(game_ident.get_type()),
            _ => None,
        }
    }

    pub fn ident_ref(&self) -> &str {
        match self {
            Identifier::Scalar(ident) => &ident,
            Identifier::State(PackageState { name, .. }) => &name,
            Identifier::Parameter(PackageConst { name_in_pkg, .. }) => &name_in_pkg,
            Identifier::Local(name) => &name,
            Identifier::ComposeLoopVar(ComposeLoopVar { name_in_pkg, .. }) => &name_in_pkg,
            Identifier::PackageIdentifier(pkg_ident) => pkg_ident.ident_ref(),
            Identifier::GameInstanceConst(_) => todo!(),
            Identifier::GameIdentifier(game_ident) => game_ident.ident_ref(),
            Identifier::PackageInstanceIdentifier(pkg_inst_ident) => pkg_inst_ident.ident_ref(),
        }
    }

    pub fn ident(&self) -> String {
        match self {
            Identifier::Scalar(ident) => ident.clone(),
            Identifier::Local(ident) => ident.clone(),
            Identifier::State(PackageState { name, .. }) => name.clone(),
            Identifier::Parameter(PackageConst { name_in_pkg, .. }) => name_in_pkg.clone(),
            Identifier::ComposeLoopVar(ComposeLoopVar { name_in_pkg, .. }) => name_in_pkg.clone(),
            Identifier::PackageIdentifier(pkg_ident) => pkg_ident.ident(),
            Identifier::GameInstanceConst(_) => todo!(),
            Identifier::GameIdentifier(game_ident) => game_ident.ident(),
            Identifier::PackageInstanceIdentifier(pkg_inst_ident) => pkg_inst_ident.ident(),
        }
    }
}

#[cfg(test)]
mod tests {
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
}
