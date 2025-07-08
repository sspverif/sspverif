use std::fmt::Write;
use std::io::Write as _;

use crate::{
    gamehops::equivalence::{
        error::{Error, Result},
        Equivalence,
    },
    proof::Proof,
    transforms::{proof_transforms::EquivalenceTransform, ProofTransform},
    util::prover_process::{Communicator, ProverResponse},
    writers::smt::exprs::SmtExpr,
};

use super::EquivalenceContext;

pub fn verify(eq: &Equivalence, proof: &Proof, mut prover: Communicator, req_oracle: &Option<String>) -> Result<()> {
    let (proof, auxs) = EquivalenceTransform.transform_proof(proof).unwrap();

    let eqctx = EquivalenceContext {
        equivalence: eq,
        proof: &proof,
        auxs: &auxs,
    };

    std::thread::sleep(std::time::Duration::from_millis(20));

    prover.write_smt(SmtExpr::Comment("\n".to_string()))?;
    prover.write_smt(SmtExpr::Comment("base declarations:\n".to_string()))?;
    eqctx.emit_base_declarations(&mut prover)?;
    prover.write_smt(SmtExpr::Comment("\n".to_string()))?;
    prover.write_smt(SmtExpr::Comment("proof param funcs:\n".to_string()))?;
    eqctx.emit_proof_paramfuncs(&mut prover)?;
    prover.write_smt(SmtExpr::Comment("\n".to_string()))?;
    prover.write_smt(SmtExpr::Comment("game definitions:\n".to_string()))?;
    eqctx.emit_game_definitions(&mut prover)?;
    eqctx.emit_constant_declarations(&mut prover)?;

    for oracle_sig in eqctx.oracle_sequence() {
        if let Some(ref req_oracle) = req_oracle {
            if *req_oracle != oracle_sig.name {
                continue;
            }
        }
        println!("verify: oracle:{oracle_sig:?}");
        write!(prover, "(push 1)").unwrap();
        eqctx.emit_return_value_helpers(&mut prover, &oracle_sig.name)?;
        eqctx.emit_invariant(&mut prover, &oracle_sig.name)?;

        for claim in eqctx
            .equivalence
            .proof_tree_by_oracle_name(&oracle_sig.name)
        {
            write!(prover, "(push 1)").unwrap();
            eqctx.emit_claim_assert(&mut prover, &oracle_sig.name, &claim)?;
            match prover.check_sat()? {
                ProverResponse::Unsat => {}
                response => {
                    let modelfile = prover.get_model().map(|model| {
                        let mut modelfile = tempfile::NamedTempFile::new().unwrap();
                        modelfile.write_all(model.as_bytes()).unwrap();
                        let (_, fname) = modelfile.keep().unwrap();
                        fname
                    });
                    return Err(Error::ClaimProofFailed {
                        claim_name: claim.name().to_string(),
                        response,
                        modelfile,
                    });
                }
            }
            write!(prover, "(pop 1)").unwrap();
        }

        write!(prover, "(pop 1)").unwrap();
    }

    // for split_oracle_sig in eqctx.split_oracle_sequence() {
    //     println!("verify: split oracle:{split_oracle_sig:?}");
    //     write!(prover, "(push 1)").unwrap();
    //     eqctx.emit_invariant(&mut prover, &split_oracle_sig.name)?;
    //
    //     for claim in eqctx
    //         .equivalence
    //         .proof_tree_by_oracle_name(&split_oracle_sig.name)
    //     {
    //         write!(prover, "(push 1)").unwrap();
    //         eqctx.emit_split_claim_assert(
    //             &mut prover,
    //             &split_oracle_sig.name,
    //             &split_oracle_sig.path,
    //             &claim,
    //         )?;
    //
    //         match prover.check_sat()? {
    //             ProverResponse::Unsat => {}
    //             response => {
    //                 return Err(Error::ClaimProofFailed {
    //                     claim_name: claim.name().to_string(),
    //                     response,
    //                     model: prover.get_model(),
    //                 });
    //             }
    //         }
    //         write!(prover, "(pop 1)").unwrap();
    //     }
    //
    //     write!(prover, "(pop 1)").unwrap();
    // }

    Ok(())
}
