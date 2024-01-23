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

use crate::{
    gamehops::{equivalence, reduction},
    package::{Composition, Package},
    proof::{GameHop, Proof},
    transforms::{
        typecheck::{typecheck_pkg, Scope},
        Transformation,
    },
    util::prover_process::ProverBackend,
};

pub const PROJECT_FILE: &str = "ssp.toml";

pub const PACKAGES_DIR: &str = "packages";
pub const GAMES_DIR: &str = "games";
pub const PROOFS_DIR: &str = "proofs";
pub const ASSUMPTIONS_DIR: &str = "assumptions";

pub const PACKAGE_EXT: &str = ".pkg.ssp";
pub const GAME_EXT: &str = ".comp.ssp"; // TODO maybe change this to .game.ssp later, and also rename the Composition type

mod load;
mod resolve;

pub mod error;

#[derive(Debug)]
pub struct Project {
    root_dir: PathBuf,
    packages: HashMap<String, Package>,
    //assumptions: HashMap<String, Assumption>,
    games: HashMap<String, Composition>,
    //game_hops: Vec<GameHop>,
    proofs: HashMap<String, Proof>,
}

impl Project {
    pub fn load() -> Result<Project> {
        let root_dir = find_project_root()?;

        let packages = load::packages(root_dir.clone())?;
        let mut pkg_names: Vec<_> = packages.keys().collect();
        pkg_names.sort();

        for pkg_name in pkg_names.into_iter() {
            let pkg = &packages[pkg_name];
            let mut scope = Scope::new();
            typecheck_pkg(pkg, &mut scope)?;
        }

        let games = load::games(root_dir.clone(), &packages)?;
        // let mut game_names: Vec<_> = games.keys().collect();
        // game_names.sort();
        //
        // for game_name in game_names.into_iter() {
        //     let game = &games[game_name];
        //     let mut scope = Scope::new();
        //     typecheck_comp(game, &mut scope)?;
        // }

        let proofs = load::proofs(root_dir.clone(), &packages, &games)?;

        let project = Project {
            root_dir,
            packages,
            games,
            proofs,
        };

        Ok(project)
    }

    // we might want to return a proof trace here instead
    // we could then extract the proof viewer output and other useful info trom the trace
    pub fn prove(&self, backend: ProverBackend, transcript: bool) -> Result<()> {
        let mut proof_keys: Vec<_> = self.proofs.keys().collect();
        proof_keys.sort();

        for proof_key in proof_keys.into_iter() {
            let proof = &self.proofs[proof_key];
            for (i, game_hop) in proof.game_hops().iter().enumerate() {
                match game_hop {
                    GameHop::Reduction(red) => reduction::verify(red, proof)?,
                    GameHop::Equivalence(eq) => {
                        let transcript_file = if transcript {
                            Some(self.get_joined_smt_file(eq.left_name(), eq.right_name())?)
                        } else {
                            None
                        };
                        equivalence::verify(eq, proof, backend, transcript_file)?
                    }
                }

                let proof_name = proof.as_name();

                println!("proof {proof_name}: step {i}: checked");
            }
        }

        Ok(())
    }

    pub fn latex(&self) -> Result<()> {
        let mut path = self.root_dir.clone();
        path.push("_build/latex/");

        for (name, game) in &self.games {
            let (transformed, _) = crate::transforms::samplify::Transformation(game)
                .transform()
                .unwrap();
            let (transformed, _) = crate::transforms::resolveoracles::Transformation(&transformed)
                .transform()
                .unwrap();
            crate::writers::tex::writer::tex_write_composition(
                &transformed,
                &name,
                path.as_path(),
            )?;
        }

        Ok(())
    }

    /*

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

    */
    pub fn get_game<'b>(&'b self, name: &str) -> Option<&'b Composition> {
        self.games.get(name)
    }

    /*
    pub fn get_assumption<'a>(&'a self, name: &str) -> Option<&'a Assumption> {
        self.assumptions.get(name)
    }
    */

    pub fn get_root_dir(&self) -> PathBuf {
        self.root_dir.clone()
    }

    pub fn get_game_smt_file(&self, game_name: &str) -> Result<std::fs::File> {
        let mut path = self.root_dir.clone();

        path.push("_build/code_eq/games/");
        std::fs::create_dir_all(&path)?;

        path.push(format!("{game_name}.smt2"));
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        Ok(f)
    }

    pub fn get_base_decl_smt_file(
        &self,
        left_game_name: &str,
        right_game_name: &str,
    ) -> Result<std::fs::File> {
        let mut path = self.root_dir.clone();

        path.push("_build/code_eq/base_decls/");
        std::fs::create_dir_all(&path)?;

        path.push(format!("{left_game_name}_{right_game_name}.smt2"));
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        Ok(f)
    }

    pub fn get_const_decl_smt_file(
        &self,
        left_game_name: &str,
        right_game_name: &str,
    ) -> Result<std::fs::File> {
        let mut path = self.root_dir.clone();

        path.push("_build/code_eq/const_decls/");
        std::fs::create_dir_all(&path)?;

        path.push(format!("{left_game_name}_{right_game_name}.smt2"));
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        Ok(f)
    }

    pub fn get_epilogue_smt_file(
        &self,
        left_game_name: &str,
        right_game_name: &str,
    ) -> Result<std::fs::File> {
        let mut path = self.root_dir.clone();

        path.push("_build/code_eq/epilogue/");
        std::fs::create_dir_all(&path)?;

        path.push(format!("{left_game_name}_{right_game_name}.smt2"));
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        Ok(f)
    }

    pub fn get_joined_smt_file(
        &self,
        left_game_name: &str,
        right_game_name: &str,
    ) -> Result<std::fs::File> {
        let mut path = self.root_dir.clone();

        path.push("_build/code_eq/joined/");
        std::fs::create_dir_all(&path)?;

        path.push(format!("{left_game_name}_{right_game_name}.smt2"));
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
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
