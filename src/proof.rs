use crate::{
    expressions::Expression,
    gamehops::{reduction::Assumption, GameHop},
    identifier::game_ident::GameConstIdentifier,
    package::{Composition, Package},
    packageinstance::instantiate::InstantiationContext,
    types::Type,
    ttheorem::TTheorem,
};

////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub(crate) struct GameInstance {
    pub(crate) name: String,
    pub(crate) game: Composition,
    pub(crate) types: Vec<(String, Type)>,
    pub(crate) consts: Vec<(GameConstIdentifier, Expression)>,
}

mod instantiate {
    use crate::{
        package::Package,
        packageinstance::{instantiate::InstantiationContext, PackageInstance},
    };

    /*
    *
    *This function looks funny.
    It is doing working during a game-to-gameinstance rewrite,
    but does things for a pacakge-to-package instance rewrite.
    *
    * */
    pub(crate) fn rewrite_pkg_inst(
        inst_ctx: InstantiationContext,
        pkg_inst: &PackageInstance,
    ) -> PackageInstance {
        let mut pkg_inst = pkg_inst.clone();

        let new_oracles = pkg_inst
            .pkg
            .oracles
            .iter()
            .map(|oracle_def| inst_ctx.rewrite_oracle_def(oracle_def.clone()))
            .collect();

        // let new_split_oracles = pkg_inst
        //     .pkg
        //     .split_oracles
        //     .iter()
        //     .map(|split_oracle_def| inst_ctx.rewrite_split_oracle_def(split_oracle_def.clone()))
        //     .collect();

        let new_state = pkg_inst
            .pkg
            .state
            .iter()
            .cloned()
            .map(|(ident, ty, span)| (ident, inst_ctx.rewrite_type(ty), span))
            .collect();

        let new_params = pkg_inst
            .pkg
            .params
            .iter()
            .cloned()
            .map(|(ident, ty, span)| (ident, inst_ctx.rewrite_type(ty), span))
            .collect();

        let pkg = Package {
            oracles: new_oracles,
            state: new_state,
            params: new_params,
            // split_oracles: new_split_oracles,
            ..pkg_inst.pkg.clone()
        };

        for (_, expr) in &mut pkg_inst.params {
            *expr = inst_ctx.rewrite_expression(expr)
        }

        for index in &mut pkg_inst.multi_instance_indices.indices {
            *index = inst_ctx.rewrite_expression(index);
        }

        let new_params = pkg_inst
            .params
            .iter()
            .map(|(ident, expr)| {
                (
                    inst_ctx
                        .rewrite_pkg_identifier(
                            crate::identifier::pkg_ident::PackageIdentifier::Const(ident.clone()),
                        )
                        .into_const()
                        .unwrap(),
                    inst_ctx.rewrite_expression(expr),
                )
            })
            .collect();

        PackageInstance {
            pkg,
            params: new_params,
            ..pkg_inst
        }
    }
}

impl GameInstance {
    pub(crate) fn new(
        game_inst_name: String,
        proof_name: String,
        game: Composition,
        types: Vec<(String, Type)>,
        params: Vec<(GameConstIdentifier, Expression)>,
    ) -> GameInstance {
        let inst_ctx: InstantiationContext = InstantiationContext::new_game_instantiation_context(
            &game_inst_name,
            &proof_name,
            &params,
            &types,
        );

        let new_pkg_instances = game
            .pkgs
            .iter()
            .map(|pkg_inst| -> crate::package::PackageInstance {
                instantiate::rewrite_pkg_inst(inst_ctx, pkg_inst)
            })
            .collect();

        let resolved_params = game
            .consts
            .iter()
            .map(|(ident, ty)| (ident.clone(), inst_ctx.rewrite_type(ty.clone())))
            .collect();

        let game = Composition {
            name: game.name.clone(),
            pkgs: new_pkg_instances,
            consts: resolved_params,

            ..game
        };

        GameInstance {
            name: game_inst_name,
            game,
            types,
            consts: params,
        }
    }

    pub(crate) fn with_other_game(&self, game: Composition) -> GameInstance {
        GameInstance {
            game,
            ..self.clone()
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn game_name(&self) -> &str {
        &self.game.name
    }

    pub(crate) fn game(&self) -> &Composition {
        &self.game
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClaimType {
    Lemma,
    Relation,
    Invariant,
}

impl ClaimType {
    pub fn guess_from_name(name: &str) -> ClaimType {
        if name.starts_with("relation") {
            ClaimType::Relation
        } else if name.starts_with("invariant") {
            ClaimType::Invariant
        } else {
            ClaimType::Lemma
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Claim {
    pub(crate) name: String,
    pub(crate) ty: ClaimType,
    pub(crate) dependencies: Vec<String>,
}

impl Claim {
    pub fn from_tuple(data: (String, Vec<String>)) -> Self {
        let (name, dependencies) = data;
        let ty = ClaimType::guess_from_name(&name);

        Self {
            name,
            ty,
            dependencies,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> ClaimType {
        self.ty
    }

    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
}


#[derive(Clone, Debug)]
pub struct Proof<'a> {
    pub(crate) name: String,
    pub(crate) consts: Vec<(String, Type)>,
    pub(crate) instances: Vec<GameInstance>,
    pub(crate) assumptions: Vec<Assumption>,
    pub(crate) theorems: Vec<TTheorem<'a>>,
    pub(crate) game_hops: Vec<GameHop<'a>>,
    pub(crate) pkgs: Vec<Package>,
}

impl Proof<'_> {
    pub(crate) fn with_new_instances(&self, instances: Vec<GameInstance>) -> Proof {
        Proof {
            instances,
            ..self.clone()
        }
    }

    pub(crate) fn as_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn game_hops(&self) -> &[GameHop] {
        &self.game_hops
    }

    pub(crate) fn instances(&self) -> &[GameInstance] {
        &self.instances
    }

    pub(crate) fn find_game_instance(&self, game_inst_name: &str) -> Option<&GameInstance> {
        self.instances
            .iter()
            .find(|inst| inst.name == game_inst_name)
    }
}
