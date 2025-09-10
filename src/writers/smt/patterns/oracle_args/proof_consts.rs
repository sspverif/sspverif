// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::writers::smt::patterns::{
    proof_consts::ProofConstsPattern as ProofConstsDatatypePattern, DatastructurePattern as _,
};

use super::*;

pub struct ProofConstsPattern<'a> {
    pub proof_name: &'a str,
}

impl OracleArgPattern for ProofConstsPattern<'_> {
    type Variant = ();

    fn global_const_name(&self, _game_inst_name: &str, _variant: &()) -> String {
        "<<proof-consts>>".to_string()
    }

    fn local_arg_name(&self) -> String {
        "<proof-consts>".to_string()
    }

    fn sort(&self) -> Sort {
        ProofConstsDatatypePattern {
            proof_name: self.proof_name,
        }
        .sort(vec![])
    }
}
