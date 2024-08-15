use crate::{
    parser::{
        error::{NoSuchTypeError, TypeMismatchError, UndefinedIdentifierError},
        package::{ParseExpressionError, ParsePackageError},
        tests::{packages, slice_source_span},
    },
    types::Type,
};

#[test]
fn undefined_type_in_pkg_params() {
    let err = packages::parse_file_fails("bad_param_type.ssp");
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
    let err = packages::parse_file_fails("bad_state_type.ssp");
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
fn type_mismatch_in_assignment_to_statevar() {
    let err = packages::parse_file_fails("state_assignment_type_mismatch.ssp");

    match err {
        ParsePackageError::ParseExpression(ParseExpressionError::TypeMismatch(
            TypeMismatchError {
                at,
                expected,
                got,
                source_code,
            },
        )) => {
            assert_eq!(expected, Type::Integer);
            assert_eq!(got, Type::Boolean);
            assert_eq!(slice_source_span(&source_code, &at), "false");
        }
        other => {
            let msg = format!("expected a different error; got {:?}", other);
            let report = miette::Report::new(other);
            panic!("{}, which looks like this:\n{:?}", msg, report)
        }
    };
}

#[test]
fn wrong_return_type_fails() {
    let err = packages::parse_file_fails("tiny_bad1.ssp");

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
    let err = packages::parse_file_fails("tiny_bad2.ssp");

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
    let err = packages::parse_file_fails("tiny_bad3.ssp");

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
    let err = packages::parse_file_fails("tiny_bad4.ssp");

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
    let err = packages::parse_file_fails("tiny_bad5.ssp");

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
    let err = packages::parse_file_fails("tiny_bad6.ssp");

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

#[test]
fn loop_start_non_integer_fails() {
    let err = packages::parse_file_fails("EmptyLoopStartNonIntegerFails.pkg.ssp");

    assert!(
        matches!(
            &err,
            ParsePackageError::ParseExpression(ParseExpressionError::TypeMismatch(
                TypeMismatchError {
                    expected: Type::Integer,
                    got,
                    ..
                }
            )) if got == &Type:: Bits("n".to_string())));
    assert_eq!(
        "type mismatch: got Bits(\"n\"), expected Integer",err.to_string()
    )
}

#[test]
fn loop_end_non_integer_fails() {
    let err = packages::parse_file_fails("EmptyLoopEndNonIntegerFails.pkg.ssp");

    assert!(
        matches!(
            &err,
            ParsePackageError::ParseExpression(ParseExpressionError::TypeMismatch(
                TypeMismatchError {
                    expected: Type::Integer,
                    got,
                    ..
                }
            )) if got == &Type:: Bits("n".to_string())));
    assert_eq!(
        "type mismatch: got Bits(\"n\"), expected Integer",err.to_string()
    )
}
