use std::fmt;
use std::fmt::Write;
use std::result;

use thiserror::Error;

use super::process;

pub struct Communicator(process::Communicator);

#[derive(Debug, Error)]
pub enum Error {
    #[error("error writing to prover process: {0}")]
    WriteError(#[from] std::fmt::Error),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("error interactiv with prover process: {0}")]
    ProcessError(#[from] super::process::Error),
    #[error("prover error: {0}")]
    ProverError(String),
}

#[derive(Debug)]
pub enum ProverResponse {
    Sat,
    Unsat,
    Unknown,
}

impl fmt::Display for ProverResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProverResponse::Sat => write!(f, "sat"),
            ProverResponse::Unsat => write!(f, "unsat"),
            ProverResponse::Unknown => write!(f, "unknown"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

impl Communicator {
    pub fn new_z3() -> Result<Self> {
        let mut cmd = std::process::Command::new("z3");
        cmd.args(["-in", "-smt2"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

        Ok(Self(process::Communicator::new_from_cmd(cmd)?))
    }

    pub fn new_cvc4() -> Result<Self> {
        let mut cmd = std::process::Command::new("cvc4");
        cmd.args(["--lang=smt2", "--incremental", "--produce-models"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

        Ok(Self(process::Communicator::new_from_cmd(cmd)?))
    }

    pub fn check_sat(&mut self) -> Result<ProverResponse> {
        writeln!(self, "(check-sat)")?;

        let pred =
            |_: usize, data: &str| -> (usize, Option<result::Result<ProverResponse, Error>>) {
                let is_err_start = data.starts_with(r#"(error ""#);
                let is_err_end = data.ends_with(")\n");
                if data.starts_with("sat\n") {
                    return (4, Some(Ok(ProverResponse::Sat)));
                } else if data.starts_with("unsat\n") {
                    return (6, Some(Ok(ProverResponse::Unsat)));
                } else if data.starts_with("unknown\n") {
                    return (6, Some(Ok(ProverResponse::Unknown)));
                } else if is_err_start && is_err_end {
                    return (data.len(), Some(Err(Error::ProverError(data.to_string()))));
                } else {
                    return (0, None);
                }
            };

        let resp = self.0.read_until_pred(pred)??;
        Ok(resp)
    }

    pub fn read_until_end(&mut self) -> Result<String> {
        Ok(self.0.read_until_end()?)
    }

    pub fn close(&mut self) {
        self.0.close();
    }

    pub fn join(&mut self) -> Result<()> {
        Ok(self.0.join()?)
    }
}

impl std::fmt::Write for Communicator {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_str(s)
    }
}
