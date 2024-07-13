use crate::{
    package::Package,
    parser::{
        package::{handle_pkg, ParsePackageError},
        SspParser,
    },
};

pub fn parse(code: &str, file_name: &str) -> (String, Package) {
    let mut pkg_pairs = SspParser::parse_package(code).unwrap_or_else(|err| {
        panic!(
            "error parsing package {file_name} (in call to pest): {err}",
            file_name = file_name,
            err = err
        )
    });
    handle_pkg(file_name, code, pkg_pairs.next().unwrap()).unwrap()
}

pub fn parse_fails(code: &str, name: &str) -> ParsePackageError {
    // any test game should adhere to the grammar
    let mut pkg_pairs = SspParser::parse_package(code).unwrap_or_else(|err| {
        panic!(
            "error parsing package {name} (in call to pest): {err}",
            name = name,
            err = err
        )
    });

    handle_pkg(name, code, pkg_pairs.next().unwrap()).expect_err(&format!(
        "expected an error when parsing {name}, but it succeeded"
    ))
}

pub const TINY: &str = r#"package TinyPkg {
            params {
              n: Integer,
            }

            oracle N() -> Integer {
              return n;
            }
        }"#;

pub const TINY_BAD_PARAM_TYPE: &str = r#"package TinyPkg {
            params {
              foo: ThisTypeDoesNotExist,
            }
        }"#;

pub const TINY_BAD_STATE_TYPE: &str = r#"package TinyPkg {
            state {
              foo: ThisTypeDoesNotExist,
            }
        }"#;

pub const SMALL_FOR: &str = r#"package SmallForPkg {
       params {
          n: Integer,
       }

       import oracles {
         for i: 1 <= i <= n {
           N[i]() -> Integer,
         }
       }

       oracle Sum() -> Integer {
         sum <- 0;

         for i: 1 <= i <= n {
           n_i <- invoke N[i]();
           sum <- (sum + n_i);
         }

         return sum;
       }
    }"#;

pub const TINY_BAD_1: &str = r#"package TinyBadPkg1 {
            params {
              n: Integer,
            }

            oracle N() -> String {
              return n;
            }
        }"#;

pub const TINY_BAD_2: &str = r#"package TinyBadPkg2 {
            oracle N() -> String {
              return n;
            }
        }"#;

pub const TINY_BAD_3: &str = r#"package TinyBadPkg3 {
            oracle N() -> Integer {
              return (true + false);
            }
        }"#;

pub const TINY_BAD_4: &str = r#"package TinyBadPkg4 {
            oracle N() -> Integer {
              return (true + 3);
            }
        }"#;

pub const TINY_BAD_5: &str = r#"package TinyBadPkg5 {
            oracle N() -> Integer {
              return (3 + true);
            }
        }"#;

pub const TINY_BAD_6: &str = r#"package TinyBadPkg6 {
            oracle N() -> Bool {
              return (3 + 2);
            }
        }"#;
