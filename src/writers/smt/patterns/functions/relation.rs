use crate::{
    types::Type,
    writers::smt::{
        patterns::{DatastructurePattern as _, GameStatePattern, ReturnPattern},
        sorts::Sort,
    },
};

use super::FunctionPattern;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Relation<'a> {
    pub(crate) game_inst_name_left: &'a str,
    pub(crate) game_inst_name_right: &'a str,
    pub(crate) relation_name: &'a str,
    pub(crate) oracle_name: &'a str,
    pub(crate) state_datatype_left: GameStatePattern<'a>,
    pub(crate) state_datatype_right: GameStatePattern<'a>,
    pub(crate) return_datatype_left: ReturnPattern<'a>,
    pub(crate) return_datatype_right: ReturnPattern<'a>,
    pub(crate) args: &'a [(String, Type)],
    pub(crate) return_type: &'a Type,
}

impl<'a> Relation<'a> {
    pub(crate) fn arg_old_state_left(&self) -> (&'static str, Sort) {
        ("old-state-left", self.state_datatype_left.sort(vec![]))
    }
    pub(crate) fn arg_old_state_right(&self) -> (&'static str, Sort) {
        ("old-state-right", self.state_datatype_right.sort(vec![]))
    }
    pub(crate) fn arg_return_left(&self) -> (&'static str, Sort) {
        ("return-left", self.return_datatype_left.sort(vec![]))
    }
    pub(crate) fn arg_return_right(&self) -> (&'static str, Sort) {
        ("return-right", self.return_datatype_right.sort(vec![]))
    }
}

impl<'a> FunctionPattern for Relation<'a> {
    fn function_name(&self) -> String {
        format!(
            "<relation-{}-{}-{}-{}>",
            self.relation_name,
            self.game_inst_name_left,
            self.game_inst_name_right,
            self.oracle_name
        )
    }

    fn function_args(&self) -> Vec<(String, Sort)> {
        vec![
            self.arg_old_state_left(),
            self.arg_old_state_right(),
            self.arg_return_left(),
            self.arg_return_right(),
        ]
        .into_iter()
        .map(|(name, sort)| (name.to_string(), sort))
        .chain(
            self.args
                .iter()
                .map(|(name, ty)| (name.clone(), ty.clone().into())),
        )
        .collect()
    }

    fn function_args_count(&self) -> usize {
        4 + self.args.len()
    }

    fn function_return_sort(&self) -> Sort {
        Sort::Bool
    }
}
