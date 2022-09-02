use super::{Error, Result};
use std::{collections::HashMap, path::PathBuf};

use serde_derive::{Deserialize, Serialize};

use crate::package::{Composition, Package};
use crate::writers::smt::writer::CompositionSmtWriter;
use crate::writers::smt::SmtFmt;

use super::filesystem::{
    find_project_root, parse_composition, parse_packages, read_compositions_directory,
    read_packages_directory,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum GameHop {
    Reduction {
        left: String,
        right: String,
        assumption: String,
        // we probably have to provide more information here
    },
    Equivalence {
        left: String,
        right: String,
        invariant_path: String,
    },
    // TODO: HybridArgument(...)
}

#[derive(Debug)]
pub struct Assumption {
    left: String,
    right: String,
}

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

        let pkgs = Self::load_packages(root.clone())?;
        let games = Self::load_games(root.clone(), &pkgs)?;
        let assumptions = Self::load_assumptions(root.clone())?;
        let game_hops = Self::load_game_hops(root.clone(), &games, &assumptions)?;

        let project = Project {
            root_dir: root,
            packages: pkgs,
            games,
            assumptions,
            game_hops,
        };

        Ok(project)
    }

    fn load_packages(root: PathBuf) -> Result<HashMap<String, Package>> {
        let mut dir = root;
        dir.push("packages");
        let dir_str = dir.to_str().unwrap();

        let pkgs = read_packages_directory(dir_str)?;
        let (pkgs, _) = parse_packages(&pkgs)?;

        Ok(pkgs)
    }

    fn load_games(
        root: PathBuf,
        pkgs: &HashMap<String, Package>,
    ) -> Result<HashMap<String, Composition>> {
        let mut dir = root;
        dir.push("games");
        let dir_str = dir.to_str().unwrap();
        let games = read_compositions_directory(dir_str)?;
        parse_composition(&games, &pkgs)
    }

    fn load_assumptions(root: PathBuf) -> Result<HashMap<String, Assumption>> {
        println!("note: currently not actually reading any assumptions, as this functonality is not implemented.");
        return Ok(HashMap::new());
        /* stub for actual functionality:
        let mut dir = root.clone();
        dir.push("assumptions");
        let dir_str = dir.to_str().unwrap();
        let assumptions = read_assumptions_directory(dir_str)?;

        parse_assumptions(&assumptions)
        */
    }

    fn load_game_hops(
        root: PathBuf,
        games: &HashMap<String, Composition>,
        assumptions: &HashMap<String, Assumption>,
    ) -> Result<Vec<GameHop>> {
        let mut path = root;
        path.push("game_hops.toml");

        let filecontent = std::fs::read(&path)?;
        let game_hops = toml::from_slice::<Vec<GameHop>>(&filecontent)?;

        for (i, hop) in game_hops.iter().enumerate() {
            match hop {
                GameHop::Reduction {
                    left,
                    right,
                    assumption,
                } => {
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
                GameHop::Equivalence { left, right, .. } => {
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
                    // TODO check that invariant file exists
                }
            }
        }

        Ok(game_hops)
    }
}

/*
impl Project {
    /*
    TODO in this function:
        - respect the parameter/constants mapping
            - generate paramter functions
            - fail if types don't match across games
        - concat the snips into full smt file
            - alternatively, don't concat but step by step feed into prover.
              that way, we know about the progress and proof status and can report meaningful data to the user
    */
    pub fn build_proof(&self, proof_name: &str) -> Result<()> {
        let proof_file_path = self.proof_toml_path(proof_name);
        let toml_string = fs::read(proof_file_path)?;
        let data: ProofFile = toml::from_slice(&toml_string)?;

        let left_comp = match self.games.get(&data.left.composition) {
            Some(comp) => comp,
            None => {
                return Err(IOError::new(
                    ErrorKind::NotFound,
                    Error::CompositionMissing(data.left.composition),
                ));
            }
        };

        let right_comp = match self.games.get(&data.right.composition) {
            Some(comp) => comp,
            None => {
                return Err(IOError::new(
                    ErrorKind::NotFound,
                    Error::CompositionMissing(data.right.composition),
                ));
            }
        };

        let mut dst_file_path = self.proof_dir_path(proof_name);
        dst_file_path.push("00_preamble.smt2snip");
        let mut dst_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(dst_file_path)?;

        hacks::declare_par_Maybe(&mut dst_file)?;
        hacks::declare_Tuple(&mut dst_file, 2)?;
        write!(dst_file, "(declare-sort Bits_n 0)")?;
        write!(dst_file, "(declare-sort Bits_* 0)")?;
        write!(dst_file, "(declare-fun f (Bits_n Bits_n) Bits_n)")?;

        for comp in [left_comp, right_comp] {
            let (comp, _, samp) = crate::transforms::transform_all(&comp)?;
            let mut writer = CompositionSmtWriter::new(&comp, &samp);
            for line in writer.smt_composition_all() {
                line.write_smt_to(&mut dst_file)?;
            }
        }

        write!(dst_file, "(check-sat)")?;

        Ok(())
    }
}

*/
