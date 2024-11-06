pub mod writer;
use crate::debug_assert_matches;


use pest::Parser;
extern crate pest;

#[derive(Parser)]
#[grammar = "writers/tex/smtmodel.pest"]
pub struct SmtModelParser;

impl SmtModelParser {
    pub fn parse_model(content: &str) -> Vec<(String,String,String)> {
        let mut ast = SmtModelParser::parse(Rule::model, content).unwrap();
        let mut ast = ast.next().unwrap();
        debug_assert_matches!(ast.as_rule(), Rule::model);

        ast.into_inner().map(|line| {
            let mut inner = line.into_inner();
            let name = inner.next().unwrap();
            debug_assert_matches!(name.as_rule(), Rule::name);
            let tipe = inner.next().unwrap();
            //debug_assert_matches!(tipe.as_rule(), Rule::type);
            let value = inner.next().unwrap();
            debug_assert_matches!(value.as_rule(), Rule::value);
            (String::from(name.as_str()), String::from(tipe.as_str()), String::from(value.as_str()))
        }).collect()
    }
}
