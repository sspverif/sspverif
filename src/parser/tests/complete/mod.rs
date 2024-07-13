use super::{games, packages::*};
use crate::{
    expressions::Expression,
    identifier::{
        game_ident::{GameConstIdentifier, GameIdentifier},
        Identifier,
    },
    types::Type,
};
use std::{collections::HashMap, iter::FromIterator as _};

#[test]
fn empty_param_section_is_fine() {
    let file_name = "test_file_name.ssp";
    let file_content = r#"package testpkg {
            params {}
        }
        "#;

    parse(file_content, file_name);
}

#[test]
fn empty_state_section_is_fine() {
    let file_name = "test_file_name.ssp";
    let file_content = r#"package testpkg {
            state {}
        }
        "#;

    parse(file_content, file_name);
}

#[test]
fn tiny_game_without_packages() {
    let game = games::parse(games::TINY, "tiny-game", &HashMap::default());

    assert_eq!(game.name, "TinyGame");
    assert_eq!(game.consts[0].0, "n");
    assert_eq!(game.consts[0].1, Type::Integer);
    assert_eq!(game.consts.len(), 1);
    assert!(game.pkgs.is_empty());
}

#[test]
fn tiny_package() {
    let (name, pkg) = parse(TINY, "tiny.ssp");

    assert_eq!(name, "TinyPkg");
    assert_eq!(pkg.params.len(), 1);
    assert_eq!(pkg.params[0].0, "n");
    assert_eq!(pkg.params[0].1, Type::Integer);
    assert_eq!(pkg.oracles.len(), 1);
    assert_eq!(pkg.oracles[0].sig.name, "N");
    assert_eq!(pkg.oracles[0].sig.tipe, Type::Integer);
    assert!(pkg.oracles[0].sig.args.is_empty());
    assert!(pkg.imports.is_empty());
}

#[test]
fn small_game() {
    let (name, pkg) = parse(TINY, "tiny.ssp");
    let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
    let game = games::parse(games::SMALL, "small-game", &pkg_map);

    assert_eq!(game.name, "SmallGame");
    assert_eq!(game.consts.len(), 1);
    assert_eq!(game.consts[0].0, "n");
    assert_eq!(game.consts[0].1, Type::Integer);
    assert_eq!(game.pkgs.len(), 1);
    assert_eq!(game.pkgs[0].name, "tiny_instance");
    assert_eq!(game.pkgs[0].params.len(), 1);
    assert_eq!(game.pkgs[0].params[0].0.ident_ref(), "n");
    assert_eq!(
        game.pkgs[0].params[0].1,
        Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::Const(
            GameConstIdentifier {
                name: "n".to_string(),
                tipe: Type::Integer,
                game_name: "SmallGame".to_string(),
                inst_info: None,
            }
        )))
    );
}

#[test]
fn small_for_package() {
    let (name, pkg) = parse(SMALL_FOR, "small-for-pkg");

    assert_eq!(name, "SmallForPkg");
    assert_eq!(pkg.params.len(), 1);
    assert_eq!(pkg.params[0].0, "n");
    assert_eq!(pkg.params[0].1, Type::Integer);
    assert_eq!(pkg.oracles.len(), 1);
    assert_eq!(pkg.oracles[0].sig.name, "Sum");
    assert_eq!(pkg.oracles[0].sig.tipe, Type::Integer);
    assert!(pkg.oracles[0].sig.args.is_empty());
}

#[test]
fn small_multi_inst_game() {
    let (name, pkg) = parse(TINY, "tiny.ssp");
    let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
    let game = games::parse(games::SMALL_MULTI_INST, "small-multi-inst-game", &pkg_map);
    println!("{game:#?}");

    assert_eq!(game.pkgs[0].multi_instance_indices.indices.len(), 1);
}
