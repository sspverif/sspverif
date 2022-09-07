use crate::{
    project::{
        error::{Error, Result},
        Project,
    },
    types::Type,
};

use crate::hacks;
use crate::package::Composition;
use crate::writers::smt::writer::CompositionSmtWriter;

use std::fmt::Write;
use std::io::Write as IOWrite;
use std::iter::FromIterator;
use std::mem::swap;
use std::path::{Path, PathBuf};
use std::{
    collections::{HashMap, HashSet},
    io::Read,
};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Equivalence {
    pub left: String,
    pub right: String,
    pub invariant_path: String,
}

impl Equivalence {
    // returns the invariant path. if it is an absolute path it jsut returns it,
    // if it is a relative path, it is returned relative to project_root.
    pub fn get_invariant_path(&self, project_root: &Path) -> PathBuf {
        let path = PathBuf::from(self.invariant_path.clone());
        if path.is_absolute() {
            path
        } else {
            project_root.join(path)
        }
    }

    // resolve fetches the compositions/games from the project and reads the invariant file.
    // this way, the data is neatly available when we want to operate on it
    pub fn resolve(&self, project: &Project) -> Result<ResolvedEquivalence> {
        let left = match project.get_game(&self.left) {
            Some(game) => game.clone(),
            _ => return Err(Error::CompositionMissing(self.left.clone())),
        };

        let right = match project.get_game(&self.right) {
            Some(game) => game.clone(),
            _ => return Err(Error::CompositionMissing(self.right.clone())),
        };

        let inv_path = self.get_invariant_path(&project.get_root_dir());
        let invariant = std::fs::read_to_string(inv_path)?;

        let left_smt_file = project.get_smt_game_file(&self.left)?;
        let right_smt_file = project.get_smt_game_file(&self.right)?;
        let decl_smt_file = project.get_smt_decl_file(&self.left, &self.right)?;

        Ok(ResolvedEquivalence {
            left,
            right,
            invariant,
            left_smt_file,
            right_smt_file,
            decl_smt_file,
        })
    }
}

// ResolvedEquivalence contains the composisitions/games and the invariant data,
// whereas the pure Equivalence just contains the names and file paths.
// TODO: explore if we can keep references to the games in the project hashmap
pub struct ResolvedEquivalence {
    left: Composition,
    right: Composition,
    invariant: String,

    left_smt_file: std::fs::File,
    right_smt_file: std::fs::File,
    decl_smt_file: std::fs::File,
}

impl ResolvedEquivalence {
    pub fn prove(&mut self) -> Result<()> {
        let ResolvedEquivalence {
            left,
            right,
            invariant,
            ref mut left_smt_file,
            ref mut right_smt_file,
            ref mut decl_smt_file,
        } = self;

        // check that the parameters shared by both are of the same type
        check_matching_parameters(left, right)?;

        // apply transformations
        let (comp_left, _, types_left, samp_left) =
            crate::transforms::transform_all(&left).unwrap();
        let (comp_right, _, types_right, samp_right) =
            crate::transforms::transform_all(&right).unwrap();

        // get bits types
        let bits_types = types_left.union(&types_right).filter_map(|t| match t {
            Type::Bits(x) => Some(x.clone()),
            _ => None,
        });

        // prepare the buffer for the data we send to the prover
        let mut definitions = String::new();

        // write bits types declarations
        for id in bits_types {
            write!(definitions, "{}", hacks::BitsDeclaration(id.clone()))?;
            write!(decl_smt_file, "{}", hacks::BitsDeclaration(id))?;
        }

        // write other type declarations
        write!(definitions, "{}", hacks::MaybeDeclaration)?;
        write!(decl_smt_file, "{}", hacks::MaybeDeclaration)?;
        write!(definitions, "{}", hacks::TuplesDeclaration(1..32))?;
        write!(decl_smt_file, "{}", hacks::TuplesDeclaration(1..32))?;

        // write left game code
        let mut left_writer = CompositionSmtWriter::new(&comp_left, &samp_left);
        for line in left_writer.smt_composition_all() {
            write!(left_smt_file, "{line}")?;
            write!(definitions, "{line}")?;
        }

        // write right game code
        let mut right_writer = CompositionSmtWriter::new(&comp_right, &samp_right);
        for line in right_writer.smt_composition_all() {
            write!(right_smt_file, "{line}")?;
            write!(definitions, "{line}")?;
        }

        ///////// start talking to prover

        let mut z3_comm = Communicator::new_z3()?;

        write!(z3_comm, "{definitions}")?;
        write!(z3_comm, "(check-sat)\n")?;

        println!("sent definitions, waiting for sat... ");
        expect_sat(&mut z3_comm)?;
        println!("received.");

        write!(z3_comm, "{invariant}").unwrap();
        write!(z3_comm, "(check-sat)\n")?;

        println!("sent invariant, waiting for sat... ");
        expect_sat(&mut z3_comm)?;
        println!("received.");

        // TODO write epilogue and handle response

        z3_comm.close();
        z3_comm.join()?;

        Ok(())
    }
}

