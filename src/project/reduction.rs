use serde_derive::{Deserialize, Serialize};
use itertools::Itertools;
use std::collections::{HashSet,HashMap};
use std::iter::FromIterator;

use crate::package::{Composition,PackageInstance,Edge};
use crate::project::util::{diff, matches_assumption, walk_up_paths, DiffRow};
use crate::project::{Assumption, Result};

use super::assumption::ResolvedAssumption;
use super::Error;

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

fn same_package(left: &PackageInstance, right: &PackageInstance) -> bool {
    left.pkg == right.pkg
}


impl ResolvedReduction {
    /// Prove needs to verify four aspects
    /// - Mapping is Valid
    ///   - PackageInstances are only mentioned once
    ///   - Mapping may only occure with the same package type
    /// - Every PackageInstance in the assumptions is mapped
    /// - Every PackageInstance in the game, which is mapped
    ///   only calls other mapped package instances
    /// - The PackageInstances in the games which are *not* mapped need to be identical
    pub fn prove(&self) -> Result<()> {
        let ResolvedReduction {
            left,
            right,
            assumption,
            assumption_name,
            leftmap,
            rightmap,
        } = self;

        // PackageInstances are only mentioned once
        if ! (leftmap.iter().map(|(from, to)| from).all_unique()) {
            return Err(Error::ProofCheck(format!("leftmap has duplicate from")));
        }
        if ! (leftmap.iter().map(|(from, to)| to).all_unique()) {
            return Err(Error::ProofCheck(format!("leftmap has duplicate to")));
        }
        if ! (rightmap.iter().map(|(from, to)| from).all_unique()) {
            return Err(Error::ProofCheck(format!("rightmap has duplicate from")));
        }
        if ! (rightmap.iter().map(|(from, to)| to).all_unique()) {
            return Err(Error::ProofCheck(format!("rightmap has duplicate to")));
        }

        // Mapping may only occure with the same package type
        let mismatches_left : Vec<_> = leftmap.iter().filter(|(from, to)|{
            ! same_package(&left.pkgs[*from], &assumption.left.pkgs[*to])
        }).map(|(from,to)|{
            format!("{} and {} have different types",
                    left.pkgs[*from].name, assumption.left.pkgs[*to].name)
        }).collect();
        if ! mismatches_left.is_empty() {
            return Err(Error::ProofCheck(format!("leftmap has incompatible package instances: {}", mismatches_left.join(", "))));
        }
        let mismatches_right : Vec<_> = rightmap.iter().filter(|(from, to)|{
            ! same_package(&right.pkgs[*from], &assumption.right.pkgs[*to])
        }).map(|(from,to)|{
            format!("{} and {} have different types",
                    right.pkgs[*from].name, assumption.right.pkgs[*to].name)
        }).collect();
        if ! mismatches_right.is_empty() {
            return Err(Error::ProofCheck(format!("rightmap has incompatible package instances: {}", mismatches_right.join(", "))));
        }

        // Every PackageInstance in the assumptions is mapped
        if assumption.left.pkgs.len() != leftmap.len() {
            return Err(Error::ProofCheck(format!("Some package instances in leftasusmption are not mapped")));
        }
        if assumption.right.pkgs.len() != rightmap.len() {
            return Err(Error::ProofCheck(format!("Some package instances in rightasusmption are not mapped")));
        }

        // Every PackageInstance in the game, which is mapped
        // only calls other mapped package instances
        for Edge(from, to, _sig) in &left.edges {
            if (
                leftmap.iter().find(|(gameidx,_)|gameidx == to).is_none() &&
                    ! leftmap.iter().find(|(gameidx,_)|gameidx == from).is_none()
            ) {
                return Err(Error::ProofCheck(format!("Left Game: Mapped package {} calls unmappedpackage {}", left.pkgs[*from].name, left.pkgs[*to].name)));
            }
        }
        for Edge(from, to, _sig) in &right.edges {
            if (
                rightmap.iter().find(|(gameidx,_)|gameidx == to).is_none() &&
                    ! rightmap.iter().find(|(gameidx,_)|gameidx == from).is_none()
            ) {
                return Err(Error::ProofCheck(format!("Right Game: Mapped package {} calls unmappedpackage {}", right.pkgs[*from].name, right.pkgs[*to].name)));
            }
        }

        // The PackageInstances in the games which are *not* mapped need to be identical
        let unmapped_left : HashMap<_,_> = left
            .pkgs
            .iter()
            .enumerate()
            .filter(|(i, _)| leftmap
                    .iter()
                    .find(|(gameidx,_)|gameidx == i)
                    .is_none())
            .map(|(_, pkginst)|{
                (pkginst.name.clone(), pkginst)
            })
            .collect();
        let unmapped_right : HashMap<_,_> = right
            .pkgs
            .iter()
            .enumerate()
            .filter(|(i, _)| rightmap
                    .iter()
                    .find(|(gameidx,_)|gameidx == i)
                    .is_none())
            .map(|(_, pkginst)|{
                (pkginst.name.clone(), pkginst)
            })
            .collect();

        if HashSet::<_>::from_iter(unmapped_left.keys()) != HashSet::<_>::from_iter(unmapped_right.keys()) {
            return Err(Error::ProofCheck(format!("unmapped mapckage instances not equal: {:?} and {:?}", unmapped_left.keys(), unmapped_right.keys())));

        }

        for name in unmapped_left.keys() {
            if ! same_package(unmapped_left[name], unmapped_right[name]) {
                return Err(Error::ProofCheck(format!("Packages with name {} have different sort",name)));
            }
        }
        
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
