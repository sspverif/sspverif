use crate::hacks;
use crate::package::{Composition, Export};
use crate::util::process::Communicator;
use crate::writers::smt::exprs::{SmtAnd, SmtAssert, SmtEq2, SmtExpr, SmtImplies, SmtNot};
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

use super::load::ProofTreeSpec;

#[derive(Debug, Serialize, Deserialize)]
pub struct Equivalence {
    pub left: String,
    pub right: String,
    pub invariant_path: String,
    pub trees: HashMap<String, ProofTreeSpec>,
}

// ResolvedEquivalence contains the composisitions/games and the invariant data,
// whereas the pure Equivalence just contains the names and file paths.
// TODO: explore if we can keep references to the games in the project hashmap
pub struct ResolvedEquivalence {
    pub left: Composition,
    pub right: Composition,
    pub invariant: String,
    pub trees: HashMap<String, ProofTreeSpec>,

    pub left_decl_smt_file: std::fs::File,
    pub right_decl_smt_file: std::fs::File,
    pub base_decl_smt_file: std::fs::File,
    pub const_decl_smt_file: std::fs::File,
    pub epilogue_smt_file: std::fs::File,
}

impl ResolvedEquivalence {
    pub fn prove(&mut self) -> Result<()> {
        //let context = ProofContext::new(self);
        let ResolvedEquivalence {
            left,
            right,
            invariant,

            left_decl_smt_file,
            right_decl_smt_file,
            base_decl_smt_file,
            const_decl_smt_file,
            epilogue_smt_file,

            trees,
        } = self;

        // check that the parameters shared by both are of the same type
        check_matching_parameters(left, right)?;

        // apply transformations
        let (comp_left, _, types_left, samp_left) =
            crate::transforms::transform_all(&left).unwrap();
        let (comp_right, _, types_right, samp_right) =
            crate::transforms::transform_all(&right).unwrap();

        // get bits types
        let mut bits_types: Vec<_> = types_left
            .union(&types_right)
            .filter_map(|t| match t {
                Type::Bits(x) => Some(x.clone()),
                _ => None,
            })
            .collect();
        bits_types.sort();

        // prepare the buffer for the data we send to the prover
        let mut base_declarations = String::new();
        let mut left_declarations = String::new();
        let mut rght_declarations = String::new();

        // write bits types declarations
        for id in bits_types {
            write!(base_declarations, "{}", hacks::BitsDeclaration(id))?;
        }

        // write other type declarations
        write!(base_declarations, "{}", hacks::MaybeDeclaration)?;
        write!(base_declarations, "{}", hacks::TuplesDeclaration(1..32))?;

        // write left game code
        let mut left_writer = CompositionSmtWriter::new(&comp_left, &samp_left);
        for line in left_writer.smt_composition_all() {
            write!(left_declarations, "{line}")?;
        }

        // write right game code
        let mut right_writer = CompositionSmtWriter::new(&comp_right, &samp_right);
        for line in right_writer.smt_composition_all() {
            write!(rght_declarations, "{line}")?;
        }

        //// Declarations
        let mut const_declarations = String::new();

        // write declaration of left (old) state constant
        let decl_state_left: SmtExpr = (
            "declare-const",
            "state-left",
            left_writer.smt_sort_composition_state(),
        )
            .into();
        write!(const_declarations, "{decl_state_left}")?;

        // write declaration of right (old) state constant
        let decl_state_right: SmtExpr = (
            "declare-const",
            "state-right",
            right_writer.smt_sort_composition_state(),
        )
            .into();
        write!(const_declarations, "{decl_state_right}")?;

        // write declarations of arguments
        for Export(_, sig) in &left.exports {
            let oracle_name = &sig.name;
            for (arg_name, arg_type) in &sig.args {
                let decl_arg: SmtExpr = (
                    "declare-const",
                    format!("arg-{oracle_name}-{arg_name}"),
                    arg_type,
                )
                    .into();
                write!(const_declarations, "{decl_arg}")?;
            }
        }

        // write declarations of left return constants and constrain them
        for Export(inst_idx, sig) in &left.exports {
            let comp_name = &left.name;
            let inst = &left.pkgs[*inst_idx];
            let inst_name = &inst.name;
            let oracle_name = &sig.name;
            let return_name = format!("return-left-{inst_name}-{oracle_name}");
            let decl_return_left: SmtExpr = (
                "declare-const",
                return_name.as_str(),
                left_writer.smt_sort_return(inst_name, oracle_name),
            )
                .into();
            write!(const_declarations, "{decl_return_left}")?;

            let mut cmdline: Vec<SmtExpr> = vec![
                format!("oracle-{comp_name}-{inst_name}-{oracle_name}").into(),
                "state-left".into(),
            ];

            for (arg_name, _arg_type) in &sig.args {
                cmdline.push(format!("arg-{oracle_name}-{arg_name}").into());
            }

            let constrain_return: SmtExpr = SmtAssert(SmtEq2 {
                lhs: return_name,
                rhs: SmtExpr::List(cmdline),
            })
            .into();
            write!(const_declarations, "{constrain_return}")?;
        }

        // write declarations of right return constants and constrain them
        for Export(inst_idx, sig) in &right.exports {
            let inst = &right.pkgs[*inst_idx];
            let inst_name = &inst.name;
            let oracle_name = &sig.name;
            let comp_name = &right.name;
            let return_name = format!("return-right-{inst_name}-{oracle_name}");
            let decl_return_right: SmtExpr = (
                "declare-const",
                return_name.as_str(),
                right_writer.smt_sort_return(inst_name, oracle_name),
            )
                .into();
            write!(const_declarations, "{decl_return_right}")?;

            let mut cmdline = vec![
                format!("oracle-{comp_name}-{inst_name}-{oracle_name}").into(),
                "state-right".into(),
            ];

            for (arg_name, _arg_type) in &sig.args {
                cmdline.push(format!("arg-{oracle_name}-{arg_name}").into());
            }

            let constrain_return: SmtExpr = SmtAssert(SmtEq2 {
                lhs: return_name,
                rhs: SmtExpr::List(cmdline),
            })
            .into();
            write!(const_declarations, "{constrain_return}")?;
        }

        // write epilogue code
        let mut epilogue = String::new();
        for (oracle_name, tree) in trees {
            write!(epilogue, "; oracle {oracle_name}\n")?;

            let sig = left
                .exports
                .iter()
                .find(|Export(_, sig)| &sig.name == oracle_name)
                .map(|Export(_, sig)| sig)
                .unwrap();

            let args: Vec<_> = sig
                .args
                .iter()
                .map(|(arg_name, _)| format!("arg-{oracle_name}-{arg_name}"))
                .collect();

            // find the package instance which is marked as exporting
            // the oracle of this name, both left and right.
            let left_return_name = left
                .exports
                .iter()
                .find(|Export(_, sig)| &sig.name == oracle_name)
                .map(|Export(inst_idx, _)| {
                    let inst_name = &left.pkgs[*inst_idx].name;
                    format!("return-left-{inst_name}-{oracle_name}")
                })
                .unwrap();

            let right_return_name = right
                .exports
                .iter()
                .find(|Export(_, sig)| &sig.name == oracle_name)
                .map(|Export(inst_idx, _)| {
                    let inst_name = &right.pkgs[*inst_idx].name;
                    format!("return-right-{inst_name}-{oracle_name}")
                })
                .unwrap();

            // this helper builds an smt expression that calls the
            // function with the given name with the old states,
            // return values and the respective arguments.
            // We expect that function to return a boolean, which makes
            // it a relation.
            let build_lemma_call = |name: String| {
                let mut tmp: Vec<SmtExpr> = vec![
                    name.into(),
                    "state-left".into(),
                    "state-right".into(),
                    left_return_name.clone().into(),
                    right_return_name.clone().into(),
                ];

                for arg in &args {
                    tmp.push(arg.clone().into());
                }

                SmtExpr::List(tmp)
            };

            /*
            implicint deps:
            - randomness mapping
            - invariant holds on old state

            (push 1)
            (assert (not (=>
                (and
                    (implicit dependencies)
                    (explicit dependencies)
                )
                (current lemma)
            )))
            (check-sat)
            (pop 1)
            */

            for (lemma_name, deps) in tree {
                write!(epilogue, "; lemma {lemma_name}\n")?;

                let mut dependencies_code: Vec<SmtExpr> = vec![
                    build_lemma_call(format!("randomness-mapping-{oracle_name}")),
                    build_lemma_call(format!("invariant-{oracle_name}")),
                ];

                for dep_name in deps {
                    dependencies_code.push(build_lemma_call(dep_name.clone()))
                }

                let code: Vec<SmtExpr> = vec![
                    ("push", "1").into(),
                    crate::writers::smt::exprs::SmtAssert(SmtNot(SmtImplies(
                        SmtAnd(dependencies_code),
                        build_lemma_call(lemma_name.clone()),
                    )))
                    .into(),
                    SmtExpr::List(vec!["check-sat".into()]),
                    ("pop", "1").into(),
                ];

                for line in code {
                    write!(epilogue, "{line}")?;
                }
            }
        }

        // write data to files
        write!(base_decl_smt_file, "{base_declarations}")?;
        write!(left_decl_smt_file, "{left_declarations}")?;
        write!(right_decl_smt_file, "{rght_declarations}")?;

        write!(const_decl_smt_file, "{const_declarations}")?;
        write!(epilogue_smt_file, "{epilogue}")?;

        // start talking to prover

        let mut prover_comm = Communicator::new_cvc4()?;

        write!(prover_comm, "{base_declarations}")?;
        write!(prover_comm, "{left_declarations}")?;
        write!(prover_comm, "{rght_declarations}")?;
        write!(prover_comm, "(check-sat)\n")?;

        println!("sent definitions, waiting for sat... ");
        expect_sat(&mut prover_comm)?;
        println!("received.");

        write!(prover_comm, "{const_declarations}")?;
        write!(prover_comm, "(check-sat)\n")?;

        println!("sent declarations and basic constraints, waiting for sat... ");
        expect_sat(&mut prover_comm)?;
        println!("received.");

        write!(prover_comm, "{invariant}").unwrap();
        write!(prover_comm, "(check-sat)\n")?;

        println!("sent invariant, waiting for sat... ");
        expect_sat(&mut prover_comm)?;
        println!("received.");

        write!(prover_comm, "{epilogue}").unwrap();
        write!(prover_comm, "(check-sat)\n")?;

        println!("sent epilogue, waiting for sat... ");
        expect_sat(&mut prover_comm)?;
        println!("received.");

        prover_comm.close();
        let rest = prover_comm.read_until_end()?;

        println!("sent everything, now receiving rest:\n{rest}");

        prover_comm.join()?;

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
