use crate::{
    parser::{
        composition::{handle_composition, ParseGameError},
        error::{NoSuchTypeError, TypeMismatchError, UndefinedIdentifierError},
        package::{ParseExpressionError, ParsePackageError},
        tests::{games, packages, slice_source_span},
        SspParser,
    },
    types::Type,
};
use std::{collections::HashMap, iter::FromIterator as _};

#[test]
fn undefined_type_in_pkg_params() {
    let err = packages::parse_fails(packages::TINY_BAD_PARAM_TYPE, "tiny-pkg-bad-param.ssp");
    assert!(
        matches!(err, ParsePackageError::NoSuchType(NoSuchTypeError {
                type_name,
                ..
            })
            if &type_name == "ThisTypeDoesNotExist"
        )
    )
}

#[test]
fn undefined_type_in_pkg_state() {
    let err = packages::parse_fails(packages::TINY_BAD_STATE_TYPE, "tiny-pkg-bad-state.ssp");
    assert!(
        matches!(err, ParsePackageError::NoSuchType(NoSuchTypeError {
                type_name,
                ..
            })
            if &type_name == "ThisTypeDoesNotExist"
        )
    )
}

#[test]
fn type_mismatch_in_game_params() {
    let (name, pkg) = packages::parse(packages::TINY, "tiny-pkg");
    let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
    let mut game_pairs = SspParser::parse_composition(games::SMALL_MISTYPED).unwrap();
    let err = handle_composition(
        "small-mistyped-game.ssp",
        games::SMALL_MISTYPED,
        game_pairs.next().unwrap(),
        &pkg_map,
    )
    .expect_err("expecting an error");
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
fn wrong_return_type_fails() {
    let err = packages::parse_fails(packages::TINY_BAD_1, "tiny-bad-pkg-1");

    assert!(
        matches!(
            err,
            ParsePackageError::ParseExpression(ParseExpressionError::TypeMismatch(
                TypeMismatchError {
                    expected: Type::String,
                    got: Type::Integer,
                    ..
                }
            ))
        ),
        "expected a different error, got {:#?}",
        err
    )
}

#[test]
fn missing_identifier_fails() {
    let err = packages::parse_fails(packages::TINY_BAD_2, "tiny-bad-pkg-2");

    assert!(matches!(
        err,
        ParsePackageError::ParseExpression(ParseExpressionError::UndefinedIdentifier(
                UndefinedIdentifierError { ref ident_name, .. }
        )) if ident_name.as_str() == "n"

    ));

    println!("{:?}", miette::Report::new(err));
}

#[test]
fn bad_add_fails_1() {
    let err = packages::parse_fails(packages::TINY_BAD_3, "tiny-bad-pkg-3");

    assert!(
        matches!(
            err,
            ParsePackageError::ParseExpression(ParseExpressionError::TypeMismatch(
                    TypeMismatchError {
                        expected: Type::Integer,
                        got: Type::Boolean,
                        ref at,
                        ref source_code,
                    }
                )) if slice_source_span(source_code, at) == "true"
        ),
        "got: {:#?}",
        err
    );

    println!("{:?}", miette::Report::new(err));
}

#[test]
fn bad_add_fails_2() {
    let err = packages::parse_fails(packages::TINY_BAD_4, "tiny-bad-pkg-4");

    assert!(
        matches!(
            err,
            ParsePackageError::ParseExpression(ParseExpressionError::TypeMismatch(
                    TypeMismatchError {
                        expected: Type::Integer,
                        got: Type::Boolean,
                        ref at,
                        ref source_code,
                    }
            )) if slice_source_span(source_code, at) == "true"
        ),
        "got: {:#?}",
        err
    );

    println!("{:?}", miette::Report::new(err));
}

#[test]
fn bad_add_fails_3() {
    let err = packages::parse_fails(packages::TINY_BAD_5, "tiny-bad-pkg-5");

    assert!(
        matches!(
            err,
            ParsePackageError::ParseExpression(ParseExpressionError::TypeMismatch(
                    TypeMismatchError {
                        expected: Type::Integer,
                        got: Type::Boolean,
                        ref at,
                        ref source_code,
                    }
            )) if slice_source_span(source_code, at) == "true"
        ),
        "got: {:#?}",
        err
    );

    println!("{:?}", miette::Report::new(err));
}

#[test]
fn bad_add_fails_4() {
    let err = packages::parse_fails(packages::TINY_BAD_6, "tiny-bad-pkg-6");

    assert!(
        matches!(
            err,
            ParsePackageError::ParseExpression(ParseExpressionError::TypeMismatch(
                    TypeMismatchError {
                        expected: Type::Boolean,
                        got: Type::Integer,
                        ref at,
                        ref source_code,
                    },
            )) if slice_source_span(source_code, at) == "(3 + 2)"
        ),
        "got: {:#?}",
        err
    );

    println!("{:?}", miette::Report::new(err));
}
