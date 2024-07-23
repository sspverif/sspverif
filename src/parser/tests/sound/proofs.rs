use std::{collections::HashMap, iter::FromIterator};

use crate::parser::tests::{games, packages, proofs, slice_source_span};

#[test]
fn fail_reduction_assumption_is_second() {
    let pkgs = packages::parse_files(&[
        "Enc.pkg.ssp",
        "KeyIdeal.pkg.ssp",
        "KeyReal.pkg.ssp",
        "PRF.pkg.ssp",
    ]);

    let games = games::parse_files(
        &[
            "AssumptionIdeal.comp.ssp",
            "AssumptionReal.comp.ssp",
            "ConstructionReal.comp.ssp",
            "ConstructionIdeal.comp.ssp",
        ],
        &pkgs,
    );

    let err = proofs::parse_file_fails("reduction-assumption-2nd-should-fail.ssp", &pkgs, &games);

    let crate::parser::proof::ParseProofError::AssumptionMappingRightGameInstanceIsFromAssumption(
        err,
    ) = err
    else {
        panic!("expected a different error. got error {}", err)
    };

    let at = slice_source_span(&err.source_code, &err.at);
    assert_eq!(at, "assumptionreal");
}

#[test]
fn fail_reduction_construction_is_first() {
    let pkgs = packages::parse_files(&[
        "Enc.pkg.ssp",
        "KeyIdeal.pkg.ssp",
        "KeyReal.pkg.ssp",
        "PRF.pkg.ssp",
    ]);

    let games = games::parse_files(
        &[
            "AssumptionIdeal.comp.ssp",
            "AssumptionReal.comp.ssp",
            "ConstructionReal.comp.ssp",
            "ConstructionIdeal.comp.ssp",
        ],
        &pkgs,
    );

    let err = proofs::parse_file_fails("reduction-construction-1st-should-fail.ssp", &pkgs, &games);

    let crate::parser::proof::ParseProofError::AssumptionMappingLeftGameInstanceIsNotFromAssumption(
        err,
    ) = err
    else {
        panic!("expected a different error. got error {}", err)
    };

    let at = slice_source_span(&err.source_code, &err.at);
    assert_eq!(at, "constructionreal");
}

#[test]
fn fail_reduction_construction_game_inst_not_defined() {
    let pkgs = packages::parse_files(&[
        "Enc.pkg.ssp",
        "KeyIdeal.pkg.ssp",
        "KeyReal.pkg.ssp",
        "PRF.pkg.ssp",
    ]);

    let games = games::parse_files(
        &[
            "AssumptionIdeal.comp.ssp",
            "AssumptionReal.comp.ssp",
            "ConstructionReal.comp.ssp",
            "ConstructionIdeal.comp.ssp",
        ],
        &pkgs,
    );

    let err = proofs::parse_file_fails(
        "reduction-missing-1st-construction-game-instance-should-fail.ssp",
        &pkgs,
        &games,
    );

    let crate::parser::proof::ParseProofError::UndefinedGameInstance(err) = err else {
        panic!("expected a different error. got error {}", err)
    };

    let at = slice_source_span(&err.source_code, &err.at);
    assert_eq!(at, "constructionreal");
}

#[test]
fn fail_reduction_construction_game_inst_not_defined2() {
    let pkgs = packages::parse_files(&[
        "Enc.pkg.ssp",
        "KeyIdeal.pkg.ssp",
        "KeyReal.pkg.ssp",
        "PRF.pkg.ssp",
    ]);

    let games = games::parse_files(
        &[
            "AssumptionIdeal.comp.ssp",
            "AssumptionReal.comp.ssp",
            "ConstructionReal.comp.ssp",
            "ConstructionIdeal.comp.ssp",
        ],
        &pkgs,
    );

    let err = proofs::parse_file_fails(
        "reduction-missing-2nd-construction-game-instance-should-fail.ssp",
        &pkgs,
        &games,
    );

    let crate::parser::proof::ParseProofError::UndefinedGameInstance(err) = err else {
        panic!("expected a different error. got error {}", err)
    };

    let at = slice_source_span(&err.source_code, &err.at);
    assert_eq!(at, "constructionideal");
}

#[test]
fn fail_reduction_assumption_not_defined() {
    let pkgs = packages::parse_files(&[
        "Enc.pkg.ssp",
        "KeyIdeal.pkg.ssp",
        "KeyReal.pkg.ssp",
        "PRF.pkg.ssp",
    ]);

    let games = games::parse_files(
        &[
            "AssumptionIdeal.comp.ssp",
            "AssumptionReal.comp.ssp",
            "ConstructionReal.comp.ssp",
            "ConstructionIdeal.comp.ssp",
        ],
        &pkgs,
    );

    let err = proofs::parse_file_fails(
        "reduction-missing-assumption-should-fail.ssp",
        &pkgs,
        &games,
    );

    let crate::parser::proof::ParseProofError::UndefinedAssumption(err) = err else {
        panic!("expected a different error. got error {}", err)
    };

    let at = slice_source_span(&err.source_code, &err.at);
    assert_eq!(at, "Assumption");
}

#[test]
fn fail_reduction_assumption_exposes_less() {
    let pkgs = packages::parse_files(&[
        "Enc.pkg.ssp",
        "KeyIdeal.pkg.ssp",
        "KeyReal.pkg.ssp",
        "PRF.pkg.ssp",
    ]);

    let games = games::parse_files(
        &[
            "AssumptionIdeal.comp.ssp",
            "AssumptionIdealWeak.comp.ssp",
            "AssumptionReal.comp.ssp",
            "AssumptionRealWeak.comp.ssp",
            "ConstructionReal.comp.ssp",
            "ConstructionIdeal.comp.ssp",
        ],
        &pkgs,
    );

    todo!("we don't have the correct error type yet, and we don't even catch this error");

    let _err = proofs::parse_file_fails(
        "reduction-assumption-exposes-less-should-fail.ssp",
        &pkgs,
        &games,
    );
}

#[test]
fn fail_reduction_inconsistent_wiring_less() {
    let pkgs = packages::parse_files(&[
        "Enc.pkg.ssp",
        "KeyIdeal.pkg.ssp",
        "KeyReal.pkg.ssp",
        "PRF.pkg.ssp",
    ]);

    let games = games::parse_files(
        &[
            "AssumptionIdeal.comp.ssp",
            "AssumptionIdealWeak.comp.ssp",
            "AssumptionReal.comp.ssp",
            "AssumptionRealWeak.comp.ssp",
            "ConstructionReal.comp.ssp",
            "ConstructionReal-badwiring.comp.ssp",
            "ConstructionIdeal.comp.ssp",
        ],
        &pkgs,
    );

    todo!("we don't have the correct error type yet, and we don't even catch this error");

    let err = proofs::parse_file_fails(
        "reduction-inconsistent-wiring-should-fail.ssp",
        &pkgs,
        &games,
    );
}
