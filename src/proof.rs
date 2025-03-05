use itertools::Itertools as _;

use crate::{
    expressions::Expression,
    gamehops::{reduction::Assumption, GameHop},
    identifier::{
        game_ident::{GameConstIdentifier, GameIdentifier},
        proof_ident::ProofIdentifier,
        Identifier,
    },
    package::{Composition, OracleSig, Package},
    packageinstance::instantiate::InstantiationContext,
    types::{CountSpec, Type},
};

use crate::impl_Named;

////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub(crate) struct GameInstance {
    pub(crate) name: String,
    pub(crate) game: Composition,
    pub(crate) types: Vec<(String, Type)>,
    pub(crate) consts: Vec<(GameConstIdentifier, Expression)>,
}

impl_Named!(GameInstance);

mod instantiate {
    use crate::{
        package::Package,
        packageinstance::{instantiate::InstantiationContext, PackageInstance},
        types::Type,
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

        let pkg = Package {
            oracles: new_oracles,
            // split_oracles: new_split_oracles,
            ..pkg_inst.pkg.clone()
        };

        for (_, expr) in &mut pkg_inst.params {
            *expr = inst_ctx.rewrite_expression(expr)
        }

        for index in &mut pkg_inst.multi_instance_indices.indices {
            *index = inst_ctx.rewrite_expression(index);
        }

        PackageInstance { pkg, ..pkg_inst }
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

        let game = Composition {
            name: game.name.clone(),
            pkgs: new_pkg_instances,

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

    pub(crate) fn types(&self) -> &[(String, Type)] {
        &self.types
    }

    pub(crate) fn game_name(&self) -> &str {
        &self.game.name
    }

    pub(crate) fn game(&self) -> &Composition {
        &self.game
    }

    /// instantiates the provided type. this means that occurrences game parameters
    /// are replaced with the assigned values.
    ///
    /// This also means that the types somehow don't match 100%, this will just ignore that type and
    /// leave it as-is. But that shouldn't really happen, since we compare the types in the package
    /// params with the types in the code. But it could be the source of annoying-to-debug bugs.
    ///
    /// this is needed for Bits(n), since the `n` is different in game and package.
    pub(crate) fn instantiate_type(&self, ty: Type) -> Type {
        // we only want the ints, because the maybe be in Bits(n)
        let int_params = self
            .consts
            .iter()
            .filter(|(_, expr)| matches!(expr.get_type(), Type::Integer))
            .flat_map(|(ident, expr)| {
                let assigned_countspec = match expr {
                    Expression::Identifier(Identifier::ProofIdentifier(ident)) => CountSpec::Identifier(ident.clone().into()),
                    Expression::IntegerLiteral(num) => CountSpec::Literal(*num as u64),
                    other => panic!("found unexpected expression in constant assignment in game instance: {other:?}"),
                };

                // The code that we are replacing might not have been enriched with this optional
                // information yet, so we we take it out in order for the comparison to not fail
                // TODO: Maybe just implement comparison such that these values are ignored?
                let noned_ident = GameConstIdentifier {
                    game_inst_name: None,
                    proof_name: None,
                    inst_info: None,
                    assigned_value: None,
                    ..ident.clone()
                };

                vec![
                            (
                                Type::Bits(Box::new(crate::types::CountSpec::Identifier(
                                    noned_ident.clone().into(),
                                ))),
                                Type::Bits(Box::new(assigned_countspec.clone())),
                            ),
                            (
                                Type::Bits(Box::new(crate::types::CountSpec::Identifier(
                                    ident.clone().into(),
                                ))),
                                Type::Bits(Box::new(assigned_countspec)),
                            ),
                ]
            })
            .collect_vec();

        ty.rewrite_type(&int_params)
    }

    /// instantiates the provided oraclae signature. this means that occurrences game parameters
    /// are replaced with the assigned values.
    ///
    /// this is needed for Bits(n), since the `n` is different in game and package.
    pub(crate) fn instantiate_oracle_signature(&self, sig: OracleSig) -> OracleSig {
        OracleSig {
            args: sig
                .args
                .into_iter()
                .map(|(name, ty)| (name, self.instantiate_type(ty)))
                .collect(),
            tipe: self.instantiate_type(sig.tipe),
            ..sig
        }
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
    pub(crate) tipe: ClaimType,
    pub(crate) dependencies: Vec<String>,
}

impl Claim {
    pub fn from_tuple(data: (String, Vec<String>)) -> Self {
        let (name, dependencies) = data;
        let tipe = ClaimType::guess_from_name(&name);

        Self {
            name,
            tipe,
            dependencies,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tipe(&self) -> ClaimType {
        self.tipe
    }

    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
}

impl crate::util::resolver::Named for Claim {
    fn as_name(&self) -> &str {
        self.name()
    }
}

#[derive(Clone, Debug)]
pub struct Proof<'a> {
    pub(crate) name: String,
    pub(crate) consts: Vec<(String, Type)>,
    pub(crate) instances: Vec<GameInstance>,
    pub(crate) assumptions: Vec<Assumption>,
    pub(crate) game_hops: Vec<GameHop<'a>>,
    pub(crate) pkgs: Vec<Package>,
}

impl Proof<'_> {
    pub(crate) fn new(
        name: String,
        consts: Vec<(String, Type)>,
        instances: Vec<GameInstance>,
        assumptions: Vec<Assumption>,
        game_hops: Vec<GameHop>,
        pkgs: Vec<Package>,
    ) -> Proof {
        Proof {
            name,
            consts,
            instances,
            assumptions,
            game_hops,
            pkgs,
        }
    }

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

    pub(crate) fn assumptions(&self) -> &[Assumption] {
        &self.assumptions
    }

    pub(crate) fn packages(&self) -> &[Package] {
        &self.pkgs
    }

    pub(crate) fn find_game_instance(&self, game_inst_name: &str) -> Option<&GameInstance> {
        self.instances
            .iter()
            .find(|inst| inst.name == game_inst_name)
    }
}
