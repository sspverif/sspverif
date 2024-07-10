use crate::types::Type;
use crate::writers::smt::exprs::SmtExpr;
use crate::writers::smt::patterns::ReturnPattern;
use crate::writers::smt::sorts::{SmtBool, SmtSort};
use crate::{package::OracleSig, split::SplitPath};

use super::{
    DatastructurePattern, GameStatePattern, GlobalStatePattern, IntermediateStatePattern,
    PartialReturnPattern, PartialReturnSort, ReturnSort,
};

pub trait FunctionPattern {
    type ReturnSort: SmtSort;

    fn function_name(&self) -> String;
    fn function_args(&self) -> Vec<(String, SmtExpr)>;
    fn function_return_sort(&self) -> Self::ReturnSort;

    fn define_fun<B: Into<SmtExpr>>(&self, body: B) -> SmtExpr {
        (
            "define-fun",
            self.function_name(),
            SmtExpr::List(
                self.function_args()
                    .into_iter()
                    .map(|pair| -> SmtExpr { pair.into() })
                    .collect(),
            ),
            self.function_return_sort(),
            body,
        )
            .into()
    }

    fn define_fun_rec<B: Into<SmtExpr>>(&self, body: B) -> SmtExpr {
        (
            "define-fun-rec",
            self.function_name(),
            SmtExpr::List(
                self.function_args()
                    .into_iter()
                    .map(|pair| -> SmtExpr { pair.into() })
                    .collect(),
            ),
            self.function_return_sort(),
            body,
        )
            .into()
    }

    fn call(&self, args: &[SmtExpr]) -> SmtExpr {
        let mut call: Vec<SmtExpr> = vec![self.function_name().into()];
        call.extend(args.iter().map(|arg| arg.clone()));
        SmtExpr::List(call)
    }
}

pub const ORACLE_ARG_GAME_STATE: &str = "__global_state";
pub const ORACLE_ARG_INTERMEDIATE_STATE: &str = "__intermediate_state";

pub struct OraclePattern<'a> {
    pub game_inst_name: &'a str,
    pub pkg_inst_name: &'a str,
    pub oracle_name: &'a str,
    pub oracle_args: &'a [(String, Type)],
}

impl<'a> FunctionPattern for OraclePattern<'a> {
    type ReturnSort = ReturnSort<'a>;

    fn function_name(&self) -> String {
        let Self {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
            ..
        } = self;

        format!("oracle-{game_inst_name}-{pkg_inst_name}-{oracle_name}")
    }

    fn function_args(&self) -> Vec<(String, SmtExpr)> {
        let Self {
            game_inst_name,
            oracle_args,
            ..
        } = self;

        let game_state_pattern = GameStatePattern { game_inst_name };

        use super::VariablePattern;
        let global_state_pattern = &GlobalStatePattern;
        let mut args: Vec<(String, SmtExpr)> =
            vec![global_state_pattern.name_sort_tuple(&game_state_pattern)];

        args.extend(
            oracle_args
                .iter()
                .cloned()
                .map(|(name, tipe)| (name, tipe.into())),
        );

        args
    }

    fn function_return_sort(&self) -> ReturnSort<'a> {
        let Self {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
            ..
        } = self;

        ReturnPattern {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
        }
        .sort()
    }
}

pub struct DispatchOraclePattern<'a> {
    pub game_inst_name: &'a str,
    pub pkg_inst_name: &'a str,
    pub oracle_sig: &'a OracleSig,
}

impl<'a> FunctionPattern for DispatchOraclePattern<'a> {
    type ReturnSort = PartialReturnSort<'a>;
    fn function_name(&self) -> String {
        let Self {
            game_inst_name,
            pkg_inst_name,
            oracle_sig,
        } = self;

        let oracle_name = &oracle_sig.name;
        format!("oracle-{game_inst_name}-{pkg_inst_name}-{oracle_name}")
    }

    fn function_args(&self) -> Vec<(String, SmtExpr)> {
        let DispatchOraclePattern {
            oracle_sig,
            game_inst_name,
            pkg_inst_name,
        } = self;

        let oracle_name = &oracle_sig.name;
        let game_state_pattern = GameStatePattern { game_inst_name };
        let intermediate_state_pattern = IntermediateStatePattern {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
        };

        let mut args = vec![
            (
                ORACLE_ARG_GAME_STATE.to_string(),
                game_state_pattern.sort().into(),
            ),
            (
                ORACLE_ARG_INTERMEDIATE_STATE.to_string(),
                intermediate_state_pattern.sort().into(),
            ),
        ];

        args.extend(
            oracle_sig
                .args
                .iter()
                .cloned()
                .map(|(name, tipe)| (name, tipe.into())),
        );

        args
    }

    fn function_return_sort(&self) -> PartialReturnSort<'a> {
        let Self {
            game_inst_name,
            pkg_inst_name,
            oracle_sig,
        } = self;

        let partial_return_pattern = PartialReturnPattern {
            game_inst_name,
            pkg_inst_name,
            oracle_name: &oracle_sig.name,
        };

        partial_return_pattern.sort()
    }
}

pub struct PartialOraclePattern<'a> {
    pub game_inst_name: &'a str,
    pub pkg_inst_name: &'a str,
    pub oracle_name: &'a str,
    pub split_path: &'a SplitPath,
}

impl<'a> FunctionPattern for PartialOraclePattern<'a> {
    type ReturnSort = PartialReturnSort<'a>;

    fn function_name(&self) -> String {
        let PartialOraclePattern {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
            split_path,
        } = self;

        let path = split_path.smt_name();
        format!("oracle-{game_inst_name}-{pkg_inst_name}-{oracle_name}-{path}")
    }

    fn function_args(&self) -> Vec<(String, SmtExpr)> {
        todo!()
    }

    fn function_return_sort(&self) -> PartialReturnSort<'a> {
        let Self {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
            ..
        } = self;
        let partial_return_pattern = PartialReturnPattern {
            game_inst_name,
            pkg_inst_name,
            oracle_name,
        };

        partial_return_pattern.sort()
    }
}

pub struct LemmaFunction<'a> {
    left_game_inst_name: &'a str,
    right_game_inst_name: &'a str,
    oracle_name: &'a str,
}

impl<'a> FunctionPattern for LemmaFunction<'a> {
    type ReturnSort = SmtBool;

    fn function_name(&self) -> String {
        let Self {
            left_game_inst_name,
            right_game_inst_name,
            oracle_name,
        } = self;
        format!("lemma-auto-{left_game_inst_name}-{right_game_inst_name}-{oracle_name}")
    }

    fn function_args(&self) -> Vec<(String, SmtExpr)> {
        let _state_left_pattern = GameStatePattern {
            game_inst_name: self.left_game_inst_name,
        };
        vec![]
    }

    fn function_return_sort(&self) -> Self::ReturnSort {
        todo!()
    }
}
