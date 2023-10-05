use super::{DatastructurePattern, DatastructureSpec};
use crate::types::Type;
use crate::writers::smt::{exprs::SmtExpr, sorts::SmtPlainSort};

pub struct ReturnPattern<'a> {
    pub game_name: &'a str,
    pub pkg_inst_name: &'a str,
    pub oracle_name: &'a str,
}

pub struct ReturnSort<'a> {
    pub game_name: &'a str,
    pub pkg_inst_name: &'a str,
    pub oracle_name: &'a str,
}

use crate::impl_Into_for_PlainSort;
impl_Into_for_PlainSort!('a, ReturnSort<'a>);

impl<'a> SmtPlainSort for ReturnSort<'a> {
    fn sort_name(&self) -> String {
        let camel_case = ReturnPattern::CAMEL_CASE;
        let Self {
            game_name,
            pkg_inst_name,
            oracle_name,
        } = self;

        format!("{camel_case}-{game_name}-{pkg_inst_name}-{oracle_name}")
    }
}

#[derive(PartialEq, Eq)]
pub struct ReturnConstructor;

#[derive(PartialEq, Eq)]
pub enum ReturnSelector<'a> {
    GameState,
    ReturnValueOrAbort { return_type: &'a Type },
}

impl<'a> DatastructurePattern<'a> for ReturnPattern<'a> {
    type Constructor = ReturnConstructor;
    type Selector = ReturnSelector<'a>;
    type DeclareInfo = &'a Type;
    type Sort = ReturnSort<'a>;

    const CAMEL_CASE: &'static str = "Return";
    const KEBAB_CASE: &'static str = "return";

    fn sort(&self) -> ReturnSort<'a> {
        let ReturnPattern {
            game_name,
            pkg_inst_name,
            oracle_name,
        } = self;
        ReturnSort {
            game_name,
            pkg_inst_name,
            oracle_name,
        }
    }

    fn constructor_name(&self, _cons: &Self::Constructor) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self {
            game_name,
            pkg_inst_name,
            oracle_name,
        } = self;

        format!("mk-{kebab_case}-{game_name}-{pkg_inst_name}-{oracle_name}")
    }

    fn selector_name(&self, sel: &Self::Selector) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self {
            game_name,
            pkg_inst_name,
            oracle_name,
        } = self;

        let field_name = match sel {
            ReturnSelector::GameState => "game-state",
            ReturnSelector::ReturnValueOrAbort { .. } => "return-value-or-abort",
        };

        format!("{kebab_case}-{game_name}-{pkg_inst_name}-{oracle_name}-{field_name}")
    }

    fn matchfield_name(&self, sel: &Self::Selector) -> String {
        let field_name = match sel {
            ReturnSelector::GameState => "game-state",
            ReturnSelector::ReturnValueOrAbort { .. } => "return-value-or-abort",
        };

        format!("match-{field_name}")
    }

    fn datastructure_spec(&self, return_type: &&'a Type) -> DatastructureSpec<'a, Self> {
        DatastructureSpec(vec![(
            ReturnConstructor,
            vec![
                ReturnSelector::GameState,
                ReturnSelector::ReturnValueOrAbort { return_type },
            ],
        )])
    }

    fn selector_sort(&self, sel: &Self::Selector) -> SmtExpr {
        let Self { game_name, .. } = self;

        let game_state_pattern = super::game_state::GameStatePattern { game_name };

        match sel {
            ReturnSelector::GameState => game_state_pattern.sort().sort_name().into(),
            ReturnSelector::ReturnValueOrAbort { return_type } => {
                ("ReturnValue", *return_type).into()
            }
        }
    }
}
