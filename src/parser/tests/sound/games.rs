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
