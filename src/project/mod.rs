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

use error::Result;

use crate::hacks;
use crate::package::{Composition, Package};
use crate::writers::smt::exprs::SmtFmt;
use crate::writers::smt::writer::CompositionSmtWriter;
use std::io::Write;

pub const PROJECT_FILE: &str = "ssp.toml";
pub const GAMEHOPS_FILE: &str = "game_hops.toml";

pub const PACKAGES_DIR: &str = "packages";
pub const GAMES_DIR: &str = "games";
pub const ASSUMPTIONS_DIR: &str = "assumptions";

pub const PACKAGE_EXT: &str = ".pkg.ssp";
pub const GAME_EXT: &str = ".comp.ssp"; // TODO maybe change this to .game.ssp later, and also rename the Composition type

mod load;

// TODO: add a HybridArgument variant
#[derive(Debug, Serialize, Deserialize)]
pub enum GameHop {
    Reduction {
        left: String,
        right: String,
        assumption: String,
        // we probably have to provide more information here,
        // in order to easily figure out how to perform the rewrite
    },
    Equivalence {
        left: String,
        right: String,
        invariant_path: String,
    },
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
        let root = find_project_root()?;

        let pkgs = load::packages(root.clone())?;
        let games = load::games(root.clone(), &pkgs)?;
        let assumptions = load::assumptions(root.clone())?;
        let game_hops = load::game_hops(root.clone(), &games, &assumptions)?;

        let project = Project {
            root_dir: root,
            packages: pkgs,
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
                GameHop::Equivalence {
                    left,
                    right,
                    invariant_path,
                } => {
                    let left_game = &self.games[left];
                    let right_game = &self.games[right];
                    prove_equivalence(left_game, right_game, invariant_path)?;
                }
            }
        }

        Ok(())
    }
}

fn prove_equivalence(left: &Composition, right: &Composition, inv_path: &str) -> Result<()> {
    /*
    TODO in this function:
        - respect the parameter/constants mapping
        - generate paramter functions
        - fail if types don't match across games
    */

    let mut proc = subprocess::Popen::create(
        &["z3", "-in", "-smt2"],
        subprocess::PopenConfig {
            stdin: subprocess::Redirection::Pipe,
            stdout: subprocess::Redirection::Pipe,
            stderr: subprocess::Redirection::Pipe,
            ..subprocess::PopenConfig::default()
        },
    )
    .unwrap();

    let mut buf = Vec::<u8>::new();

    hacks::declare_par_Maybe(&mut buf)?;
    hacks::declare_Tuple(&mut buf, 2)?;
    write!(buf, "(declare-sort Bits_n 0)")?;
    write!(buf, "(declare-sort Bits_* 0)")?;
    write!(buf, "(declare-fun f (Bits_n Bits_n) Bits_n)")?;

    for comp in [left, right] {
        let (comp, _, samp) = crate::transforms::transform_all(&comp).unwrap();
        let mut writer = CompositionSmtWriter::new(&comp, samp);
        for line in writer.smt_composition_all() {
            line.write_smt_to(&mut buf)?;
        }
    }

    write!(buf, "(check-sat)")?;

    match proc.communicate_bytes(Some(&buf)).unwrap() {
        (Some(stdout_data), Some(stderr_data)) => {
            let data = String::from_utf8(stdout_data).unwrap();
            let err = String::from_utf8(stderr_data).unwrap();
            println!("data: {data}.");
            println!("err: {err}.");
        }
        _ => {
            unreachable!("")
        }
    }

    Ok(())
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
