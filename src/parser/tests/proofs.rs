use std::collections::HashMap;

use crate::{
    package::{Composition, Package},
    parser::{
        proof::{handle_proof, ParseProofError},
        tests::TESTDATA_SSPCODE_PATH,
        SspParser,
    },
    proof::Proof,
};
pub fn parse<'a>(
    code: &'a str,
    name: &'a str,
    pkgs: &'a HashMap<String, Package>,
    games: &'a HashMap<String, Composition>,
) -> Proof<'a> {
    let mut proof_pairs = SspParser::parse_proof(code).unwrap();
    handle_proof(
        name,
        code,
        proof_pairs.next().unwrap(),
        pkgs.clone(),
        games.clone(),
    )
    .unwrap_or_else(|err| panic!("handle error: {err}"))
}

pub fn parse_fails(
    code: &str,
    name: &str,
    pkgs: &HashMap<String, Package>,
    games: &HashMap<String, Composition>,
) -> ParseProofError {
    // any test game should adhere to the grammar
    let mut proof_pairs = SspParser::parse_proof(code).unwrap();

    let Err(err) = handle_proof(
        name,
        code,
        proof_pairs.next().unwrap(),
        pkgs.clone(),
        games.clone(),
    ) else {
        panic!("expected an error when parsing {name}, but it succeeded")
    };

    err
}

pub fn read_file(file_name: &'static str) -> String {
    std::fs::read_to_string(format!("{TESTDATA_SSPCODE_PATH}/proofs/{file_name}"))
        .unwrap_or_else(|_| panic!("error reading test code proof {file_name}"))
}

pub fn parse_file_fails(
    file_name: &'static str,
    pkgs: &HashMap<String, Package>,
    games: &HashMap<String, Composition>,
) -> ParseProofError {
    let file = std::fs::File::open(format!("{TESTDATA_SSPCODE_PATH}/proofs/{file_name}"))
        .unwrap_or_else(|_| panic!("error opening test code proof {file_name}"));

    let contents = std::io::read_to_string(file)
        .unwrap_or_else(|_| panic!("error reading test code proof {file_name}"));

    parse_fails(&contents, file_name, pkgs, games)
}
