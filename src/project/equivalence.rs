use crate::hacks;
use crate::package::Composition;
use crate::util::process::Communicator;
use crate::writers::smt::writer::CompositionSmtWriter;
use crate::{
    project::error::{Error, Result},
    types::Type,
};

use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::io::Write as IOWrite;
use std::iter::FromIterator;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Equivalence {
    pub left: String,
    pub right: String,
    pub invariant_path: String,
}

// ResolvedEquivalence contains the composisitions/games and the invariant data,
// whereas the pure Equivalence just contains the names and file paths.
// TODO: explore if we can keep references to the games in the project hashmap
pub struct ResolvedEquivalence {
    pub left: Composition,
    pub right: Composition,
    pub invariant: String,

    pub left_smt_file: std::fs::File,
    pub right_smt_file: std::fs::File,
    pub decl_smt_file: std::fs::File,
}

impl ResolvedEquivalence {
    pub fn prove(&mut self) -> Result<()> {
        let ResolvedEquivalence {
            left,
            right,
            invariant,
            ref mut left_smt_file,
            ref mut right_smt_file,
            decl_smt_file,
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
