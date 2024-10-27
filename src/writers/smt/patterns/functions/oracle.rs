use crate::{
    expressions::Expression,
    identifier::{game_ident::GameConstIdentifier, pkg_ident::PackageConstIdentifier},
    types::Type,
    writers::smt::{
        exprs::SmtExpr,
        patterns::{
            self,
            instance_names::{encode_params, only_non_function_expression},
            oracle_args::OracleArgPattern as _,
            DatastructurePattern as _, FunctionPattern, ReturnPattern, ReturnSort,
        },
    },
};

pub const ORACLE_ARG_GAME_STATE: &str = "<game-state>";
pub const ORACLE_ARG_INTERMEDIATE_STATE: &str = "__intermediate_state";

pub struct OraclePattern<'a> {
    pub game_name: &'a str,
    pub pkg_name: &'a str,
    pub oracle_name: &'a str,
    pub oracle_args: &'a [(String, Type)],
    pub game_params: &'a [(GameConstIdentifier, Expression)],
    pub pkg_params: &'a [(PackageConstIdentifier, Expression)],
}

impl<'a> FunctionPattern for OraclePattern<'a> {
    type ReturnSort = ReturnSort<'a>;

    fn function_name(&self) -> String {
        let Self {
            game_name,
            pkg_name,
            oracle_name,
            ..
        } = self;

        let encoded_game_params = encode_params(only_non_function_expression(self.game_params));
        let encoded_pkg_params = encode_params(only_non_function_expression(self.pkg_params));

        format!("<oracle-{game_name}-{encoded_game_params}-{pkg_name}-{encoded_pkg_params}-{oracle_name}>")
    }

    fn function_args(&self) -> Vec<(String, SmtExpr)> {
        let Self {
            game_name,
            game_params,
            oracle_args,
            ..
        } = self;

        let game_state = patterns::oracle_args::GameStatePattern {
            game_name,
            game_params,
        };

        let game_consts = patterns::oracle_args::GameConstsPattern { game_name };

        let game_state_pair = (game_state.local_arg_name(), game_state.sort().into());
        let game_const_pair = (game_consts.local_arg_name(), game_consts.sort().into());

        vec![game_state_pair, game_const_pair]
            .into_iter()
            .chain(
                oracle_args
                    .iter()
                    .cloned()
                    .map(|(name, tipe)| (name, tipe.into())),
            )
            .collect()
    }

    fn function_args_count(&self) -> usize {
        // game state, game consts, args
        2 + self.oracle_args.len()
    }

    fn function_return_sort(&self) -> ReturnSort<'a> {
        let Self {
            game_name,
            pkg_name,
            oracle_name,
            game_params,
            pkg_params,
            ..
        } = self;

        ReturnPattern {
            game_name,
            pkg_name,
            oracle_name,
            game_params,
            pkg_params,
        }
        .sort()
    }
}
