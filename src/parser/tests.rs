// These contain testdata
mod games;
mod packages;
mod proofs;

// these contain tests:

/// The complete module contains tests whether things that are supposed to parse actually parse
/// into what is expected.
mod complete;

/// The sound module contains tests whether things that are not supposed to parse actually produce
/// errors - and the right ones.
mod sound;

#[test]
fn empty_param_section_is_fine() {
    let file_name = "test_file_name.ssp";
    let file_content = r#"package testpkg {
            params {}
        }
        "#;

    let mut pairs =
        super::SspParser::parse_package(file_content).expect("empty param section fails parsing");

    super::package::handle_pkg(file_name, file_content, pairs.next().unwrap()).unwrap();
}

#[test]
fn empty_state_section_is_fine() {
    let file_name = "test_file_name.ssp";
    let file_content = r#"package testpkg {
            state {}
        }
        "#;

    let mut pairs =
        super::SspParser::parse_package(file_content).expect("empty state section fails parsing");
    super::package::handle_pkg(file_name, file_content, pairs.next().unwrap()).unwrap();
}
