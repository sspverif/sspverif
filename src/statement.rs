use pest::Span;

use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::types::Type;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeBlock(pub Vec<Statement>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Abort(FilePosition),
    Return(Option<Expression>, FilePosition),
    Assign(Identifier, Option<Expression>, Expression, FilePosition),
    Parse(Vec<Identifier>, Expression, FilePosition),
    IfThenElse(Expression, CodeBlock, CodeBlock, FilePosition),
    Sample(
        Identifier,
        Option<Expression>,
        Option<usize>,
        Type,
        FilePosition,
    ),
    InvokeOracle {
        id: Identifier,
        opt_idx: Option<Expression>,
        opt_dst_inst_idx: Option<Expression>,
        name: String,
        args: Vec<Expression>,
        target_inst_name: Option<String>,
        tipe: Option<Type>,
        file_pos: FilePosition,
    },
    For(Identifier, Expression, Expression, CodeBlock, FilePosition),
}

impl Statement {
    pub fn file_pos(&self) -> &FilePosition {
        match self {
            Statement::Abort(file_pos)
            | Statement::Return(_, file_pos)
            | Statement::Assign(_, _, _, file_pos)
            | Statement::Parse(_, _, file_pos)
            | Statement::IfThenElse(_, _, _, file_pos)
            | Statement::Sample(_, _, _, _, file_pos)
            | Statement::InvokeOracle { file_pos, .. }
            | Statement::For(_, _, _, _, file_pos) => file_pos,
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub struct FilePosition {
    file_name: String,
    start_line_col: (usize, usize),
    end_line_col: (usize, usize),
}

impl FilePosition {
    pub fn new(file_name: String, line_start: usize, line_end: usize) -> FilePosition {
        FilePosition {
            file_name,
            start_line_col: (line_start, 0),
            end_line_col: (line_end, 0),
        }
    }

    pub fn from_span(file_name: impl ToString, span: Span) -> FilePosition {
        FilePosition {
            file_name: file_name.to_string(),
            start_line_col: span.start_pos().line_col(),
            end_line_col: span.end_pos().line_col(),
        }
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn start(&self) -> (usize, usize) {
        self.start_line_col
    }

    pub fn end(&self) -> (usize, usize) {
        self.end_line_col
    }

    pub fn line_start(&self) -> usize {
        self.start_line_col.0
    }

    pub fn line_end(&self) -> usize {
        self.end_line_col.0
    }
}

impl std::fmt::Display for FilePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            file_name,
            start_line_col: (line_start, col_start),
            end_line_col: (line_end, col_end),
        } = self;

        write!(
            f,
            "{file_name}:{line_start}:{col_start}..{line_end}:{col_end}"
        )
    }
}

#[macro_export]
macro_rules! block {
    ( $( $s:expr ),* ) => {
        {
            CodeBlock(vec![ $( $s.clone(), )* ])
        }
    }
}
