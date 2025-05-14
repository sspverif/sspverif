use std::path::PathBuf;
use crate::{
    gamehops::equivalence::{
        EquivalenceContext,
        Equivalence
    },
    util::{
        prover_process::{
            ProverResponse,
            Result as ProverResponseResult
        },
        smtmodel::{
            SmtModel,
            SmtModelEntry,
        },
    },
    writers::smt::{
        contexts::GenericOracleContext,
        patterns::ReturnIsAbortConst,
        patterns::proof_constants::ConstantPattern,
    },
    proof::Claim,
    package::OracleSig,
};


#[derive(Debug)]
pub enum Error {
    UnsatAfterInvariantRead {
        equivalence: Equivalence,
        oracle_name: String,
    },
    ProverProcessError(crate::util::prover_process::Error),
    InvariantFileReadError {
        oracle_name: String,
        invariant_file_name: String,
        err: std::io::Error,
    },
    CompositionParamMismatch {
        left_game_inst_name: String,
        right_game_inst_name: String,
        mismatching_param_name: String,
    },
    ClaimProofFailed {
        claim_name: String,
        response: ProverResponse,
        modelfile: ProverResponseResult<PathBuf>,
        auxinfo: String,
    },
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ProverProcessError(err) => Some(err),
            Error::InvariantFileReadError { err, .. } => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnsatAfterInvariantRead {
                equivalence,
                oracle_name,
            } => {
                let left_game_inst_name = equivalence.left_name();
                let right_game_inst_name = equivalence.right_name();
                write!(
                    f,
                    "It seems the provided invariant file for the equivalence of \
                       game instances {left_game_inst_name} and {right_game_inst_name} \
                       contains unsatisfiable assert statements at oracle {oracle_name}. \
                       This is most likely an issue with the invariant file. \
                       Hint: Most invariant file should not contains assert statements at all."
                )
            }
            Error::ProverProcessError(err) => write!(f, "error communicating with prover: {err}"),
            Error::InvariantFileReadError {
                oracle_name,
                invariant_file_name,
                err,
            } => write!(f, "error reading invariant file {invariant_file_name} for oracle {oracle_name}: {err}"),
            Error::CompositionParamMismatch {
                left_game_inst_name,
                right_game_inst_name,
                mismatching_param_name,
            } => write!(f, "parameter {mismatching_param_name} does not match in equivalence proof of game instances {left_game_inst_name} and {right_game_inst_name}"),
            Error::ClaimProofFailed {
                claim_name,
                response,
                modelfile,
                auxinfo,
            } => {
                match modelfile {
                    Ok(model) => {
                        let model = model.as_path().display();
                        write!(f, "error proving claim {claim_name}. status: {response}. model file: {model}. Additional Information: {auxinfo}")
                    }
                    Err(model_err) => write!(f, "error proving claim {claim_name}. status: {response}. \
                                             Also, encountered the following error when trying to get the model: {model_err}"),
                }
            },
        }
    }
}

pub(crate) fn aux_info_claim_failed(claim: &Claim, model: &SmtModel, eqctx: &EquivalenceContext, sig: &OracleSig) -> String {
    let claim_name = claim.name.as_str();
    let eq = eqctx.equivalence();

    let oracle_name = &sig.name;
    
    let left_gctx = eqctx.left_game_inst_ctx();
    let left_octx = left_gctx.exported_oracle_ctx_by_name(oracle_name).unwrap();
    let left_pctx = left_octx.pkg_inst_ctx();

    let right_gctx = eqctx.right_game_inst_ctx();
    let right_octx = right_gctx.exported_oracle_ctx_by_name(oracle_name).unwrap();
    let right_pctx = right_octx.pkg_inst_ctx();

    let left_is_abort = ReturnIsAbortConst {
        game_inst_name: left_gctx.game_inst().name(),
        pkg_inst_name: left_pctx.pkg_inst_name(),
        oracle_name,
        tipe: left_octx.oracle_return_type(),
    };

    let right_is_abort = ReturnIsAbortConst {
        game_inst_name: right_gctx.game_inst().name(),
        pkg_inst_name: right_pctx.pkg_inst_name(),
        oracle_name,
        tipe: right_octx.oracle_return_type(),
    };

    match claim_name {
        "equal-aborts" => {
            let Some(SmtModelEntry::BoolEntry{name: _, value: left_aborts}) =
                model.get_value(&left_is_abort.name()) else { unreachable!() };
            let Some(SmtModelEntry::BoolEntry{name: _, value: right_aborts}) =
                model.get_value(&right_is_abort.name()) else { unreachable!() };
            if left_aborts {
                format!("Oracle {}: {} aborts but {} does not",
                        oracle_name,
                        left_gctx.game_inst().name(),
                        right_gctx.game_inst().name())
            } else {
                format!("Oracle {}: {} aborts but {} does not",
                        oracle_name,
                        right_gctx.game_inst().name(),
                        left_gctx.game_inst().name())
            }
        }
        "no-abort" => {
            let mut result = String::new();
            let Some(SmtModelEntry::BoolEntry{name: _, value: left_aborts}) =
                model.get_value(&left_is_abort.name()) else { unreachable!() };
            let Some(SmtModelEntry::BoolEntry{name: _, value: right_aborts}) =
                model.get_value(&right_is_abort.name()) else { unreachable!() };
            if left_aborts {
                result.push_str(
                    &format!("Oracle {}: {} aborts; ",
                             oracle_name,
                             left_gctx.game_inst().name()));
            }
            if right_aborts {
                result.push_str(
                    &format!("Oracle {}: {} aborts; ",
                             oracle_name,
                             right_gctx.game_inst().name()));
            }
            result
        }
        _ => {"".to_string()}
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<crate::util::prover_process::Error> for Error {
    fn from(err: crate::util::prover_process::Error) -> Self {
        new_prover_process_error(err)
    }
}

pub(crate) fn new_prover_process_error(err: crate::util::prover_process::Error) -> Error {
    Error::ProverProcessError(err)
}

pub(crate) fn new_invariant_file_read_error(
    oracle_name: String,
    invariant_file_name: String,
    err: std::io::Error,
) -> Error {
    Error::InvariantFileReadError {
        oracle_name,
        invariant_file_name,
        err,
    }
}
