pub mod common;
pub mod composition;
pub mod package;

pub mod error;
pub mod proof;

use pest::Parser;
extern crate pest;

use pest::error::Error;
use pest::iterators::Pairs;

use crate::util::scope::Scope;

#[derive(Parser)]
#[grammar = "parser/ssp.pest"]
pub struct SspParser;

#[derive(Debug, Clone)]
pub struct ParseContext<'a> {
    pub file_name: &'a str,
    pub file_content: &'a str,
    pub scope: Scope,
}

impl<'a> ParseContext<'a> {
    pub fn new(file_name: &'a str, file_content: &'a str) -> Self {
        let mut scope = Scope::new();
        scope.enter();

        Self {
            file_name,
            file_content,
            scope,
        }
    }
}

impl SspParser {
    pub fn parse_package(contents: &str) -> Result<Pairs<Rule>, Error<Rule>> {
        SspParser::parse(Rule::package, contents)
    }

    pub fn parse_composition(contents: &str) -> Result<Pairs<Rule>, Error<Rule>> {
        SspParser::parse(Rule::composition, contents)
    }

    pub fn parse_proof(contents: &str) -> Result<Pairs<Rule>, Error<Rule>> {
        SspParser::parse(Rule::proof, contents)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn empty_param_section_is_fine() {
        let file_name = "test_file_name.ssp";
        let file_content = r#"package testpkg {
            params {}
        }
        "#;

        let mut pairs = super::SspParser::parse_package(file_content)
            .expect("empty param section fails parsing");

        super::package::handle_pkg(file_name, file_content, pairs.next().unwrap()).unwrap();
    }

    #[test]
    fn empty_state_section_is_fine() {
        let file_name = "test_file_name.ssp";
        let file_content = r#"package testpkg {
            state {}
        }
        "#;

        let mut pairs = super::SspParser::parse_package(file_content)
            .expect("empty state section fails parsing");
        super::package::handle_pkg(file_name, file_content, pairs.next().unwrap()).unwrap();
    }
}
