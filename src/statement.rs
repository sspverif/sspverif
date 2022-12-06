use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::types::Type;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeBlock(pub Vec<Statement>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Abort,
    Return(Option<Expression>),
    Assign(Identifier, Option<Expression>, Expression),
    Parse(Vec<Identifier>, Expression),
    IfThenElse(Expression, CodeBlock, CodeBlock),
    Sample(Identifier, Option<Expression>, Option<usize>, Type),
    InvokeOracle {
        id: Identifier,
        opt_idx: Option<Expression>,
        name: String,
        args: Vec<Expression>,
        target_inst_name: Option<String>,
        tipe: Option<Type>,
    },
}

#[macro_export]
macro_rules! block {
    ( $( $s:expr ),* ) => {
        {
            CodeBlock(vec![ $( $s.clone(), )* ])
        }
    }
}
