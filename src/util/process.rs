//use crate::project::error::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("error parsing utf8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

use std::io::Read;
use std::io::Write as IOWrite;
use std::mem::swap;

pub struct Communicator {
    stdout: std::process::ChildStdout,
    chan: Option<std::sync::mpsc::Sender<String>>,
    thrd: Option<std::thread::JoinHandle<Result<()>>>,
    buf: Vec<u8>,
    pos: usize,
}

impl Communicator {
    pub fn new_z3() -> Result<Self> {
        let mut cmd = std::process::Command::new("z3");
        cmd.args(["-in", "-smt2"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

        Self::new_from_cmd(cmd)
    }

    pub fn new_cvc4() -> Result<Self> {
        let mut cmd = std::process::Command::new("cvc4");
        cmd.args(["--lang=smt2", "--incremental", "--produce-models"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

        Self::new_from_cmd(cmd)
    }

    pub fn new_from_cmd(mut cmd: std::process::Command) -> Result<Self> {
        let cmd = cmd.spawn()?;

        let (send, recv) = std::sync::mpsc::channel();

        let mut stdin = cmd.stdin.unwrap();
        let stdout = cmd.stdout.unwrap();

        let mut debug = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open("/tmp/debugfile")?;

        let thrd = std::thread::spawn(move || {
            for data in recv {
                write!(stdin, "{data}")?;
                write!(debug, "{data}")?;
                stdin.flush()?;
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

    pub fn read_until_pred<T, P: Fn(usize, &str) -> (usize, Option<T>)>(
        &mut self,
        p: P,
    ) -> Result<T> {
        loop {
            let read_cnt = self.stdout.read(&mut self.buf[self.pos..])?;
            self.pos += read_cnt;

            let data = String::from_utf8(self.buf[..self.pos].to_vec())?;
            if let (data_read, Some(v)) = p(read_cnt, &data) {
                let rest_bs = data[data_read..].as_bytes().to_owned();

                self.buf.fill(0);
                self.pos = rest_bs.len();
                self.buf[..self.pos].copy_from_slice(&rest_bs);

                return Ok(v);
            }
        }
    }

    pub fn read_until(&mut self, pattern: &str) -> Result<(usize, String)> {
        loop {
            self.pos += self.stdout.read(&mut self.buf[self.pos..])?;
            let data = String::from_utf8(self.buf[..self.pos].to_vec())?;

            if let Some(match_start) = data.find(pattern) {
                let match_end = match_start + pattern.len();

                let ret = data[..match_end].to_string();
                let rest_bs = data[match_end..].as_bytes();

                self.buf.fill(0);
                self.pos = rest_bs.len();
                self.buf[..self.pos].copy_from_slice(&rest_bs);

                return Ok((match_start, ret));
            }
        }
    }

    pub fn read_until_end(&mut self) -> Result<String> {
        let mut data = String::from_utf8(self.buf[..self.pos].to_vec())?;
        self.stdout.read_to_string(&mut data)?;
        Ok(data)
    }

    pub fn close(&mut self) {
        let mut none = None;
        swap(&mut self.chan, &mut none)
    }

    pub fn join(&mut self) -> Result<()> {
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
