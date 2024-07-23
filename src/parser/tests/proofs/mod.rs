use std::collections::HashMap;

use crate::{
    package::{Composition, Package},
    parser::{
        proof::{handle_proof, ParseProofError},
        SspParser,
    },
    proof::Proof,
};
pub fn parse(
    code: &str,
    name: &str,
    pkgs: &HashMap<String, Package>,
    games: &HashMap<String, Composition>,
) -> Proof {
    let mut proof_pairs = SspParser::parse_proof(code).unwrap();
    handle_proof(name, code, proof_pairs.next().unwrap(), pkgs, games)
        .unwrap_or_else(|err| panic!("handle error: {err}", err = err))
}

pub fn parse_fails(
    code: &str,
    name: &str,
    pkgs: &HashMap<String, Package>,
    games: &HashMap<String, Composition>,
) -> ParseProofError {
    // any test game should adhere to the grammar
    let mut proof_pairs = SspParser::parse_proof(code).unwrap();

    handle_proof(name, code, proof_pairs.next().unwrap(), pkgs, games).expect_err(&format!(
        "expected an error when parsing {name}, but it succeeded"
    ))
}

pub fn parse_file(
    file_name: &'static str,
    pkgs: &HashMap<String, Package>,
    games: &HashMap<String, Composition>,
) -> Proof {
    let file = std::fs::File::open(format!("src/parser/tests/proofs/{file_name}"))
        .unwrap_or_else(|_| panic!("error opening test code proof {}", file_name));

    let contents = std::io::read_to_string(file)
        .unwrap_or_else(|_| panic!("error reading test code proof {}", file_name));

    parse(&contents, file_name, pkgs, games)
}

pub fn parse_file_fails(
    file_name: &'static str,
    pkgs: &HashMap<String, Package>,
    games: &HashMap<String, Composition>,
) -> ParseProofError {
    let file = std::fs::File::open(format!("src/parser/tests/proofs/{file_name}"))
        .unwrap_or_else(|_| panic!("error opening test code proof {}", file_name));

    let contents = std::io::read_to_string(file)
        .unwrap_or_else(|_| panic!("error reading test code proof {}", file_name));

    parse_fails(&contents, file_name, pkgs, games)
}
