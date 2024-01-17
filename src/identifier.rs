use crate::expressions::Expression;

#[derive(Debug, Clone, Hash, PartialOrd, Eq, Ord)]
pub enum Identifier {
    Scalar(String),
    State(PackageState),
    Parameter(PackageConst),
    ComposeLoopVar(ComposeLoopVar),
    Local(String),
    GameInstanceConst(GameInstanceConst),
    // TODO add parameter identifiers for each place of definition (package/game/proof)
}

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

    pub fn ident_ref(&self) -> &str {
        match self {
            Identifier::Scalar(ident) => &ident,
            Identifier::State(PackageState { name, .. }) => &name,
            Identifier::Parameter(PackageConst { name_in_pkg, .. }) => &name_in_pkg,
            Identifier::Local(name) => &name,
            Identifier::ComposeLoopVar(ComposeLoopVar { name_in_pkg, .. }) => &name_in_pkg,
            Identifier::GameInstanceConst(_) => todo!(),
        }
    }

    pub fn ident(&self) -> String {
        match self {
            Identifier::Scalar(ident) => ident.clone(),
            Identifier::Local(ident) => ident.clone(),
            Identifier::State(PackageState { name, .. }) => name.clone(),
            Identifier::Parameter(PackageConst { name_in_pkg, .. }) => name_in_pkg.clone(),
            Identifier::ComposeLoopVar(ComposeLoopVar { name_in_pkg, .. }) => name_in_pkg.clone(),
            Identifier::GameInstanceConst(_) => todo!(),
        }
    }
}
