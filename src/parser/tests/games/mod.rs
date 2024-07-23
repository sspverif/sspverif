use std::collections::HashMap;

use crate::{
    package::{Composition, Package},
    parser::{
        composition::{handle_composition, ParseGameError},
        SspParser,
    },
};

pub fn parse(code: &str, name: &str, pkg_map: &HashMap<String, Package>) -> Composition {
    let mut game_pairs = SspParser::parse_composition(code).unwrap();
    handle_composition(name, code, game_pairs.next().unwrap(), pkg_map)
        .unwrap_or_else(|err| panic!("handle error: {err}", err = err))
}

pub fn parse_fails(code: &str, name: &str, pkg_map: &HashMap<String, Package>) -> ParseGameError {
    // any test game should adhere to the grammar
    let mut game_pairs = SspParser::parse_composition(code).unwrap();

    handle_composition(name, code, game_pairs.next().unwrap(), pkg_map).expect_err(&format!(
        "expected an error when parsing {name}, but it succeeded"
    ))
}

pub fn parse_file(file_name: &'static str, pkg_map: &HashMap<String, Package>) -> Composition {
    let file = std::fs::File::open(format!("src/parser/tests/games/{file_name}"))
        .unwrap_or_else(|_| panic!("error opening test code game {}", file_name));

    let contents = std::io::read_to_string(file)
        .unwrap_or_else(|_| panic!("error reading test code game {}", file_name));

    parse(&contents, file_name, pkg_map)
}

pub fn parse_file_fails(
    file_name: &'static str,
    pkg_map: &HashMap<String, Package>,
) -> ParseGameError {
    let file = std::fs::File::open(format!("src/parser/tests/games/{file_name}"))
        .unwrap_or_else(|_| panic!("error opening test code game {}", file_name));

    let contents = std::io::read_to_string(file)
        .unwrap_or_else(|_| panic!("error reading test code game {}", file_name));

    parse_fails(&contents, file_name, pkg_map)
}

pub const SMALL_MULTI_INST: &str = r#"composition SmallMultiInstGame {
        for i: 0 <= i < 200 {
            instance tiny_instance[i] = TinyPkg {
                params {
                    n:  i
                }
            }
        }
    }"#;
