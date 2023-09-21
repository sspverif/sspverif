use super::DatastructurePattern2;
use crate::writers::smt::{declare::declare_datatype, exprs::SmtExpr};

pub struct PartialReturnPattern<'a> {
    pub game_name: &'a str,
    pub pkg_inst_name: &'a str,
    pub oracle_name: &'a str,
}

pub enum PartialReturnConstructor {
    Return,
    Abort,
}

pub enum PartialReturnSelector {
    GameState,
    IntermediateState,
}

impl<'a> DatastructurePattern2 for PartialReturnPattern<'a> {
    type Constructor = PartialReturnConstructor;
    type Selector = PartialReturnSelector;
    type DeclareInfo = ();

    const CAMEL_CASE: &'static str = "PartialReturn";
    const KEBAB_CASE: &'static str = "partial-return";

    fn sort_name(&self) -> String {
        let camel_case = Self::CAMEL_CASE;
        let Self {
            game_name,
            pkg_inst_name,
            oracle_name,
        } = self;

        format!("{camel_case}-{game_name}-{pkg_inst_name}-{oracle_name}")
    }

    fn constructor_name(&self, cons: &Self::Constructor) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self {
            game_name,
            pkg_inst_name,
            oracle_name,
        } = self;

        let cons_name = match cons {
            PartialReturnConstructor::Return => kebab_case,
            PartialReturnConstructor::Abort => "partial-abort",
        };

        format!("mk-{cons_name}-{game_name}-{pkg_inst_name}-{oracle_name}")
    }

    fn selector_name(&self, sel: &Self::Selector) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self {
            game_name,
            pkg_inst_name,
            oracle_name,
        } = self;

        let field_name = match sel {
            PartialReturnSelector::GameState => "game-state",
            PartialReturnSelector::IntermediateState => "intermediate-state",
        };

        format!("{kebab_case}-{game_name}-{pkg_inst_name}-{oracle_name}-{field_name}")
    }

    fn declare_datatype(&self, _info: &Self::DeclareInfo) -> SmtExpr {
        let Self {
            game_name,
            pkg_inst_name,
            oracle_name,
        } = self;
        let game_state_pattern = super::DatastructurePattern::GameState { game_name };

        let intermediate_state_pattern = super::IntermediateStatePattern {
            game_name,
            pkg_inst_name,
            oracle_name,
        };

        let constructors = vec![
            (
                self.constructor_name(&PartialReturnConstructor::Return),
                vec![
                    (
                        self.selector_name(&PartialReturnSelector::GameState),
                        game_state_pattern.sort_name().into(),
                    ),
                    (
                        self.selector_name(&PartialReturnSelector::IntermediateState),
                        intermediate_state_pattern.sort_name().into(),
                    ),
                ],
            ),
            (
                self.constructor_name(&PartialReturnConstructor::Abort),
                vec![].into(),
            ),
        ];

        declare_datatype(&self.sort_name(), constructors.into_iter())
    }

    fn selector_sort(&self, sel: &Self::Selector) -> SmtExpr {
        let Self {
            game_name,
            pkg_inst_name,
            oracle_name,
        } = self;

        let game_state_pattern = super::DatastructurePattern::GameState { game_name };
        let intermediate_state_pattern = super::IntermediateStatePattern {
            game_name,
            pkg_inst_name,
            oracle_name,
        };

        match sel {
            PartialReturnSelector::GameState => game_state_pattern.sort_name(),
            PartialReturnSelector::IntermediateState => intermediate_state_pattern.sort_name(),
        }
        .into()
    }
}
