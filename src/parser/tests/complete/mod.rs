use super::{games, packages::*, proofs};
use crate::{
    expressions::Expression,
    gamehops::equivalence,
    identifier::{
        game_ident::{GameConstIdentifier, GameIdentifier},
        Identifier,
    },
    proof::GameHop,
    types::Type,
    util::prover_process::{Communicator, ProverBackend},
};
use std::{
    collections::HashMap,
    fmt::Display,
    iter::FromIterator as _,
    sync::{Arc, RwLock},
};

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
    let game = games::parse_file("tiny.ssp", &HashMap::default());

    assert_eq!(game.name, "TinyGame");
    assert_eq!(game.consts[0].0, "n");
    assert_eq!(game.consts[0].1, Type::Integer);
    assert_eq!(game.consts.len(), 1);
    assert!(game.pkgs.is_empty());
}

#[test]
fn tiny_package() {
    let (name, pkg) = parse_file("tiny.ssp");

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
    let (name, pkg) = parse_file("tiny.ssp");
    let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
    let game = games::parse_file("small.ssp", &pkg_map);

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
    let (name, pkg) = parse_file("small_for.ssp");

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
    let (name, pkg) = parse_file("tiny.ssp");
    let pkg_map = HashMap::from_iter(vec![(name, pkg.clone())]);
    let game = games::parse_file("small_multi_inst.ssp", &pkg_map);
    println!("{game:#?}");

    assert_eq!(game.pkgs[0].multi_instance_indices.indices.len(), 1);
}

#[test]
fn untyped_none_type_inference_works() {
    let (name, pkg) = parse_file("none_inference_return.ssp");
}

#[test]
#[ignore]
fn equivalence_gamehome_generates_code() {
    let packages = parse_files(&["tiny.ssp"]);
    let games = games::parse_files(&["small.ssp"], &packages);
    let proof = proofs::parse_file("equivalence-small-small.ssp", &packages, &games);

    let eq = proof
        .game_hops
        .iter()
        .find_map(|hop| match hop {
            GameHop::Equivalence(eq) => Some(eq),
            _ => None,
        })
        .unwrap();

    let backend = ProverBackend::Cvc5;
    let transcript = SharedVecWriter::default();
    let prover = Communicator::new_with_transcript(backend, transcript.clone()).unwrap();
    equivalence::verify(eq, &proof, prover).unwrap_or_else(|err| {
        panic!(
            "got error {err}.\n\ntranscript:\n{transcript}",
            err = err,
            transcript = transcript
        )
    })
}

/// This is a helper for transcripts. It can be cloned, and what is written in one clone can be
/// read in all others. It is concurrency-safe. This can be passed into the Communicator, a simple
/// `&mut Vec<u8>` can't. a `Vec<u8>` can, but then we lose access to it. This solves that problem.
#[derive(Clone, Default)]
struct SharedVecWriter(Arc<RwLock<Vec<u8>>>);

impl Display for SharedVecWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vec_guard = self.0.read().unwrap();
        let vec_ref: &Vec<u8> = vec_guard.as_ref();
        let string = String::from_utf8(vec_ref.to_vec()).unwrap();

        write!(f, "{string}")
    }
}

impl std::io::Write for SharedVecWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write().as_mut().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.write().as_mut().unwrap().flush()
    }
}
