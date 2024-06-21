pub mod common;
pub mod composition;
pub mod package;

pub mod error;
pub mod proof;

use miette::{NamedSource, SourceSpan};
use pest::Parser;
extern crate pest;

use pest::error::Error;
use pest::iterators::Pairs;

#[derive(Parser)]
#[grammar = "parser/ssp.pest"]
pub struct SspParser;

#[derive(Debug, Clone, Copy)]
pub struct ParseContext<'a> {
    pub file_name: &'a str,
    pub file_content: &'a str,
}

pub trait CommonContext {
    fn file_name(&self) -> &str;
    fn file_contents(&self) -> &str;
    fn scope_enter(&mut self);
    fn scope_leave(&mut self);
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
    use super::ParseContext;

    #[test]
    fn empty_param_section_is_fine() {
        let file_name = "test_file_name.ssp";
        let file_content = r#"package testpkg {
            params {}
        }
        "#;

        let ctx = ParseContext {
            file_name,
            file_content,
        };

        let mut pairs = super::SspParser::parse_package(file_content)
            .expect("empty param section fails parsing");

        super::package::handle_pkg(ctx, pairs.next().unwrap()).unwrap();
    }

    #[test]
    fn empty_state_section_is_fine() {
        let file_name = "test_file_name.ssp";
        let file_content = r#"package testpkg {
            state {}
        }
        "#;

        let ctx = ParseContext {
            file_name,
            file_content,
        };

        let mut pairs = super::SspParser::parse_package(file_content)
            .expect("empty state section fails parsing");
        super::package::handle_pkg(ctx, pairs.next().unwrap()).unwrap();
    }
}
