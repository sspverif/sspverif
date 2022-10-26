use serde_derive::{Deserialize, Serialize};

use crate::package::Composition;
use crate::project::util::{diff, matches_assumption, walk_up_paths, DiffRow};
use crate::project::{Assumption, Result};

use super::assumption::ResolvedAssumption;

// TODO: add a HybridArgument variant
#[derive(Debug, Serialize, Deserialize)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    Unspecified,
}

// TODO: add a HybridArgument variant
#[derive(Debug, Serialize, Deserialize)]
pub struct Reduction {
    pub left: String,
    pub right: String,
    //pub direction: Direction, // unclear if we need it
    pub assumption: String,
    pub leftmap: Vec<(String, String)>,
    pub rightmap: Vec<(String, String)>,
    // we probably have to provide more information here,
    // in order to easily figure out how to perform the rewrite
}

pub struct ResolvedReduction {
    pub left: Composition,
    pub right: Composition,
    pub assumption: ResolvedAssumption,
    pub assumption_name: String,
    pub leftmap: Vec<(usize, usize)>,
    pub rightmap: Vec<(usize, usize)>,
}

/*

approach:

1. find the diff of left/right games and assumptions, recoding the path of signatures
2. for both left and right:
2.1. in the game, walk the path given by these signatures
2.2. check that the subgame starting by that root is identical


22-09-21 conceptualization
1. find diffs
2. check that diffs are the same package and params
    -> actually it might make sense to make this a separate function
3. in the game hop games, walk back the paths (diff->roots) from the assumption
4. use these as roots to a new composition (both left and right) and generate that (take care of exports)
5. compare to assumption

- a lot of this is comparing parts of the composition. Maybe we should add that as a function on the composition.
- what makes these comparisons tricky is that they don't need to be equal, they just need to be at least as strict as the assumption. It's okay if it offers less oracles to the adversary.
  -> this only concerns the exports


impl Composition {
    fn
}
 */

impl ResolvedReduction {
    pub fn prove(&self) -> Result<()> {
        let ResolvedReduction {
            left,
            right,
            assumption,
            assumption_name,
            leftmap,
            rightmap,
        } = self;

        let game_diff = diff(left, right);
        let assumption_diff = diff(&assumption.left, &assumption.right);

        println!("game diff: {game_diff:#?}");

        if game_diff.is_same() {
            panic!("same game I suppose??");
        }

        if assumption_diff.is_same() {
            panic!("assumption has same games I suppose??");
        }

        let left_path: Vec<_> = game_diff
            .iter()
            .map(|DiffRow(left, _)| left)
            .cloned()
            .collect();

        let right_path: Vec<_> = game_diff
            .iter()
            .map(|DiffRow(_, right)| right)
            .cloned()
            .collect();

        let left_comp_slice = walk_up_paths(left, &left_path);
        let right_comp_slice = walk_up_paths(right, &right_path);

        //println!("left root: {left_root}");
        //println!("right root: {right_root}");

        println!("applying assumption {assumption_name}.");

        assert!(matches_assumption(left_comp_slice, &assumption.left));
        assert!(matches_assumption(right_comp_slice, &assumption.right));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::package::{
        Composition, Edge, Export, OracleDef, OracleSig, Package, PackageInstance,
    };
    use crate::project::assumption::{self, ResolvedAssumption};
    use crate::statement::CodeBlock;
    use crate::types::Type;

    use super::ResolvedReduction;

    #[test]
    fn check_assumption() {
        let osig_a = OracleSig {
            name: "doA".to_string(),
            args: vec![],
            tipe: Type::Empty,
        };

        let osig_b = OracleSig {
            name: "doB".to_string(),
            args: vec![],
            tipe: Type::Empty,
        };

        let pkg_a = Package {
            name: "A".to_string(),
            params: vec![],
            state: vec![],
            oracles: vec![OracleDef {
                sig: osig_a.clone(),
                code: CodeBlock(vec![]),
            }],
            imports: vec![],
        };

        let pkg_b = Package {
            name: "B".to_string(),
            params: vec![],
            state: vec![],
            oracles: vec![OracleDef {
                sig: osig_b.clone(),
                code: CodeBlock(vec![]),
            }],
            imports: vec![],
        };

        let left = Composition {
            pkgs: vec![
                PackageInstance {
                    pkg: pkg_a.clone(),
                    params: HashMap::new(),
                    name: "leftA1".to_string(),
                },
                PackageInstance {
                    pkg: pkg_a.clone(),
                    params: HashMap::new(),
                    name: "leftA2".to_string(),
                },
                PackageInstance {
                    pkg: pkg_a.clone(),
                    params: HashMap::new(),
                    name: "leftA3".to_string(),
                },
            ],
            exports: vec![Export(0, osig_a.clone())],
            edges: vec![Edge(0, 1, osig_a.clone()), Edge(0, 2, osig_a.clone())],
            name: "left".to_string(),
            consts: vec![],
        };

        let right = Composition {
            pkgs: vec![
                PackageInstance {
                    pkg: pkg_a.clone(),
                    params: HashMap::new(),
                    name: "leftA1".to_string(),
                },
                PackageInstance {
                    pkg: pkg_a.clone(),
                    params: HashMap::new(),
                    name: "leftA2".to_string(),
                },
                PackageInstance {
                    pkg: pkg_b.clone(),
                    params: HashMap::new(),
                    name: "leftB3".to_string(),
                },
            ],

            exports: vec![Export(0, osig_a.clone())],
            edges: vec![Edge(0, 1, osig_a.clone()), Edge(0, 2, osig_b.clone())],
            name: "right".to_string(),
            consts: vec![],
        };

        let assumption_left = Composition {
            pkgs: vec![
                PackageInstance {
                    pkg: pkg_a.clone(),
                    params: HashMap::new(),
                    name: "leftA1".to_string(),
                },
                PackageInstance {
                    pkg: pkg_a.clone(),
                    params: HashMap::new(),
                    name: "leftA3".to_string(),
                },
            ],
            exports: vec![Export(0, osig_a.clone())],
            edges: vec![Edge(0, 1, osig_a.clone())],
            name: "assumption_left".to_string(),
            consts: vec![],
        };
        let assumption_right = Composition {
            pkgs: vec![
                PackageInstance {
                    pkg: pkg_a.clone(),
                    params: HashMap::new(),
                    name: "rightA1".to_string(),
                },
                PackageInstance {
                    pkg: pkg_b.clone(),
                    params: HashMap::new(),
                    name: "rightB3".to_string(),
                },
            ],
            exports: vec![Export(0, osig_a.clone())],
            edges: vec![Edge(0, 1, osig_b.clone())],
            name: "assumption_right".to_string(),
            consts: vec![],
        };

        let reduction = ResolvedReduction {
            left,
            right,
            assumption: ResolvedAssumption {
                left: assumption_left,
                right: assumption_right,
            },
            assumption_name: "assumption_name".to_string(),
        };

        reduction.prove();
    }
}
