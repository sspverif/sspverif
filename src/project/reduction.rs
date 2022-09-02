use serde_derive::{Deserialize, Serialize};

// TODO: add a HybridArgument variant
#[derive(Debug, Serialize, Deserialize)]
pub struct Reduction {
    pub left: String,
    pub right: String,
    pub assumption: String,
    // we probably have to provide more information here,
    // in order to easily figure out how to perform the rewrite
}
