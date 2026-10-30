use std::fmt;

use parser::Statement;
use lexer::Token;
pub use ast_flattener::{CompiledData, FlattenError, OpCode};
pub use lexer::{LexError, DataType};
pub use parser::{LiteralType, ParseError};

mod lexer;
mod parser;
pub mod ast_flattener;

#[derive(Debug)]
pub enum CompileError {
    Lex(LexError),
    Parse(ParseError),
    Flatten(FlattenError),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Lex(error) => error.fmt(f),
            CompileError::Parse(error) => error.fmt(f),
            CompileError::Flatten(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<LexError> for CompileError {
    fn from(error: LexError) -> Self {
        CompileError::Lex(error)
    }
}

impl From<ParseError> for CompileError {
    fn from(error: ParseError) -> Self {
        CompileError::Parse(error)
    }
}

impl From<FlattenError> for CompileError {
    fn from(error: FlattenError) -> Self {
        CompileError::Flatten(error)
    }
}

pub fn compile(source: &str) -> Result<CompiledData, CompileError> {
    let (tokens, spans): (Vec<Token>, Vec<crate::error::Span>) = lexer::lex_chars(source.chars())?;
    let statements: Vec<Statement> = parser::parse_tokens(tokens, spans)?;
    let compiled_data: CompiledData = ast_flattener::flatten_ast(&statements)?;
    Ok(compiled_data)
}