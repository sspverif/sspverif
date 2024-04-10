use std::iter::FromIterator;
use std::{collections::HashMap, path::PathBuf};

use super::*;
use error::{Error, Result};

use crate::package::{Composition, Package};
use crate::parser::composition::handle_composition;
use crate::parser::package::handle_pkg;
use crate::parser::proof::handle_proof;
use crate::parser::SspParser;
use crate::proof::Proof;
use crate::util::scope::Scope;
extern crate toml_edit;

/*
[assumptions]
[assumptions.LeftRight]
left = "Left"
right = "Right"

[[game_hops]]

[game_hops.Reduction]

left = "Left"
right = "Right"
assumption = "LeftRight"
 */

// left is the name of the lemma, right is list of names of dependency lemmas
pub type ProofTreeSpec = Vec<(String, Vec<String>)>;

// TODO: add a HybridArgument variant
#[derive(Debug, Serialize, Deserialize)]
pub enum TomlGameHop {
    Reduction {
        left: String,
        right: String,
        assumption: String,
        leftmap: Vec<(String, String)>,
        rightmap: Vec<(String, String)>,
        //direction: String,
        // we probably have to provide more information here,
        // in order to easily figure out how to perform the rewrite
    },
    Equivalence {
        left: String,
        right: String,
        invariant_path: String,
        // the key here is the oracle name I guess??
        trees: HashMap<String, ProofTreeSpec>,
    },
}

/*
#[derive(Debug, Serialize, Deserialize)]
struct TomlStructure {
    game_hops: Vec<TomlGameHop>,
    assumptions: HashMap<String, Assumption>,
}

pub(crate) fn toml_file(
    root: PathBuf,
    games: &HashMap<String, Composition>,
) -> Result<(Vec<GameHop>, HashMap<String, Assumption>)> {
    let mut path = root.clone();
    path.push(PROJECT_FILE);

    let filecontent = std::fs::read(&path)?;
    let toml_data = toml_edit::easy::from_slice::<TomlStructure>(&filecontent).unwrap();

    validate_assumptions(&toml_data.assumptions, games)?;

    let hops: Vec<GameHop> = toml_data
        .game_hops
        .into_iter()
        .map(|toml_hop| toml_hop.into())
        .collect();

    validate_game_hops(&hops, games, &toml_data.assumptions)?;

    Ok((hops, toml_data.assumptions))
}
*/
pub(crate) fn packages(root: PathBuf) -> Result<HashMap<String, Package>> {
    let mut dir = root;
    dir.push(PACKAGES_DIR);
    let dir_str = dir.to_str().expect("coun't get the path string");

    let mut pkgs = HashMap::new();
    let mut pkgs_filenames: HashMap<String, String> = HashMap::new();

    for dir_entry in std::fs::read_dir(dir_str)? {
        let dir_entry = dir_entry?;
        let filename = dir_entry.file_name();
        if let Some(filename) = filename.to_str() {
            if filename.ends_with(PACKAGE_EXT) {
                let contents = std::fs::read_to_string(dir_entry.path())?;

                let mut ast = SspParser::parse_package(&contents).map_err(|e| (filename, e))?;
                let (pkg_name, pkg) =
                    handle_pkg(ast.next().unwrap(), filename).map_err(Error::PackageParse)?;

                if let Some(other_filename) = pkgs_filenames.get(&pkg_name) {
                    return Err(Error::RedefinedPackage(
                        pkg_name,
                        filename.to_string(),
                        other_filename.to_string(),
                    ));
                }

                pkgs.insert(pkg_name.clone(), pkg);
                pkgs_filenames.insert(pkg_name, filename.to_string());
            }
        }
    }

    Ok(pkgs)
}

pub(crate) fn games(
    root: PathBuf,
    pkgs: &HashMap<String, Package>,
) -> Result<HashMap<String, Composition>> {
    let mut dir = root;
    dir.push(GAMES_DIR);
    let dir_str = dir.to_str().expect("coun't get the path string");

    let mut games = HashMap::new();

    for dir_entry in std::fs::read_dir(dir_str)? {
        let dir_entry = dir_entry?;
        if let Some(file_name) = dir_entry.file_name().to_str() {
            if file_name.ends_with(GAME_EXT) {
                let filecontent = std::fs::read_to_string(dir_entry.path())?;
                let mut ast =
                    SspParser::parse_composition(&filecontent).map_err(|err| (file_name, err))?;

                let comp = match handle_composition(ast.next().unwrap(), pkgs, file_name) {
                    Ok(game) => game,
                    Err(err) => {
                        println!("printing error...");
                        return Err(err.with_source(filecontent).into());
                    }
                };
                let comp_name = comp.name.clone();

                games.insert(comp_name, comp);
            }
        }
    }

    Ok(games)
}

