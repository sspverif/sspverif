/**
 *  project is the high-level structure of sspverif.
 *
 *  here we assemble all the users' packages, assumptions, game hops and equivalence proofs.
 *  we also facilitate individual proof steps here, and provide an interface for doing the whole proof.
 *
 */
use serde_derive::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::{collections::HashMap, path::PathBuf};

use error::{Error, Result};

use crate::package::{Composition, Package};

pub const PROJECT_FILE: &str = "ssp.toml";
pub const GAMEHOPS_FILE: &str = "game_hops.toml";

pub const PACKAGES_DIR: &str = "packages";
pub const GAMES_DIR: &str = "games";
pub const ASSUMPTIONS_DIR: &str = "assumptions";

pub const PACKAGE_EXT: &str = ".pkg.ssp";
pub const GAME_EXT: &str = ".comp.ssp"; // TODO maybe change this to .game.ssp later, and also rename the Composition type

mod equivalence;
mod load;
mod reduction;

use equivalence::Equivalence;
use reduction::Reduction;

// TODO: add a HybridArgument variant
#[derive(Debug, Serialize, Deserialize)]
pub enum GameHop {
    Reduction(Reduction),
    Equivalence(Equivalence),
}

impl From<load::TomlGameHop> for GameHop {
    fn from(toml_hop: load::TomlGameHop) -> Self {
        match toml_hop {
            load::TomlGameHop::Reduction {
                left,
                right,
                assumption,
            } => GameHop::Reduction(Reduction {
                left,
                right,
                assumption,
            }),
            load::TomlGameHop::Equivalence {
                left,
                right,
                invariant_path,
            } => GameHop::Equivalence(Equivalence {
                left,
                right,
                invariant_path,
            }),
        }
    }
}

#[derive(Debug)]
pub struct Assumption {
    left: String,
    right: String,
}

pub mod error;

#[derive(Debug)]
pub struct Project {
    root_dir: PathBuf,
    packages: HashMap<String, Package>,
    assumptions: HashMap<String, Assumption>,
    games: HashMap<String, Composition>,
    game_hops: Vec<GameHop>,
}

impl Project {
    pub fn load() -> Result<Project> {
        let root_dir = find_project_root()?;

        let packages = load::packages(root_dir.clone())?;
        let games = load::games(root_dir.clone(), &packages)?;
        let assumptions = load::assumptions(root_dir.clone())?;
        let game_hops = load::game_hops(root_dir.clone(), &games, &assumptions)?;

        let project = Project {
            root_dir,
            packages,
            games,
            assumptions,
            game_hops,
        };

        Ok(project)
    }

    // we might want to return a proof trace here instead
    // we could then extract the proof viewer output and other useful info trom the trace
    pub fn prove(&self) -> Result<()> {
        for game_hop in &self.game_hops {
            match game_hop {
                GameHop::Reduction { .. } => {
                    println!("skipping reductions for now, not implemented");
                    continue;
                }
                GameHop::Equivalence(eq) => {
                    eq.resolve(self)?.prove()?;
                }
            }
        }

        Ok(())
    }

    pub fn explain_game(&self, game_name: &str) -> Result<String> {
        let game = match self.get_game(game_name) {
            Some(game) => game,
            None => {
                return Err(Error::CompositionMissing(game_name.to_string()));
            }
        };

        let mut buf = String::new();
        let mut w = crate::writers::pseudocode::fmtwriter::FmtWriter::new(&mut buf);
        let (game, _, _) = crate::transforms::transform_explain(&game)?;

        println!("Explaining game {game_name}:");
        for inst in game.pkgs {
            let pkg = inst.pkg;
            w.write_package(&pkg).unwrap();
        }

        Ok(buf)
        //tex_write_composition(&comp, Path::new(&args.output));
    }

    pub fn get_game<'a>(&'a self, name: &str) -> Option<&'a Composition> {
        self.games.get(name)
    }

    pub fn get_root_dir(&self) -> PathBuf {
        self.root_dir.clone()
    }
}

fn find_project_root() -> std::io::Result<std::path::PathBuf> {
    let mut dir = std::env::current_dir()?;

    loop {
        let lst = dir.read_dir()?;
        for entry in lst {
            let entry = entry?;
            let file_name = match entry.file_name().into_string() {
                Err(_) => continue,
                Ok(name) => name,
            };
            if file_name == PROJECT_FILE {
                return Ok(dir);
            }
        }

        match dir.parent() {
            None => return Err(std::io::Error::from(ErrorKind::NotFound)),
            Some(parent) => dir = parent.into(),
        }
    }
}
