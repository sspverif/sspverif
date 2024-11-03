use crate::{
    expressions::Expression,
    identifier::game_ident::GameConstIdentifier,
    proof::GameInstance,
    transforms::samplify::SampleInfo,
    types::Type,
    writers::smt::{
        exprs::SmtExpr,
        patterns::{
            instance_names::{encode_params, only_non_function_expression},
            DatastructurePattern, DatastructureSpec, PackageStatePattern,
        },
        sorts::Sort,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameStatePattern<'a> {
    pub game_name: &'a str,
    pub params: &'a [(GameConstIdentifier, Expression)],
}

#[derive(Debug, PartialEq, Eq)]
pub enum GameStateSelector<'a> {
    PackageInstance { pkg_inst_name: &'a str, sort: Sort },
    Randomness { sample_id: usize },
}

pub struct GameStateDeclareInfo<'a> {
    pub(crate) game_inst: &'a GameInstance,
    pub sample_info: &'a SampleInfo,
}

impl<'a> DatastructurePattern<'a> for GameStatePattern<'a> {
    type Constructor = ();
    type Selector = GameStateSelector<'a>;
    type DeclareInfo = GameStateDeclareInfo<'a>;

    const CAMEL_CASE: &'static str = "GameState";
    const KEBAB_CASE: &'static str = "game";

    fn sort_name(&self) -> String {
        let Self { game_name, params } = self;
        let camel_case = <GameStatePattern as DatastructurePattern>::CAMEL_CASE;
        let encoded_params = encode_params(only_non_function_expression(*params));

        format!("<{camel_case}_{game_name}_{encoded_params}>")
    }

    fn constructor_name(&self, _cons: &Self::Constructor) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self { game_name, .. } = self;
        let encoded_params = encode_params(only_non_function_expression(self.params));

        format!("<mk-{kebab_case}-{game_name}-{encoded_params}>")
    }

    fn selector_name(&self, sel: &Self::Selector) -> String {
        let kebab_case = Self::KEBAB_CASE;
        let Self { game_name, .. } = self;
        let encoded_params = encode_params(only_non_function_expression(self.params));

        let (kind_name, field_name) = match sel {
            GameStateSelector::PackageInstance { pkg_inst_name, .. } => {
                ("pkgstate", pkg_inst_name.to_string())
            }
            GameStateSelector::Randomness { sample_id } => ("rand", format!("{sample_id}")),
        };

        format!("<{kebab_case}-{game_name}-{encoded_params}-{kind_name}-{field_name}>")
    }

    fn matchfield_name(&self, sel: &Self::Selector) -> String {
        let (kind_name, field_name) = match sel {
            GameStateSelector::PackageInstance { pkg_inst_name, .. } => {
                ("pkgstate", pkg_inst_name.to_string())
            }
            GameStateSelector::Randomness { sample_id } => ("rand", format!("{sample_id}")),
        };

        format!("match-{kind_name}-{field_name}")
    }

    fn selector_sort(&self, sel: &Self::Selector) -> SmtExpr {
        match sel {
            GameStateSelector::PackageInstance { sort, .. } => sort.clone().into(),
            GameStateSelector::Randomness { .. } => Type::Integer.into(),
        }
    }

    fn datastructure_spec(&self, info: &Self::DeclareInfo) -> DatastructureSpec<'a, Self> {
        let Self { game_name, .. } = self;

        debug_assert_eq!(game_name, &info.game_inst.game_name());

        let pkgstate_selectors = info.game_inst.game().pkgs.iter().map(|inst| {
            let params = &inst.params;
            let pkg_name = &inst.pkg.name;

            let sort = PackageStatePattern { pkg_name, params }.sort(vec![]);

            GameStateSelector::PackageInstance {
                pkg_inst_name: &inst.name,
                sort,
            }
        });

        let rand_selectors = (0..info.sample_info.count)
            .map(|sample_id| GameStateSelector::Randomness { sample_id });

        let fields = pkgstate_selectors.chain(rand_selectors).collect();

        DatastructureSpec(vec![((), fields)])
    }
}
