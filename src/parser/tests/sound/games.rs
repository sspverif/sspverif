use crate::{
    parser::{
        composition::ParseGameError,
        error::{MissingEdgeForImportedOracleError, TypeMismatchError},
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

    let report = miette::Report::new(err);
    println!("{report:?}");
    todo!("figure out what error this should be");
}
#[test]
fn missing_game_empty_block() {
    let (name, pkg) = packages::parse_file("tiny.ssp");
    let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
    let err = games::parse_file_fails("small_emptyparams.ssp", &pkg_map);

    let report = miette::Report::new(err);
    println!("{report:?}");
    todo!("figure out what error this should be");
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
fn oracle_missing_edge_for_imported_oracle() {
    let pkgs = packages::parse_files(&["PRF.pkg.ssp", "KeyReal.pkg.ssp", "Enc.pkg.ssp"]);
    let err = games::parse_file_fails("Game-missing-edge-should-fail.comp.ssp", &pkgs);

    assert!(
        matches!(
            &err,
            ParseGameError::MissingEdgeForImportedOracle(MissingEdgeForImportedOracleError {
                pkg_inst_name,
                pkg_name,
                oracle_name,
                ..
            }) if pkg_inst_name == "enc" && pkg_name == "Enc" && oracle_name == "Get"
        ),
        "got instead:\n{err:?}",
        //err = err,
        err = miette::Report::new(err)
    );

    let report = miette::Report::new(err);
    println!("{report:?}");
}

#[test]
fn oracle_imported_twice() {
    let pkgs = packages::parse_files(&["PRF.pkg.ssp", "KeyReal.pkg.ssp", "Enc.pkg.ssp"]);
    let err = games::parse_file_fails("Game-too-many-edges-left-should-fail.comp.ssp", &pkgs);

    let report = miette::Report::new(err);
    println!("{report:?}");
    todo!("figure out what error this should be");
}

#[test]
fn oracle_imported_but_not_exported() {
    let pkgs = packages::parse_files(&["PRF.pkg.ssp", "KeyReal.pkg.ssp", "Enc.pkg.ssp"]);
    let err = games::parse_file_fails("Game-too-many-edges-right-should-fail.comp.ssp", &pkgs);

    let report = miette::Report::new(err);
    println!("{report:?}");
    todo!("figure out what error this should be");
}

