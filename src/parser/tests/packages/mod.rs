use crate::{
    package::Package,
    parser::{
        package::{handle_pkg, ParsePackageError},
        SspParser,
    },
};

pub fn parse_pkg(code: &str, file_name: &str) -> (String, Package) {
    let mut pkg_pairs = SspParser::parse_package(code).unwrap_or_else(|err| {
        panic!(
            "error parsing package {file_name} (in call to pest): {err}",
            file_name = file_name,
            err = err
        )
    });
    handle_pkg(file_name, code, pkg_pairs.next().unwrap()).unwrap()
}

pub fn parse_pkg_fails(code: &str, name: &str) -> ParsePackageError {
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

pub const TINY_PKG_CODE: &str = r#"package TinyPkg {
            params {
              n: Integer,
            }

            oracle N() -> Integer {
              return n;
            }
        }"#;

pub const TINY_PKG_BAD_PARAM: &str = r#"package TinyPkg {
            params {
              foo: ThisTypeDoesNotExist,
            }
        }"#;
pub const SMALL_FOR_PKG_CODE: &str = r#"package SmallForPkg {
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
