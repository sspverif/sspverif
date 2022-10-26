use std::{collections::HashMap, path::PathBuf};

use super::*;
use error::{Error, Result};

use crate::package::{Composition, Package};
use crate::parser::composition::handle_composition;
use crate::parser::package::handle_pkg;
use crate::parser::SspParser;

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
    },
}

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

                let parse_result = SspParser::parse_package(&contents);
                if let Err(e) = parse_result {
                    return Err((filename, e).into());
                }
                let mut ast = parse_result.unwrap();
                let (pkg_name, pkg) = handle_pkg(ast.next().unwrap());

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
        if let Some(name) = dir_entry.file_name().to_str() {
            if name.ends_with(GAME_EXT) {
                let filecontent = std::fs::read_to_string(dir_entry.path())?;
                let parse_result = SspParser::parse_composition(&filecontent);
                if let Err(e) = parse_result {
                    return Err((name, e).into());
                }
                let mut ast = parse_result.unwrap();
                let comp = handle_composition(ast.next().unwrap(), pkgs);
                let comp_name = comp.name.clone();

                games.insert(comp_name, comp);
            }
        }
    }

    Ok(games)
}

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
            }
        }
    }

    Ok(())
}
