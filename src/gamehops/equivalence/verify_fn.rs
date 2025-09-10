// SPDX-License-Identifier: MIT OR Apache-2.0

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fmt::Write;
use std::io::Write as _;
use std::sync::{Arc, Mutex};

use crate::ui::ProofUI;
use crate::{
    gamehops::equivalence::{
        error::{Error, Result},
        Equivalence,
    },
    package::OracleSig,
    project::Project,
    proof::Proof,
    transforms::{proof_transforms::EquivalenceTransform, ProofTransform},
    util::prover_process::{Communicator, ProverBackend, ProverResponse},
    writers::smt::exprs::SmtExpr,
};

use super::EquivalenceContext;

fn verify_oracle<UI: ProofUI>(
    project: &Project,
    ui: Arc<Mutex<&mut UI>>,
    eqctx: &EquivalenceContext,
    backend: ProverBackend,
    transcript: bool,
    req_oracles: &[&OracleSig],
) -> Result<()> {
    let eq = eqctx.equivalence();
    let proofstep_name = format!("{} == {}", eq.left_name(), eq.right_name());

    let mut prover = if transcript {
        let oracle = if req_oracles.len() == 1 {
            Some(req_oracles[0].name.as_str())
        } else {
            None
        };

        let transcript_file: std::fs::File = project
            .get_joined_smt_file(eq.left_name(), eq.right_name(), oracle)
            .unwrap();

        Communicator::new_with_transcript(backend, transcript_file)?
    } else {
        Communicator::new(backend)?
    };
    std::thread::sleep(std::time::Duration::from_millis(20));

    log::debug!(
        "emitting base declarations for {}-{}",
        eq.left_name,
        eq.right_name
    );
    prover.write_smt(SmtExpr::Comment("\n".to_string()))?;
    prover.write_smt(SmtExpr::Comment("base declarations:\n".to_string()))?;
    eqctx.emit_base_declarations(&mut prover)?;
    log::debug!(
        "emitting proof paramfuncs for {}-{}",
        eq.left_name,
        eq.right_name
    );
    prover.write_smt(SmtExpr::Comment("\n".to_string()))?;
    prover.write_smt(SmtExpr::Comment("proof param funcs:\n".to_string()))?;
    eqctx.emit_proof_paramfuncs(&mut prover)?;
    log::debug!(
        "emitting game definitions for {}-{}",
        eq.left_name,
        eq.right_name
    );
    prover.write_smt(SmtExpr::Comment("\n".to_string()))?;
    prover.write_smt(SmtExpr::Comment("game definitions:\n".to_string()))?;
    eqctx.emit_game_definitions(&mut prover)?;
    log::debug!(
        "emitting const declarations for {}-{}",
        eq.left_name,
        eq.right_name
    );
    eqctx.emit_constant_declarations(&mut prover)?;

    for oracle_sig in req_oracles {
        let claims = eqctx
            .equivalence
            .proof_tree_by_oracle_name(&oracle_sig.name);
        ui.lock().unwrap().start_oracle(
            eqctx.proof().as_name(),
            &proofstep_name,
            &oracle_sig.name,
            claims.len().try_into().unwrap(),
        );

        log::info!("verify: oracle:{oracle_sig:?}");
        write!(prover, "(push 1)").unwrap();
        eqctx.emit_return_value_helpers(&mut prover, &oracle_sig.name)?;
        eqctx.emit_invariant(&mut prover, &oracle_sig.name)?;

        for claim in claims {
            ui.lock().unwrap().start_lemma(
                eqctx.proof().as_name(),
                &proofstep_name,
                &oracle_sig.name,
                claim.name(),
            );

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
                        oracle_name: oracle_sig.name.clone(),
                        response,
                        modelfile,
                    });
                }
            }
            write!(prover, "(pop 1)").unwrap();
            ui.lock().unwrap().finish_lemma(
                eqctx.proof().as_name(),
                &proofstep_name,
                &oracle_sig.name,
                claim.name(),
            );
        }

        write!(prover, "(pop 1)").unwrap();
        ui.lock().unwrap().finish_oracle(
            eqctx.proof().as_name(),
            &proofstep_name,
            &oracle_sig.name,
        );
    }
    Ok(())
}

pub fn verify<UI: ProofUI>(
    project: &Project,
    ui: &mut UI,
    eq: &Equivalence,
    orig_proof: &Proof,
    backend: ProverBackend,
    transcript: bool,
    req_oracle: &Option<String>,
) -> Result<()> {
    let (proof, auxs) = EquivalenceTransform.transform_proof(orig_proof).unwrap();

    let eqctx = EquivalenceContext {
        equivalence: eq,
        proof: &proof,
        auxs: &auxs,
    };

    let proofstep_name = format!("{} == {}", eq.left_name(), eq.right_name());
    let oracle_sequence: Vec<_> = eqctx
        .oracle_sequence()
        .into_iter()
        .filter(|sig| {
            if let Some(name) = req_oracle {
                sig.name == *name
            } else {
                true
            }
        })
        .collect();

    ui.proofstep_set_oracles(
        proof.as_name(),
        &proofstep_name,
        oracle_sequence.len().try_into().unwrap(),
    );

    let ui = Arc::new(Mutex::new(ui));

    verify_oracle(project, ui, &eqctx, backend, transcript, &oracle_sequence)?;

    Ok(())
}

pub fn verify_parallel<UI: ProofUI + std::marker::Send>(
    project: &Project,
    ui: &mut UI,
    eq: &Equivalence,
    orig_proof: &Proof,
    backend: ProverBackend,
    transcript: bool,
    parallel: usize,
    req_oracle: &Option<String>,
) -> crate::project::error::Result<()> {
    let (proof, auxs) = EquivalenceTransform.transform_proof(orig_proof).unwrap();

    let eqctx = EquivalenceContext {
        equivalence: eq,
        proof: &proof,
        auxs: &auxs,
    };

    let proofstep_name = format!("{} == {}", eq.left_name(), eq.right_name());
    let oracle_sequence: Vec<_> = eqctx
        .oracle_sequence()
        .into_iter()
        .filter(|sig| {
            if let Some(name) = req_oracle {
                sig.name == *name
            } else {
                true
            }
        })
        .collect();

    ui.proofstep_set_oracles(
        proof.as_name(),
        &proofstep_name,
        oracle_sequence.len().try_into().unwrap(),
    );

    let ui = Arc::new(Mutex::new(ui));

    rayon::ThreadPoolBuilder::new()
        .num_threads(parallel + 1) // one process is reserved for the "main" method
        .build()
        .unwrap()
        .install(|| -> crate::project::error::Result<()> {
            let result_count = oracle_sequence
                .par_iter()
                .map(|oracle_sig| -> Result<()> {
                    let result = verify_oracle(
                        project,
                        ui.clone(),
                        &eqctx,
                        backend,
                        transcript,
                        &[*oracle_sig],
                    );
                    if let Err(ref e) = result {
                        ui.lock().unwrap().println(&format!("{e}")).unwrap();
                    }
                    result
                })
                .filter(Result::is_err)
                .count();
            if result_count == 0 {
                Ok(())
            } else {
                Err(crate::project::error::Error::ParallelEquivalenceError(
                    result_count,
                ))
            }
        })
}
