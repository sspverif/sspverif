use std::collections::HashMap;

use crate::{
    package::{Composition, Package},
    parser::{
        composition::{handle_composition, ParseGameError},
        Rule, SspParser,
    },
};

pub fn parse_game(code: &str, name: &str, pkg_map: &HashMap<String, Package>) -> Composition {
    let mut game_pairs = SspParser::parse_composition(code).unwrap();
    handle_composition(name, code, game_pairs.next().unwrap(), pkg_map)
        .unwrap_or_else(|err| panic!("handle error: {err}", err = err))
}

pub fn parse_game_fails(
    code: &str,
    name: &str,
    pkg_map: &HashMap<String, Package>,
) -> ParseGameError {
    // any test game should adhere to the grammar
    let mut game_pairs = SspParser::parse_composition(code).unwrap();

    handle_composition(name, code, game_pairs.next().unwrap(), pkg_map).expect_err(&format!(
        "expected an error when parsing {name}, but it succeeded"
    ))
}

pub const TINY_GAME_CODE: &str = r#"composition TinyGame {
    const n: Integer;
}
"#;

pub const SMALL_GAME_CODE: &str = r#"composition SmallGame {
        const n: Integer;

        instance tiny_instance  = TinyPkg {
            params {
                n: n,
            }
        }
    }"#;

pub const SMALL_MISTYPED_GAME_CODE: &str = r#"composition SmallMistypedGame {
        const n: Bool;

        instance tiny_instance  = TinyPkg {
            params {
                n: n,
            }
        }
    }"#;

pub const SMALL_MULTI_INST_GAME_CODE: &str = r#"composition SmallMultiInstGame {
        for i: 0 <= i < 200 {
            instance tiny_instance[i] = TinyPkg {
                params {
                    n:  i
                }
            }
        }
    }"#;
