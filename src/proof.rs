use crate::{
    expressions::Expression,
    identifier::Identifier,
    package::{Composition, Package},
    types::Type,
    util::resolver::{Resolver, SliceResolver},
};

use crate::impl_Named;

////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct GameInstance {
    name: String,
    game_name: String,
    game: Composition,
    types: Vec<(String, Type)>,
    consts: Vec<(Identifier, Expression)>,
}

impl_Named!(GameInstance);

mod instantiate {
    use crate::{
        expressions::Expression, identifier::Identifier, package::PackageInstance, types::Type,
    };

    pub(crate) fn rewrite_pkg_inst(
        _game_inst_name: &str,
        _proof_name: &str,
        pkg_inst: &PackageInstance,
        _params: &[(Identifier, Expression)],
        _types: &[(String, Type)],
    ) -> PackageInstance {
        PackageInstance { ..pkg_inst.clone() }
    }
}

impl GameInstance {
    pub fn new(
        game_inst_name: &str,
        proof_name: &str,
        name: String,
        game: Composition,
        types: Vec<(String, Type)>,
        params: Vec<(Identifier, Expression)>,
    ) -> GameInstance {
        let game_name = game.name.clone();

        let new_pkg_instances = game
            .pkgs
            .iter()
            .map(|pkg_inst| {
                instantiate::rewrite_pkg_inst(game_inst_name, proof_name, pkg_inst, &params, &types)
            })
            .collect();

        let game = Composition {
            name: name.clone(),
            pkgs: new_pkg_instances,
            consts: vec![],

            ..game
        };

        GameInstance {
            name,
            game_name,
            game,
            types,
            consts: params,
        }
    }

    pub fn with_other_game(&self, game: Composition) -> GameInstance {
        GameInstance {
            game,
            ..self.clone()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn consts(&self) -> &[(Identifier, Expression)] {
        &self.consts
    }

    pub fn types(&self) -> &[(String, Type)] {
        &self.types
    }

    pub fn game_name(&self) -> &str {
        &self.game_name
    }

    pub fn game(&self) -> &Composition {
        &self.game
    }
}

#[derive(Clone, Debug)]
pub struct Mapping {
    game_inst_name: String,
    assumption_game_inst_name: String,

    // these also need validation
    // but let's not resolve them
    pkg_maps: Vec<(String, String)>,
}

impl Mapping {
    pub fn new(
        assumption_game_inst_name: String,
        game_inst_name: String,
        pkg_maps: Vec<(String, String)>,
    ) -> Mapping {
        Mapping {
            game_inst_name,
            assumption_game_inst_name,
            pkg_maps,
        }
    }

    pub fn as_game_inst_name(&self) -> &str {
        &self.game_inst_name
    }

    pub fn as_assumption_game_inst_name(&self) -> &str {
        &self.assumption_game_inst_name
    }

    pub fn pkg_maps(&self) -> &[(String, String)] {
        &self.pkg_maps
    }
}

#[derive(Debug, Clone)]
pub struct Assumption {
    pub name: String,
    pub left_name: String,
    pub right_name: String,
}

#[derive(Debug, Clone)]
pub struct Reduction {
    left: Mapping,
    right: Mapping,

    assumption_name: String,
}

impl Reduction {
    pub fn new(left: Mapping, right: Mapping, assumption_name: String) -> Self {
        Self {
            left,
            right,
            assumption_name,
        }
    }

    pub fn left(&self) -> &Mapping {
        &self.left
    }

    pub fn right(&self) -> &Mapping {
        &self.right
    }

    pub fn assumption_name(&self) -> &str {
        &self.assumption_name
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

// Equivalence contains the composisitions/games and the invariant data,
// whereas the pure Equivalence just contains the names and file paths.
// TODO: explore if we can keep references to the games in the project hashmap
#[derive(Debug, Clone)]
pub struct Equivalence {
    // these two are game instance names
    left_name: String,
    right_name: String,
    invariants: Vec<(String, Vec<String>)>,
    trees: Vec<(String, Vec<Claim>)>,
}

impl Equivalence {
    pub fn new(
        left_name: String,
        right_name: String,
        mut invariants: Vec<(String, Vec<String>)>,
        mut trees: Vec<(String, Vec<Claim>)>,
    ) -> Self {
        trees.sort();
        invariants.sort();

        Equivalence {
            left_name,
            right_name,
            invariants, // TODO INV
            trees,
        }
    }

    pub fn trees(&self) -> &[(String, Vec<Claim>)] {
        &self.trees
    }

    pub fn left_name(&self) -> &str {
        &self.left_name
    }

    pub fn right_name(&self) -> &str {
        &self.right_name
    }

    pub fn get_invariants(&self, offs: usize) -> Option<&[String]> {
        self.invariants
            .get(offs)
            .map(|(_name, invariants)| invariants.as_slice())
    }

    pub fn invariants_by_oracle_name(&self, oracle_name: &str) -> Vec<String> {
        SliceResolver(&self.invariants)
            .resolve_value(oracle_name)
            .map(|(_oname, inv_file_names)| inv_file_names.clone())
            .unwrap_or(vec![])
    }

    pub fn proof_tree_by_oracle_name(&self, oracle_name: &str) -> Vec<Claim> {
        SliceResolver(&self.trees)
            .resolve_value(oracle_name)
            .map(|(_oname, tree)| tree.clone())
            .unwrap_or(vec![])
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

// TODO: add a HybridArgument variant
#[derive(Debug, Clone)]
pub enum GameHop {
    Reduction(Reduction),
    Equivalence(Equivalence),
}

#[derive(Clone, Debug)]
pub struct Proof {
    name: String,
    consts: Vec<(String, Type)>,
    instances: Vec<GameInstance>,
    assumptions: Vec<Assumption>,
    game_hops: Vec<GameHop>,
    pkgs: Vec<Package>,
}

impl Proof {
    pub fn new(
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

    pub fn with_new_instances(&self, instances: Vec<GameInstance>) -> Proof {
        Proof {
            instances,
            ..self.clone()
        }
    }

    pub fn as_name(&self) -> &str {
        &self.name
    }

    pub fn game_hops(&self) -> &[GameHop] {
        &self.game_hops
    }

    pub fn instances(&self) -> &[GameInstance] {
        &self.instances
    }

    pub fn assumptions(&self) -> &[Assumption] {
        &self.assumptions
    }

    pub fn packages(&self) -> &[Package] {
        &self.pkgs
    }
}
