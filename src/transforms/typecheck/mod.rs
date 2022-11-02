mod codeblock;
mod composition;
mod errors;
mod expression;
mod oracledef;
mod pkg;
mod scope;

use crate::types;

use composition::typecheck_comp;
pub use errors::TypeCheckError;
pub use scope::Scope;

use crate::package::Composition;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    Type(types::Type),
    Oracle(Vec<types::Type>, types::Type), // arg types, return type
}

impl From<types::Type> for Type {
    fn from(value: types::Type) -> Self {
        Type::Type(value)
    }
}

pub struct Transformation {
    scope: Scope,
    comp: Composition,
}

impl Transformation {
    pub fn new(scope: Scope, comp: Composition) -> Transformation {
        Transformation { scope, comp }
    }

    pub fn new_with_empty_scope(comp: Composition) -> Transformation {
        Transformation::new(Scope::new(), comp)
    }

    pub fn scope(self) -> Scope {
        self.scope
    }
}

impl super::Transformation for Transformation {
    type Err = TypeCheckError;
    type Aux = Scope;
    fn transform(&self) -> Result<(Composition, Scope), TypeCheckError> {
        let mut scope = self.scope.clone();
        let typed_comp = typecheck_comp(&self.comp, &mut scope)?;

        Ok((typed_comp, scope))
    }
}
