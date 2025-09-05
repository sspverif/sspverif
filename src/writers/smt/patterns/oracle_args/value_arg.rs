// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::types::Type;

use super::*;

pub struct ValueArgPattern<'a> {
    pub pkg_inst_name: &'a str,
    pub oracle_name: &'a str,
    pub arg_name: &'a str,
    pub arg_ty: &'a Type,
}

impl OracleArgPattern for ValueArgPattern<'_> {
    type Variant = ();

    fn global_const_name(&self, game_inst_name: &str, _variant: &()) -> String {
        let Self {
            pkg_inst_name,
            oracle_name,
            arg_name,
            ..
        } = self;
        format!("<<arg-{game_inst_name}-{pkg_inst_name}-{oracle_name}-{arg_name}>>")
    }

    fn local_arg_name(&self) -> String {
        self.arg_name.to_string()
    }

    fn sort(&self) -> Sort {
        self.arg_ty.clone().into()
    }
}
