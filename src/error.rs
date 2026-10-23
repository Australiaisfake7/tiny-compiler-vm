//! Crate-wide error types and the conventions every error in the toolchain follows.
//!
//! # Structure
//!
//! Each stage of the pipeline owns one error type, defined next to the code that produces it:
//!
//! | stage          | type           | location in the source | category label   |
//! |----------------|----------------|------------------------|------------------|
//! | lexer          | `LexError`     | line and column        | `lexical error`  |
//! | parser         | `ParseError`   | line and column        | `syntax error`   |
//! | AST flattener  | `FlattenError` | none (AST has no spans)| `semantic error` |
//! | VM             | `VmError`      | none                   | `runtime error`  |
//!
//! The compiler stages are grouped into [`CompileError`](crate::compiler::CompileError), and
//! everything the command line tool can fail with is grouped into [`Error`]. All conversions
//! are `From` impls, so `?` works at every boundary.
//!
//! # Conventions
//!
//! * Errors that point at source text are `struct { kind, span }`; errors that do not are plain
//!   enums.
//! * `Display` output is `<category> error[ at line L, column C]: <message>`, so every error says
//!   which stage failed.
//! * Messages are lowercase, have no trailing period, and quote names, types, keywords and
//!   expressions with single quotes. Types, tokens and expressions are printed in source syntax
//!   (`'int?'`, `'a + b'`), never with `{:?}`.
//! * An error that wraps a lower-level cause (an I/O or bytecode error) exposes it through
//!   `source()` and does not repeat it in its own message; [`Error::report`] joins the chain.
//! * Faults that can only come from a bug in the toolchain are kept apart from mistakes in the
//!   user's program (`FlattenError::Internal`, `VmError::InvalidBytecode`).

use std::fmt;
use std::path::PathBuf;

use crate::compiler::CompileError;
use crate::vm::VmError;

/// A position in source text. Both fields are 1-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// Everything the command line tool can fail with.
#[derive(Debug)]
pub enum Error {
    /// The file given on the command line does not have the extension the mode requires.
    UnexpectedExtension { expected: &'static str, found: Option<String> },
    /// A file could not be read, created or opened.
    Io { action: &'static str, path: PathBuf, source: std::io::Error },
    /// A bytecode file could not be written or read.
    Bytecode { action: &'static str, path: PathBuf, source: bincode::Error },
    Compile(CompileError),
    Runtime(VmError),
}

impl Error {
    /// The full message: this error followed by each underlying cause, separated by `: `.
    pub fn report(&self) -> String {
        let mut message: String = self.to_string();
        let mut cause: Option<&dyn std::error::Error> = std::error::Error::source(self);

        while let Some(error) = cause {
            message.push_str(": ");
            message.push_str(&error.to_string());
            cause = error.source();
        }

        message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedExtension { expected, found: Some(found) } =>
                write!(f, "usage error: expected a file ending in '.{expected}', found '.{found}'"),
            Error::UnexpectedExtension { expected, found: None } =>
                write!(f, "usage error: expected a file ending in '.{expected}', found a file with no extension"),
            Error::Io { action, path, .. } =>
                write!(f, "file error: cannot {action} '{}'", path.display()),
            Error::Bytecode { action, path, .. } =>
                write!(f, "bytecode error: cannot {action} '{}'", path.display()),
            Error::Compile(error) => error.fmt(f),
            Error::Runtime(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            Error::Bytecode { source, .. } => Some(source.as_ref()),
            // The compile and runtime errors are shown as-is, so they are not a separate cause.
            Error::UnexpectedExtension { .. } | Error::Compile(_) | Error::Runtime(_) => None,
        }
    }
}

impl From<CompileError> for Error {
    fn from(error: CompileError) -> Self {
        Error::Compile(error)
    }
}

impl From<VmError> for Error {
    fn from(error: VmError) -> Self {
        Error::Runtime(error)
    }
}