pub(crate) fn proofs(
    root: PathBuf,
    pkgs: &HashMap<String, Package>,
    games: &HashMap<String, Composition>,
) -> Result<HashMap<String, Proof>> {
    let mut dir = root;
    dir.push(PROOFS_DIR);
    let dir_str = dir.to_str().expect("couldn't get the path string");

    let pkgs = Vec::from_iter(pkgs.values().cloned());
    let games = Vec::from_iter(games.values().cloned());

    let mut proofs = HashMap::new();

    for dir_entry in std::fs::read_dir(dir_str)? {
        let dir_entry = dir_entry?;
        if let Some(name) = dir_entry.file_name().to_str() {
            if name.ends_with(".ssp") {
                // TODO make a constant and figure out if we really need the sub-extensions

                let filecontent = std::fs::read_to_string(dir_entry.path())?;
                let parse_result = SspParser::parse_proof(&filecontent);
                if let Err(e) = parse_result {
                    return Err((name, e).into());
                }

                let mut scope = Scope::new();

                let mut ast = parse_result.unwrap();
                let proof = match handle_proof(ast.next().unwrap(), &mut scope, &pkgs, &games, name)
                {
                    Ok(proof) => proof,
                    Err(err) => {
                        return Err(err.with_source(filecontent).into());
                    }
                };
                let proof_name = proof.as_name().to_string();

                proofs.insert(proof_name, proof);
            }
        }
    }

    Ok(proofs)
}

/*
pub(crate) fn validate_assumptions(
    assumptions: &HashMap<String, Assumption>,
    games: &HashMap<String, Composition>,
) -> Result<()> {
    for (name, Assumption { left, right }) in assumptions.iter() {
        if !games.contains_key(left) {
            return Err(Error::UndefinedGame(
                left.clone(),
                format!("left in assumption {name}"),
            ));
        }

        if !games.contains_key(right) {
            return Err(Error::UndefinedGame(
                right.clone(),
                format!("right in assumption {name}"),
            ));
        }
    }

    return Ok(());
}

fn validate_game_hops(
    hops: &[GameHop],
    games: &HashMap<String, Composition>,
    assumptions: &HashMap<String, Assumption>,
) -> Result<()> {
    for (i, hop) in hops.iter().enumerate() {
        match hop {
            GameHop::Reduction(Reduction {
                left,
                right,
                assumption,
                ..
            }) => {
                if !games.contains_key(left) {
                    return Err(Error::UndefinedGame(
                        left.clone(),
                        format!("left in game hop {i} ({hop:?})"),
                    ));
                }
                if !games.contains_key(right) {
                    return Err(Error::UndefinedGame(
                        right.clone(),
                        format!("right in game hop {i} ({hop:?})"),
                    ));
                }
                if !assumptions.contains_key(assumption) {
                    return Err(Error::UndefinedAssumption(
                        assumption.clone(),
                        format!("in game hop {i} ({hop:?})"),
                    ));
                }
            }
            GameHop::Equivalence(eq) => {
                let Equivalence {
                    left,
                    right,
                    invariant_path: _,
                    trees,
                } = &eq;
                if !games.contains_key(left) {
                    return Err(Error::UndefinedGame(
                        left.clone(),
                        format!("left in game hop {i} ({hop:?})"),
                    ));
                }

                if !games.contains_key(right) {
                    return Err(Error::UndefinedGame(
                        right.clone(),
                        format!("right in game hop {i} ({hop:?})"),
                    ));
                };

                let left_game = &games[left];
                let right_game = &games[right];

                let left_sigs: HashSet<_> = HashSet::from_iter(&left_game.exports);
                let right_sigs: HashSet<_> = HashSet::from_iter(&right_game.exports);

                let left_not_right = left_sigs.difference(&right_sigs).collect_vec();
                let right_not_left = right_sigs.difference(&left_sigs).collect_vec();

                if left_sigs != right_sigs {
                    let err_msg =  match (left_not_right.is_empty(), right_not_left.is_empty()) {
                        (false, false) => format!("right game {right} exports oracles {right_not_left:?} that are not exported by left game {left} and left game exports oracles {left_not_right:?} that are not exported by right game"),
                        (false, true) => format!("left game {left} exports oracles {left_not_right:?} that are not exported by right game {right}"),
                        (true, false) => format!("right game {right} exports oracles {right_not_left:?} that are not exported by left game {left}"),
                        (true, true) => unreachable!(),
                    };

                    return Err(Error::ProofTreeValidationError(err_msg));
                }

                let sig_names: HashSet<_> =
                    left_sigs.iter().map(|Export(_, sig)| &sig.name).collect();
                let tree_keys: HashSet<_> = HashSet::from_iter(trees.keys());

                if !tree_keys.is_subset(&sig_names) {
                    let diff = tree_keys
                        .difference(&sig_names)
                        .map(|x| *x)
                        .cloned()
                        .collect_vec();
                    return Err(Error::ProofTreeValidationError(format!(
                        "proof trees {diff:?} refer to unexported oracles"
                    )));
                }
            }
        }
    }

    Ok(())
}
 */
