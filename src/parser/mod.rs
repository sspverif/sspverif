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
mod tests;
