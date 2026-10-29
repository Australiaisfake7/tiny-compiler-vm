use std::fmt;

use parser::Statement;
use lexer::Token;
pub use ast_flattener::{CompiledData, FlattenError, OpCode};
pub use lexer::{LexError, DataType};
pub use parser::{LiteralType, ParseError};

mod lexer;
mod parser;
pub mod ast_flattener;

/// A failure in any stage of compilation.
#[derive(Debug)]
pub enum CompileError {
    Lex(LexError),
    Parse(ParseError),
    Flatten(FlattenError),
}

// Shown as-is: each stage's own message already names the stage that failed.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Error, Span};
    use crate::vm::{self, VmError};

    fn compile_error(source: &str) -> CompileError {
        compile(source).err().expect("source should not compile")
    }

    fn runtime_error(source: &str) -> VmError {
        vm::execute(compile(source).expect("source should compile")).err().expect("program should fail")
    }

    #[test]
    fn lex_errors_point_at_the_offending_character() {
        match compile_error("int x = 1;\nint y = 2 @ 3;") {
            CompileError::Lex(error) => assert_eq!(error.span, Span { line: 2, column: 11 }),
            other => panic!("expected a lex error, got {other}"),
        }
    }

    #[test]
    fn unterminated_strings_point_at_the_opening_quote() {
        match compile_error("print 1;\nstring s = \"abc;\n") {
            CompileError::Lex(error) => assert_eq!(error.span, Span { line: 2, column: 12 }),
            other => panic!("expected a lex error, got {other}"),
        }
    }

    #[test]
    fn parse_errors_carry_the_span_of_the_offending_token() {
        match compile_error("int x = 5\nprint x;") {
            CompileError::Parse(error) => assert_eq!(error.span, Span { line: 2, column: 1 }),
            other => panic!("expected a parse error, got {other}"),
        }
    }

    #[test]
    fn running_out_of_tokens_is_a_normal_parse_error() {
        // The parser used to have a separate "read past the end" error for this.
        let error = compile_error("while (true) {\n");
        assert_eq!(error.to_string(), "syntax error at line 2, column 1: expected '}', found end of input");
    }

    #[test]
    fn every_stage_names_itself_and_none_uses_debug_formatting() {
        let cases: [(&str, &str); 4] = [
            ("print @;", "lexical error at line 1, column 7: invalid character '@'"),
            ("1 + 2 = 3;", "syntax error at line 1, column 7: '1 + 2' is not a valid assignment target"),
            ("int x = \"s\";", "semantic error: 'x' is declared as 'int', but assigned a value of type 'string'"),
            ("print 1 + true;", "semantic error: cannot apply '+' to operands of type 'int' and 'bool'"),
        ];
        for (source, expected) in cases {
            assert_eq!(compile_error(source).to_string(), expected);
        }
    }

    #[test]
    fn runtime_errors_are_labelled_and_typed() {
        assert_eq!(runtime_error("int z = 0;\nprint 1 / z;").to_string(), "runtime error: division by zero");
        assert!(matches!(
            runtime_error("int[3] a = [1; 2; 3];\nint i = 5;\nprint a[i];"),
            VmError::ArrayIndexOutOfBounds { index: 5, len: 3 }
        ));
    }

    #[test]
    fn compile_and_runtime_errors_convert_into_the_top_level_error() {
        let error: Error = compile_error("print y;").into();
        assert_eq!(error.report(), "semantic error: undeclared variable 'y'");

        let error: Error = VmError::DivisionByZero.into();
        assert_eq!(error.report(), "runtime error: division by zero");
    }

    #[test]
    fn io_errors_expose_their_cause_instead_of_repeating_it() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let error = Error::Io { action: "read", path: "a.tl".into(), source: cause };

        assert_eq!(error.to_string(), "file error: cannot read 'a.tl'");
        assert_eq!(error.report(), "file error: cannot read 'a.tl': no such file");
    }
}