fn expect_sat(comm: &mut Communicator) -> Result<()> {
    let (_, read) = comm.read_until("sat\n")?;
    expect_sat_str(&read)
}

fn expect_sat_str(data: &str) -> Result<()> {
    match data {
        "sat\n" => Ok(()),
        _ => Err(Error::ExpectedSatError(data.to_string())),
    }
}

struct Communicator {
    stdout: std::process::ChildStdout,
    chan: Option<std::sync::mpsc::Sender<String>>,
    thrd: Option<std::thread::JoinHandle<Result<()>>>,
    buf: Vec<u8>,
    pos: usize,
}

impl Communicator {
    fn new_z3() -> Result<Self> {
        let cmd = std::process::Command::new("z3")
            .args(["-in", "-smt2"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()?;

        let (send, recv) = std::sync::mpsc::channel();

        let mut stdin = cmd.stdin.unwrap();
        let stdout = cmd.stdout.unwrap();

        let thrd = std::thread::spawn(move || {
            for data in recv {
                write!(stdin, "{data}")?;
            }
            Ok(())
        });

        let buf = vec![0u8; 1 << 20];

        Ok(Communicator {
            stdout,
            chan: Some(send),
            thrd: Some(thrd),
            buf,
            pos: 0,
        })
    }

    fn read_until(&mut self, pattern: &str) -> Result<(usize, String)> {
        loop {
            self.pos += self.stdout.read(&mut self.buf[self.pos..])?;
            let data = String::from_utf8(self.buf[..self.pos].to_vec())?;

            if let Some(match_start) = data.find(pattern) {
                let match_end = match_start + pattern.len();

                let ret = data[..match_end].to_string();
                let rest_bs = data[match_end..].as_bytes();

                self.buf.fill(0);
                let written = self.buf.write(rest_bs)?;
                self.pos = written;

                return Ok((match_start, ret));
            }
        }
    }

    fn close(&mut self) {
        let mut none = None;
        swap(&mut self.chan, &mut none)
    }

    fn join(&mut self) -> Result<()> {
        if let None = self.thrd {
            return Ok(());
        }

        let mut thrd = None;
        swap(&mut thrd, &mut self.thrd);

        thrd.unwrap().join().expect("error joining thread")
    }
}

impl std::fmt::Write for Communicator {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if let Some(ref chan) = self.chan {
            if let Err(_) = chan.send(s.to_string()) {
                return Err(std::fmt::Error);
            } else {
                return Ok(());
            }
        }

        panic!("writing to closed communicator");
    }
}

// This function gets the parameter names that both have and checks that
// both use the same types.
// The invariant should be used to make assertions on equality or other
// relations between the two.
//
// TODO figure out if there is a better mechanism we could use here
fn check_matching_parameters(left: &Composition, right: &Composition) -> Result<()> {
    use std::collections::hash_map::RandomState;

    // populate tables name -> type
    let left_params: HashMap<_, _, RandomState> =
        HashMap::from_iter(left.consts.clone().into_iter());
    let right_params: HashMap<_, _, RandomState> =
        HashMap::from_iter(right.consts.clone().into_iter());

    // prepare sets of names
    let left_params_set = HashSet::<_, RandomState>::from_iter(left_params.keys());
    let right_params_set = HashSet::<_, RandomState>::from_iter(right_params.keys());
    let common_params = left_params_set.intersection(&right_params_set);

    // check that the common params have the same type
    for param in common_params {
        if left_params[*param] != right_params[*param] {
            return Err(Error::CompositionParamMismatch(
                left.name.clone(),
                right.name.clone(),
                (*param).clone(),
            ));
        }
    }

    Ok(())
}
