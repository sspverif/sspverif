use super::{games::*, packages::*};
use crate::{
    parser::{
        composition::{handle_composition, ParseGameError},
        error::TypeMismatchError,
        SspParser,
    },
    types::Type,
};
use std::{collections::HashMap, iter::FromIterator as _};

#[test]
fn type_mismatch_in_params() {
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
