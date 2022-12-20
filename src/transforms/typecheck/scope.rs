use super::errors::ScopeError;
use super::Type;
use crate::identifier::Identifier;
use crate::types;
use std::collections::{HashMap, HashSet};

// TODO
#[derive(Debug, Clone)]
pub struct Scope {
    entries: Vec<HashMap<Identifier, Type>>,
    types: HashSet<Type>,
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

    pub fn known_types(&self) -> HashSet<Type> {
        self.types.clone()
    }

    pub fn declare_oracle(
        &mut self,
        id: Identifier,
        arg_types: Vec<types::Type>,
        ret_type: types::Type,
    ) -> Result<(), ScopeError> {
        self.declare(id, Type::Oracle(arg_types, ret_type))
    }

    /* Error conditions:
     *  - No scope at all
     *  - Identifier exists somewhere in the scope tower already
     */
    pub fn declare<I: Into<Type> + Clone>(
        &mut self,
        id: Identifier,
        t: I,
    ) -> Result<(), ScopeError> {
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
                    eprintln!("DEBUG Scope declare {id:?} {t:?}");
                }
            }
        }
        */

        self.types.insert(t.clone().into());
        //let bt = Backtrace::capture();
        //println!("declaring: {:?} {:?} {}", id, t, bt);
        if self.lookup(&id) == None {
            if let Some(last) = self.entries.last_mut() {
                last.insert(id, t.into());
                Ok(())
            } else {
                panic!("scope declare: scope stack is empty");
            }
        } else {
            Err(ScopeError) // already defined
        }
    }

    pub fn lookup(&self, id: &Identifier) -> Option<Type> {
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
    use super::super::Type;
    use crate::identifier::Identifier;
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
        let id = Identifier::Local("test_id".to_string());
        let t = types::Type::Integer;

        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(id.clone(), t.clone())
            .expect("declare failed");
        let t_ = scope.lookup(&id).expect("lookup failed");

        // TODO: use this instead, once it doesn't require nightly:
        //assert_matches!(t_, Type::Type(t_), "t_ should be a real type, found {t_:?}");

        if let Type::Type(t_) = t_ {
            assert_eq!(t, t_, "lookup returned wrong type");
        } else {
            panic!("t_ should be a real type, found {:?}", t_);
        }
    }

    #[test]
    fn gone_after_leave() {
        let id = Identifier::Local("test_id".to_string());
        let t = types::Type::Integer;

        let mut scope = Scope::new();
        scope.enter();
        scope.enter();
        scope.declare(id.clone(), t).expect("declare failed");
        scope.leave();

        assert_eq!(None, scope.lookup(&id));
    }

    #[test]
    fn still_there_after_enter_and_leave() {
        let id = Identifier::Local("test_id".to_string());
        let id2 = Identifier::Local("test_id2".to_string());
        let t = types::Type::Integer;
        let t2 = types::Type::String;

        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(id.clone(), t.clone())
            .expect("declare id failed");

        scope.enter();
        scope.declare(id2, t2).expect("declare id2 failed");
        scope.leave();

        let t_ = scope.lookup(&id).expect("lookup failed");

        // TODO: use this instead, once it doesn't require nightly:
        //assert_matches!(t_, Type::Type(t_), "t_ should be a real type, found {t_:?}");

        if let Type::Type(t_) = t_ {
            assert_eq!(t, t_, "lookup returned wrong type");
        } else {
            panic!("t_ should be a real type, found {:?}", t_);
        }
    }
}
