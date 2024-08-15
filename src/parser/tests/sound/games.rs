use crate::{
    parser::{
        composition::ParseGameError,
        error::TypeMismatchError,
        package::ParseExpressionError,
        tests::{games, packages},
    },
    types::Type,
};
use std::{collections::HashMap, iter::FromIterator as _};

#[test]
fn type_mismatch_in_game_params() {
    let (name, pkg) = packages::parse_file("tiny.ssp");
    let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
    let err = games::parse_file_fails("small_mistyped.ssp", &pkg_map);

    assert!(matches!(
        &err,
        ParseGameError::ParseExpression(ParseExpressionError::TypeMismatch(
            TypeMismatchError {
                at,
                expected: Type::Integer,
                got: Type::Boolean,
                source_code,
            }
        )) if &source_code.inner()[at.offset()..(at.offset()+at.len())] == "n"
    ));

    let report = miette::Report::new(err);
    println!("{report:?}");
}

#[test]
fn missing_game_params_block() {
    let (name, pkg) = packages::parse_file("tiny.ssp");
    let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
    let err = games::parse_file_fails("small_noparams.ssp", &pkg_map);

    // TODO: figure out what error this should be

    let report = miette::Report::new(err);
    println!("{report:?}");
}
#[test]
fn missing_game_empty_block() {
    let (name, pkg) = packages::parse_file("tiny.ssp");
    let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
    let err = games::parse_file_fails("small_emptyparams.ssp", &pkg_map);

    // TODO: figure out what error this should be

    let report = miette::Report::new(err);
    println!("{report:?}");
}

#[test]
fn param_wrong_type() {
    let pkgs = packages::parse_files(&["PRF.pkg.ssp", "KeyReal.pkg.ssp", "Enc.pkg.ssp"]);
    let err = games::parse_file_fails("Game-param-wrong-type-should-fail.comp.ssp", &pkgs);

    let ParseGameError::ParseExpression(ParseExpressionError::TypeMismatch(err)) = &err else {
        panic!("expected different error, got {}", err);
    };

    assert_eq!(err.expected, Type::Integer);
    assert_eq!(err.got, Type::Bits("n".to_string()));
}

#[test]
fn oracle_missing() {
    let pkgs = packages::parse_files(&["PRF.pkg.ssp", "KeyReal.pkg.ssp", "Enc.pkg.ssp"]);
    let err = games::parse_file_fails("Game-missing-edge-should-fail.comp.ssp", &pkgs);

    // TODO: figure out what error this should be, started writing OracleMissingError, but the code doesn't do anything interesting yet. 

    let report = miette::Report::new(err);
    println!("{report:?}");
}

#[test]
fn oracle_imported_twice() {
    let pkgs = packages::parse_files(&["PRF.pkg.ssp", "KeyReal.pkg.ssp", "Enc.pkg.ssp"]);
    let err = games::parse_file_fails("Game-too-many-edges-left-should-fail.comp.ssp", &pkgs);

    // TODO: figure out what error this should be, error type seems to be still missing

    let report = miette::Report::new(err);
    println!("{report:?}");
}

#[test]
fn oracle_imported_but_not_exported() {
    let pkgs = packages::parse_files(&["PRF.pkg.ssp", "KeyReal.pkg.ssp", "Enc.pkg.ssp"]);
    let err = games::parse_file_fails("Game-too-many-edges-right-should-fail.comp.ssp", &pkgs);

    // TODO: figure out what error this should be, error type seems to be still missing

    let report = miette::Report::new(err);
    println!("{report:?}");
}