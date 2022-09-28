use serde::Serialize;
use serde_derive::Deserialize;

use crate::package::Composition;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Assumption {
    pub left: String,
    pub right: String,
}

pub struct ResolvedAssumption {
    pub left: Composition,
    pub right: Composition,
}
