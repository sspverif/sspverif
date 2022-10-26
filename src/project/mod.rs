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

mod assumption;
mod equivalence;
mod load;
mod reduction;
mod resolve;
//mod util;

pub use assumption::Assumption;
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
                leftmap,
                rightmap,
                ..
            } => GameHop::Reduction(Reduction {
                left,
                right,
                assumption,
                leftmap,
                rightmap,
                //direction: Direction::Unspecified,
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
        let (game_hops, assumptions) = load::toml_file(root_dir.clone(), &games)?;

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
        for (i, game_hop) in self.game_hops.iter().enumerate() {
            match game_hop {
                GameHop::Reduction(red) => self.resolve_reduction(&red, i)?.prove()?,
                GameHop::Equivalence(eq) => self.resolve_equivalence(eq)?.prove()?,
            }
        }

        Ok(())
    }

    pub fn explain_game(&self, game_name: &str) -> Result<String> {
        let game = self.get_game(game_name).ok_or(Error::UndefinedGame(
            game_name.to_string(),
            format!("in explain"),
        ))?;

        let mut buf = String::new();
        let mut w = crate::writers::pseudocode::fmtwriter::FmtWriter::new(&mut buf, true);
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

    pub fn get_assumption<'a>(&'a self, name: &str) -> Option<&'a Assumption> {
        self.assumptions.get(name)
    }

    pub fn get_root_dir(&self) -> PathBuf {
        self.root_dir.clone()
    }

    pub fn get_smt_game_file(&self, game_name: &str) -> Result<std::fs::File> {
        let mut path = self.root_dir.clone();

        path.push("_build/code_eq/games/");
        std::fs::create_dir_all(&path)?;

        path.push(format!("{game_name}.smt2"));
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)?;

        Ok(f)
    }

    pub fn get_smt_decl_file(
        &self,
        left_game_name: &str,
        right_game_name: &str,
    ) -> Result<std::fs::File> {
        let mut path = self.root_dir.clone();

        path.push("_build/code_eq/decls/");
        std::fs::create_dir_all(&path)?;

        path.push(format!("{left_game_name}_{right_game_name}.smt2"));
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)?;

        Ok(f)
    }

    fn get_invariant_path(&self, invariant_path: &str) -> PathBuf {
        let path = PathBuf::from(invariant_path);
        if path.is_absolute() {
            path
        } else {
            self.root_dir.join(path)
        }
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
