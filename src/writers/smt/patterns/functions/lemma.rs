// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    expressions::Expression,
    identifier::game_ident::GameConstIdentifier,
    writers::smt::{
        names::FunctionNameBuilder,
        patterns::{FunctionPattern, GameStatePattern},
        sorts::Sort,
    },
};

struct LemmaFunction<'a> {
    left_game_inst_name: &'a str,
    left_game_name: &'a str,
    left_game_params: &'a [(GameConstIdentifier, Expression)],

    right_game_inst_name: &'a str,
    right_game_name: &'a str,
    right_game_params: &'a [(GameConstIdentifier, Expression)],

    oracle_name: &'a str,
}

impl FunctionPattern for LemmaFunction<'_> {
    fn function_name(&self) -> String {
        FunctionNameBuilder::new()
            .push("lemma-auto")
            .push(self.left_game_inst_name)
            .push(self.right_game_inst_name)
            .push(self.oracle_name)
            .build()
    }

    fn function_args(&self) -> Vec<(String, Sort)> {
        let _state_left_pattern = GameStatePattern {
            game_name: self.left_game_name,
            params: self.left_game_params,
        };
        todo!()
    }

    fn function_return_sort(&self) -> Sort {
        Sort::Bool
    }

    fn function_args_count(&self) -> usize {
        todo!()
    }
}
