use std::fmt::Write;
use std::fs::File;

use crate::{
    gamehops::equivalence::error::{Error, Result},
    proof::{Equivalence, Proof},
    transforms::{proof_transforms::EquivalenceTransform, ProofTransform},
    util::prover_process::{Communicator, ProverBackend, ProverResponse},
};

use super::EquivalenceContext;

pub fn verify(
    eq: &Equivalence,
    proof: &Proof,
    backend: ProverBackend,
    transcript_file: Option<File>,
) -> Result<()> {
    let (proof, auxs) = EquivalenceTransform.transform_proof(proof).unwrap();

    let eqctx = EquivalenceContext {
        eq,
        proof: &proof,
        auxs: &auxs,
    };

    let mut prover = Communicator::new(backend, transcript_file)?;

    std::thread::sleep(std::time::Duration::from_millis(20));

    eqctx.emit_base_declarations(&mut prover)?;
    eqctx.emit_game_definitions(&mut prover)?;
    eqctx.emit_constant_declarations(&mut prover)?;

    for oracle_sig in eqctx.oracle_sequence() {
        println!("verify: oracle:{oracle_sig:?}");
        write!(prover, "(push 1)").unwrap();
        eqctx.emit_invariant(&mut prover, &oracle_sig.name)?;

        for claim in eqctx.eq.proof_tree_by_oracle_name(&oracle_sig.name) {
            write!(prover, "(push 1)").unwrap();
            eqctx.emit_claim_assert(&mut prover, &oracle_sig.name, &claim)?;
            match prover.check_sat()? {
                ProverResponse::Unsat => {}
                response => {
                    return Err(Error::ClaimProofFailed {
                        claim_name: claim.name().to_string(),
                        response,
                        model: prover.get_model(),
                    });
                }
            }
            write!(prover, "(pop 1)").unwrap();
        }

        write!(prover, "(pop 1)").unwrap();
    }

    for split_oracle_sig in eqctx.split_oracle_sequence() {
        println!("verify: split oracle:{split_oracle_sig:?}");
        write!(prover, "(push 1)").unwrap();
        eqctx.emit_invariant(&mut prover, &split_oracle_sig.name)?;

        for claim in eqctx.eq.proof_tree_by_oracle_name(&split_oracle_sig.name) {
            write!(prover, "(push 1)").unwrap();
            eqctx.emit_split_claim_assert(
                &mut prover,
                &split_oracle_sig.name,
                &split_oracle_sig.path,
                &claim,
            )?;

            match prover.check_sat()? {
                ProverResponse::Unsat => {}
                response => {
                    return Err(Error::ClaimProofFailed {
                        claim_name: claim.name().to_string(),
                        response,
                        model: prover.get_model(),
                    });
                }
            }
            write!(prover, "(pop 1)").unwrap();
        }

        write!(prover, "(pop 1)").unwrap();
    }

    Ok(())
}
