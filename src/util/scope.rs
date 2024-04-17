use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::OracleSig;
use crate::parser::package::ForComp;
use crate::parser::package::ForSpec;
use crate::types;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ValidityContext {
    Package,
    Game,
    Proof,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum OracleContext {
    Package {
        pkg_name: String,
    },

    PackageInstance {
        pkg_name: String,
        pkg_inst_name: String,
        game_name: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Declaration {
    Identifier(Identifier),
    Oracle(OracleContext, OracleSig),
    // old stuff below
    CompositionConst {
        tipe: Type,
        game_name: String,
    },

    CompositionForSpec {
        game_name: String,
        start: Expression,
        end: Expression,
        start_comp: ForComp,
        end_comp: ForComp,
    },

    PackageConst {
        tipe: Type,
        pkg_name: String,
    },
    PackageState {
        tipe: Type,
        pkg_name: String,
    },

    PackageOracleArg {
        tipe: Type,
        pkg_name: String,
        oracle_name: String,
    },

    PackageOracleLocal {
        tipe: Type,
        pkg_name: String,
        oracle_name: String,
    },

    PackageOracleImport {
        pkg_name: String,
        index: Vec<Expression>,
        index_forspecs: Vec<ForSpec>,
        args: Vec<types::Type>,
        out: types::Type,
    },

    PackageOracleImportsForSpec {
        pkg_name: String,
        start: Expression,
        end: Expression,
        start_comp: ForComp,
        end_comp: ForComp,
    },
}

impl Declaration {
    pub fn validity_context(&self) -> ValidityContext {
        match self {
            Declaration::Oracle(_, _) => ValidityContext::Package,
            Declaration::Identifier(ident) => match ident {
                Identifier::PackageIdentifier(_) => ValidityContext::Package,
                Identifier::PackageInstanceIdentifier(_) => ValidityContext::Package,
                Identifier::GameIdentifier(_) => ValidityContext::Game,
                _ => {
                    panic!("found old-style identifier, should not be used")
                }
            },
            Declaration::CompositionConst { .. } | Declaration::CompositionForSpec { .. } => {
                ValidityContext::Game
            }
            Declaration::PackageConst { .. }
            | Declaration::PackageState { .. }
            | Declaration::PackageOracleArg { .. }
            | Declaration::PackageOracleLocal { .. }
            | Declaration::PackageOracleImport { .. }
            | Declaration::PackageOracleImportsForSpec { .. } => ValidityContext::Package,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    AlreadyDefined(String, Declaration),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyDefined(name, declaration) => write!(
                f,
                "identifier `{name}` already declared with {declaration:?}"
            ),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
pub struct Scope {
    entries: Vec<HashMap<String, Declaration>>,
    types: HashSet<Declaration>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            entries: vec![],
            types: HashSet::new(),
        }
    }

    pub fn enter(&mut self) {
        self.entries.push(HashMap::new())
    }

    pub fn leave(&mut self) {
        if !self.entries.is_empty() {
            self.entries.pop();
        } else {
            panic!("scope leave: scope stack is empty");
        }
    }

    pub fn known_types(&self) -> HashSet<Declaration> {
        self.types.clone()
    }

    pub fn declare_oracle(
        &mut self,
        id: &str,
        pkg_name: String,
        arg_types: Vec<types::Type>,
        ret_type: types::Type,
    ) -> Result<(), Error> {
        self.declare(
            id,
            Declaration::PackageOracleImport {
                pkg_name,
                index: vec![],
                index_forspecs: vec![],
                args: arg_types,
                out: ret_type,
            },
        )
    }

    /* Error conditions:
     *  - No scope at all
     *  - Identifier exists somewhere in the scope tower already
     */
    pub fn declare(&mut self, id: &str, t: Declaration) -> Result<(), Error> {
        let scope_type: Declaration = t.into();
        self.types.insert(scope_type.clone());
        if self.lookup(id) == None {
            if let Some(last) = self.entries.last_mut() {
                last.insert(id.to_string(), scope_type.clone());
                Ok(())
            } else {
                panic!("scope declare: scope stack is empty");
            }
        } else {
            Err(Error::AlreadyDefined(id.to_string(), scope_type)) // already defined
        }
    }

    pub fn lookup(&self, id: &str) -> Option<Declaration> {
        /* Only needed for debug printing
        match &id {
            Identifier::Local(name)
            | Identifier::Scalar(name)
            | Identifier::State { name, .. }
            | Identifier::Params {
                name_in_pkg: name, ..
            } => {
                let print_names: Vec<&str> = vec![];
                let do_print = print_names.iter().any(|print_name| name == print_name);

                if do_print {
                    eprintln!("DEBUG Scope lookup {id:?}");
                }
            }
        }
        */

        for table in self.entries.clone().into_iter().rev() {
            if let Some(t) = table.get(id) {
                return Some(t.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::Declaration;
    use crate::types;

    use super::Scope;

    /* Properties:
    - (enter then) only lookup -> fails trivially (not tested)
    - (enter then) declare then lookup -> success
    - access variable that was declared inside a block after leaving -> fails
    - access varable that was declared, then enter and leave -> success
    */

    #[test]
    fn declare_then_lookup_succeeds() {
        let id = "test_id";
        let t = types::Type::Integer;

        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(
                id,
                Declaration::PackageConst {
                    tipe: t.clone(),
                    pkg_name: "SomePkg".to_string(),
                },
            )
            .expect("declare failed");
        let t_ = scope.lookup(&id).expect("lookup failed");

        // TODO: use this instead, once it doesn't require nightly:
        //assert_matches!(t_, Type::Type(t_), "t_ should be a real type, found {t_:?}");

        if let Declaration::PackageConst { tipe: t_, .. } = t_ {
            assert_eq!(t, t_, "lookup returned wrong type");
        } else {
            panic!("t_ should be a real type, found {:?}", t_);
        }
    }

    #[test]
    fn gone_after_leave() {
        let id = "test_id";
        let t = types::Type::Integer;

        let mut scope = Scope::new();
        scope.enter();
        scope.enter();
        scope
            .declare(
                id,
                Declaration::PackageConst {
                    tipe: t,
                    pkg_name: "SomePkg".to_string(),
                },
            )
            .expect("declare failed");
        scope.leave();

        assert_eq!(None, scope.lookup(id));
    }

    #[test]
    fn still_there_after_enter_and_leave() {
        let id = "test_id";
        let id2 = "test_id2";
        let t = types::Type::Integer;
        let t2 = types::Type::String;

        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(
                id,
                Declaration::PackageConst {
                    tipe: t.clone(),
                    pkg_name: "SomePkg".to_string(),
                },
            )
            .expect("declare id failed");

        scope.enter();
        scope
            .declare(
                id2,
                Declaration::PackageConst {
                    tipe: t2.clone(),
                    pkg_name: "SomePkg".to_string(),
                },
            )
            .expect("declare id2 failed");
        scope.leave();

        let t_ = scope.lookup(id).expect("lookup failed");

        // TODO: use this instead, once it doesn't require nightly:
        //assert_matches!(t_, Type::Type(t_), "t_ should be a real type, found {t_:?}");

        if let Declaration::PackageConst { tipe: t_, .. } = t_ {
            assert_eq!(t, t_, "lookup returned wrong type");
        } else {
            panic!("t_ should be a real type, found {:?}", t_);
        }
    }
}
