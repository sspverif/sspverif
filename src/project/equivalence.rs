use crate::hacks;
use crate::writers::smt::writer::CompositionSmtWriter;
use std::io::Write;
use std::{collections::HashSet, io::Read};

use super::error::{Error, Result};
use std::iter::FromIterator;

use std::collections::HashMap;

use crate::package::Composition;
use crate::writers::smt::exprs::SmtFmt;

use serde_derive::{Deserialize, Serialize};

use std::path::{Path, PathBuf};

use super::Project;

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
        let invariant = std::fs::read(inv_path)?;

        Ok(ResolvedEquivalence {
            left,
            right,
            invariant,
        })
    }
}

// ResolvedEquivalence contains the composisitions/games and the invariant data,
// whereas the pure Equivalence just contains the names and file paths.
// TODO: explore if we can keep references to the games in the project hashmap
pub struct ResolvedEquivalence {
    left: Composition,
    right: Composition,
    invariant: Vec<u8>,
}

impl ResolvedEquivalence {
    pub fn prove(&self) -> Result<()> {
        let ResolvedEquivalence {
            left,
            right,
            invariant,
        } = self;

        // check that the parameters shared by both are of the same type
        check_matching_parameters(left, right)?;

        let z3_proc = std::process::Command::new("z3")
            .args(["-in", "-smt2"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()?;

        let mut z3_stdin = z3_proc.stdin.unwrap();

        // prepare the data we send to the prover
        let mut buf = Vec::<u8>::new();

        hacks::declare_par_Maybe(&mut buf)?;
        hacks::declare_Tuple(&mut buf, 2)?;
        write!(buf, "(declare-sort Bits_n 0)")?;
        write!(buf, "(declare-sort Bits_* 0)")?;

        /*
        TODO generate Bits_x sorts
        TODO generate paramter functions
            - i.e. encryption, prf, ...
            - this already exists on the other branch, should be easy
        */

        for comp in [left, right] {
            let (comp, _, samp) = crate::transforms::transform_all(&comp).unwrap();
            let mut writer = CompositionSmtWriter::new(&comp, samp);
            for line in writer.smt_composition_all() {
                line.write_smt_to(&mut buf)?;
            }
        }

        write!(buf, "(check-sat)")?;

        let (writer, rx) = std::sync::mpsc::channel::<Vec<u8>>();

        // writer thread
        std::thread::spawn(move || {
            for data in rx {
                if let Err(e) = z3_stdin.write_all(&data) {
                    eprintln!("error in writer thread: {e}");
                }
            }
        });

        writer.send(buf).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(1250));

        let mut z3_outbuf = [0u8; 1 << 14];
        let mut z3_stdout = z3_proc.stdout.unwrap();
        let mut bytes_read = 0;
        let mut z3_outstr: String;
        loop {
            bytes_read += z3_stdout.read(&mut z3_outbuf[bytes_read..])?;

            z3_outstr = String::from_utf8(z3_outbuf[..bytes_read].to_vec()).unwrap();
            print!("read {bytes_read} bytes from z3: {z3_outstr}");

            if z3_outstr.ends_with("sat") {
                break;
            }
        }

        println!("data: {z3_outstr}.");

        /*
        // send data to prover and handle response
        match proc.communicate_bytes(Some(&buf))? {
            (Some(stdout_data), Some(stderr_data)) => {
                let data = String::from_utf8(stdout_data).unwrap();
                let err = String::from_utf8(stderr_data).unwrap();
                println!("data: {data}.");
                println!("err: {err}.");
            }
            _ => {
                unreachable!("")
            }
        } */

        // TODO handle response, don't really expect any
        writer.send(self.invariant.clone()).unwrap();

        // TODO write epilogue and handle response

        Ok(())
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
