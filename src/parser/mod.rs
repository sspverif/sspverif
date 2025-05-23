#![allow(clippy::result_large_err)]

pub mod common;
pub mod composition;
pub mod package;

pub mod error;
pub mod proof;
// mod wire_checks;

pub mod ast;
pub mod reduction;

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
    pub types: Vec<String>,
}

impl<'a> ParseContext<'a> {
    pub fn new(file_name: &'a str, file_content: &'a str) -> Self {
        let mut scope = Scope::new();
        scope.enter();
        let types = vec![];

        Self {
            file_name,
            file_content,
            scope,
            types,
        }
    }

    pub fn named_source(&self) -> miette::NamedSource<String> {
        miette::NamedSource::new(self.file_name, self.file_content.to_string())
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
pub(crate) mod tests;
