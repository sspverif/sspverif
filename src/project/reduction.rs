use serde_derive::{Deserialize, Serialize};

use crate::package::Composition;
use crate::project::{Assumption, Result};

// TODO: add a HybridArgument variant
#[derive(Debug, Serialize, Deserialize)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    Unspecified,
}

// TODO: add a HybridArgument variant
#[derive(Debug, Serialize, Deserialize)]
pub struct Reduction {
    pub left: String,
    pub right: String,
    pub direction: Direction,
    pub assumption: String,
    // we probably have to provide more information here,
    // in order to easily figure out how to perform the rewrite
}

pub struct ResolvedReduction {
    pub left: Composition,
    pub right: Composition,
    pub assumption: Assumption,
    pub assumption_name: String,
}

impl ResolvedReduction {
    pub fn prove(&self) -> Result<()> {
        todo!("implement reduction proofs  next time");
    }
}
