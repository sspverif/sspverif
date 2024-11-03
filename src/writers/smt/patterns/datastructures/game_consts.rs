use crate::{
    package::Composition,
    types::Type,
    writers::smt::{
        exprs::{SmtExpr, SmtLet},
        patterns::{DatastructurePattern, DatastructureSpec},
    },
};

#[derive(Debug, Clone)]
pub struct GameConstsPattern<'a> {
    pub game_name: &'a str,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct GameConstsSelector<'a> {
    pub(crate) name: &'a str,
    pub(crate) ty: &'a Type,
}

impl<'a> DatastructurePattern<'a> for GameConstsPattern<'a> {
    type Constructor = ();

    type Selector = GameConstsSelector<'a>;

    type DeclareInfo = Composition;

    const CAMEL_CASE: &'static str = "GameConsts";

    const KEBAB_CASE: &'static str = "game-consts";

    fn sort_name(&self) -> String {
        let camel_case = Self::CAMEL_CASE;
        let game_name = self.game_name;
        format!("<{camel_case}_{game_name}>")
    }

    fn constructor_name(&self, _cons: &Self::Constructor) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self { game_name } = self;

        format!("mk-{kebab_case}-{game_name}")
    }

    fn selector_name(&self, sel: &Self::Selector) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let const_name = sel.name;
        let Self { game_name } = self;

        format!("{kebab_case}-{game_name}-{const_name}")
    }

    fn selector_sort(&self, sel: &Self::Selector) -> crate::writers::smt::exprs::SmtExpr {
        sel.ty.into()
    }

    fn datastructure_spec(&self, info: &'a Self::DeclareInfo) -> DatastructureSpec<'a, Self> {
        let fields = info
            .consts
            .iter()
            // function parameters are just declared as smtlib functions globally, so we don't
            // want them to be part of this datatype. This way we also stay compatible with
            // solvers that don't support higher-order functions.
            .filter(|(_name, ty)| !matches!(ty, crate::types::Type::Fn(_, _)))
            .map(|(name, ty)| GameConstsSelector { name, ty })
            .collect();

        DatastructureSpec(vec![((), fields)])
    }

    fn matchfield_name(&self, sel: &Self::Selector) -> String {
        let const_name = sel.name;
        format!("<match-{const_name}>")
    }
}

pub fn bind_game_consts<Inner: Into<SmtExpr>>(
    game: &Composition,
    game_consts: &SmtExpr,
    inner: Inner,
) -> SmtLet<Inner> {
    let game_name = game.name();

    let pattern = GameConstsPattern { game_name };
    let spec = pattern.datastructure_spec(game);

    // unpack the only (constructor, selector_list) pair
    let (_, selectors) = &spec.0[0];

    SmtLet {
        bindings: selectors
            .iter()
            .map(|selector| {
                (
                    selector.name.to_string(),
                    pattern.access_unchecked(selector, game_consts.clone()),
                )
            })
            .collect(),
        body: inner,
    }
}
