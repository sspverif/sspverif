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
    use crate::{expressions::Expression, types};

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
                Declaration::Identifier(crate::identifier::Identifier::PackageIdentifier(
                    crate::identifier::pkg_ident::PackageIdentifier::Const(
                        crate::identifier::pkg_ident::PackageConstIdentifier {
                            pkg_name: "SomePkg".to_string(),
                            name: id.to_string(),
                            tipe: t.clone(),
                        },
                    ),
                )),
            )
            .expect("declare failed");

        let decl = scope.lookup(id).expect("lookup failed");

        // TODO: use this instead, once it doesn't require nightly:
        //assert_matches!(t_, Type::Type(t_), "t_ should be a real type, found {t_:?}");

        if let Declaration::Identifier(ident) = decl {
            let t_ = crate::parser::package::infer_type(&scope, &Expression::Identifier(ident));
            assert_eq!(t, t_, "lookup returned wrong type");
        } else {
            panic!("t_ should be a real type, found {:?}", decl);
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
                Declaration::Identifier(crate::identifier::Identifier::PackageIdentifier(
                    crate::identifier::pkg_ident::PackageIdentifier::Const(
                        crate::identifier::pkg_ident::PackageConstIdentifier {
                            pkg_name: "SomePkg".to_string(),
                            name: id.to_string(),
                            tipe: t.clone(),
                        },
                    ),
                )),
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
                Declaration::Identifier(crate::identifier::Identifier::PackageIdentifier(
                    crate::identifier::pkg_ident::PackageIdentifier::Const(
                        crate::identifier::pkg_ident::PackageConstIdentifier {
                            pkg_name: "SomePkg".to_string(),
                            name: id.to_string(),
                            tipe: t.clone(),
                        },
                    ),
                )),
            )
            .expect("declare id failed");

        scope.enter();
        scope
            .declare(
                id2,
                Declaration::Identifier(crate::identifier::Identifier::PackageIdentifier(
                    crate::identifier::pkg_ident::PackageIdentifier::Const(
                        crate::identifier::pkg_ident::PackageConstIdentifier {
                            pkg_name: "SomePkg".to_string(),
                            name: id2.to_string(),
                            tipe: t2.clone(),
                        },
                    ),
                )),
            )
            .expect("declare id2 failed");
        scope.leave();

        let decl = scope.lookup(id).expect("lookup failed");

        // TODO: use this instead, once it doesn't require nightly:
        //assert_matches!(t_, Type::Type(t_), "t_ should be a real type, found {t_:?}");

        if let Declaration::Identifier(ident) = decl {
            let t_ = crate::parser::package::infer_type(&scope, &Expression::Identifier(ident));
            assert_eq!(t, t_, "lookup returned wrong type");
        } else {
            panic!("t_ should be a real type, found {:?}", decl);
        }
    }
}
