use super::{games::*, packages::*, slice_source_span};
use crate::{
    parser::{
        composition::{handle_composition, ParseGameError},
        error::{NoSuchTypeError, TypeMismatchError, UndefinedIdentifierError},
        package::{ParseExpressionError, ParsePackageError},
        SspParser,
    },
    types::Type,
};
use std::{collections::HashMap, iter::FromIterator as _};

#[test]
fn undefined_type_in_pkg_params() {
    let err = parse_pkg_fails(TINY_PKG_BAD_PARAM, "tiny-pkg-bad-param.ssp");
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
    let (name, pkg) = parse_pkg(TINY_PKG_CODE, "tiny-pkg");
    let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
    let mut game_pairs = SspParser::parse_composition(SMALL_MISTYPED_GAME_CODE).unwrap();
    let err = handle_composition(
        "small-mistyped-game.ssp",
        SMALL_MISTYPED_GAME_CODE,
        game_pairs.next().unwrap(),
        &pkg_map,
    )
    .expect_err("expecting an error");
    assert!(matches!(
        &err,
        ParseGameError::ParseExpression(super::super::package::ParseExpressionError::TypeMismatch(
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
    let err = parse_pkg_fails(TINY_BAD_PKG_1_CODE, "tiny-bad-pkg-1");

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
    let err = parse_pkg_fails(TINY_BAD_PKG_2_CODE, "tiny-bad-pkg-2");

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
    let err = parse_pkg_fails(TINY_BAD_PKG_3_CODE, "tiny-bad-pkg-3");

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
    let err = parse_pkg_fails(TINY_BAD_PKG_4_CODE, "tiny-bad-pkg-4");

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
    let err = parse_pkg_fails(TINY_BAD_PKG_5_CODE, "tiny-bad-pkg-5");

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
    let err = parse_pkg_fails(TINY_BAD_PKG_6_CODE, "tiny-bad-pkg-6");

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
