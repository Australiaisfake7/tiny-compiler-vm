use std::fmt;
use std::str::Chars;

use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBracket, RightBracket, LeftBrace, RightBrace, LeftSquare, RightSquare,
    Dot, Minus, Plus, Slash, Asterisk, Semicolon, Colon, Question, QuestionQuestion, QuestionDot,

    Identifier(String), String(String), Int(i64), Float(f64), Bool(bool),

    Assign, Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    LAnd, LOr, LNot,

    Print, Fun, DataType(DataType), Static, Null, Void, Override,

    If, Else, For, While, Class, Return, Continue, Break,

    EOF,

    Assert,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int, Float, String, Bool, Nullable(Box<DataType>), Null, Void,
    Instance(String), Array(Box<DataType>, usize),
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Int => write!(f, "int"),
            DataType::Float => write!(f, "float"),
            DataType::String => write!(f, "string"),
            DataType::Bool => write!(f, "bool"),
            DataType::Null => write!(f, "null"),
            DataType::Void => write!(f, "void"),
            DataType::Nullable(inner) => write!(f, "{inner}?"),
            DataType::Instance(name) => write!(f, "{name}"),
            DataType::Array(inner, len) => write!(f, "{inner}[{len}]"),
        }
    }
}

/// Describes a token the way it should appear in an error message ("found ...").
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol: &str = match self {
            Token::LeftBracket => "(", Token::RightBracket => ")",
            Token::LeftBrace => "{", Token::RightBrace => "}",
            Token::LeftSquare => "[", Token::RightSquare => "]",
            Token::Dot => ".", Token::Minus => "-", Token::Plus => "+", Token::Slash => "/",
            Token::Asterisk => "*", Token::Semicolon => ";", Token::Colon => ":",
            Token::Question => "?", Token::QuestionQuestion => "??", Token::QuestionDot => "?.",
            Token::Assign => "=", Token::Equal => "==", Token::NotEqual => "!=",
            Token::Less => "<", Token::Greater => ">", Token::LessEqual => "<=", Token::GreaterEqual => ">=",
            Token::LAnd => "&&", Token::LOr => "||", Token::LNot => "!", Token::Assert => "!!",
            Token::Print => "print", Token::Fun => "fun", Token::Static => "static",
            Token::Null => "null", Token::Void => "void", Token::Override => "override",
            Token::If => "if", Token::Else => "else", Token::For => "for", Token::While => "while",
            Token::Class => "class", Token::Return => "return", Token::Continue => "continue",
            Token::Break => "break",
            Token::Bool(true) => "true", Token::Bool(false) => "false",
            Token::DataType(data_type) => return write!(f, "'{data_type}'"),
            Token::Identifier(name) => return write!(f, "identifier '{name}'"),
            Token::String(value) => return write!(f, "string \"{value}\""),
            Token::Int(value) => return write!(f, "integer {value}"),
            Token::Float(value) => return write!(f, "float {value:?}"),
            Token::EOF => return write!(f, "end of input"),
        };

        write!(f, "'{symbol}'")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexErrorKind {
    InvalidCharacter(char),
    /// A two character operator (`&&`, `||`) that was cut short.
    IncompleteOperator { expected: &'static str },
    UnterminatedString,
    IntegerOutOfRange(String),
    InvalidNumber(String),
}

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexErrorKind::InvalidCharacter(c) => write!(f, "invalid character '{c}'"),
            LexErrorKind::IncompleteOperator { expected } =>
                write!(f, "expected '{expected}', found a single '{}'", &expected[..1]),
            LexErrorKind::UnterminatedString => write!(f, "unterminated string literal"),
            LexErrorKind::IntegerOutOfRange(text) =>
                write!(f, "integer literal '{text}' does not fit in a 64-bit integer"),
            LexErrorKind::InvalidNumber(text) => write!(f, "invalid number literal '{text}'"),
        }
    }
}

#[derive(Debug)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Span,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lexical error at {}: {}", self.span, self.kind)
    }
}

impl std::error::Error for LexError {}

struct Scanner<'a> {
    source: Chars<'a>,
    line: usize,
    column: usize,
    /// Where the token most recently returned by `read_next_token` starts.
    token_start: Span,
}

impl<'a> Scanner<'a> {
    fn from_source(source: Chars<'a>) -> Self {
        return Self {
            source: source,
            line: 1,
            column: 0,
            token_start: Span { line: 1, column: 0 },
        };
    }
    /// The position of the character most recently consumed. Columns are 1-based; the newline
    /// that ends a line, and the end of input after one, are reported at column 1 of their line.
    fn here(&self) -> Span {
        Span { line: self.line, column: self.column.max(1) }
    }
    fn error_here(&self, kind: LexErrorKind) -> LexError {
        LexError { kind, span: self.here() }
    }
    fn error_at_token_start(&self, kind: LexErrorKind) -> LexError {
        LexError { kind, span: self.token_start }
    }
    fn advance(&mut self) -> Option<char> {
        match self.peek() {
            Some('\n') => {
                self.line += 1;
                self.column = 0;
            },
            Some(_) => { self.column += 1; },
            None => (),
        }
        return self.source.next();
    }
    fn peek(&self) -> Option<char> {
        return self.source.clone().next();
    }
    fn peek_ahead(&self, i: usize) -> Option<char> {
        self.source.clone().nth(i)
    }
    fn match_advance(&mut self, c: char) -> bool {
        if Some(c) == self.peek() {
            self.advance();
            return true;
        }
        else {
            return false;
        }
    }
    fn read_next_token(&mut self) -> Result<Token, LexError> {
        let first: Option<char> = self.advance();
        self.token_start = self.here();

        return match first {
            Some('{') => Ok(Token::LeftBrace),
            Some('}') => Ok(Token::RightBrace),
            Some('(') => Ok(Token::LeftBracket),
            Some(')') => Ok(Token::RightBracket),
            Some('[') => Ok(Token::LeftSquare),
            Some(']') => Ok(Token::RightSquare),
            Some('.') => Ok(Token::Dot),
            Some('-') => Ok(Token::Minus),
            Some('+') => Ok(Token::Plus),
            Some('*') => Ok(Token::Asterisk),
            Some(';') => Ok(Token::Semicolon),
            Some(':') => Ok(Token::Colon),
            Some('?') => {
                if self.match_advance('?') { Ok(Token::QuestionQuestion) }
                else if self.match_advance('.') { Ok(Token::QuestionDot) }
                else { Ok(Token::Question) }
            },
            Some('/') => {
                if self.match_advance('/') { self.skip_comments() }
                else { Ok(Token::Slash) }
            }
            Some('=') => {
                if self.match_advance('=') { Ok(Token::Equal) }
                else { Ok(Token::Assign) }
            },
            Some('!') => {
                if self.match_advance('=') { Ok(Token::NotEqual) }
                else if self.match_advance('!') { Ok(Token::Assert) }
                else { Ok(Token::LNot) }
            },
            Some('<') => {
                if self.match_advance('=') { Ok(Token::LessEqual) }
                else { Ok(Token::Less) }
            },
            Some('>') => {
                if self.match_advance('=') { Ok(Token::GreaterEqual) }
                else { Ok(Token::Greater) }
            },
            Some('&') => {
                if self.match_advance('&') { Ok(Token::LAnd) }
                else { Err(self.error_here(LexErrorKind::IncompleteOperator { expected: "&&" })) }
            },
            Some('|') => {
                if self.match_advance('|') { Ok(Token::LOr) }
                else { Err(self.error_here(LexErrorKind::IncompleteOperator { expected: "||" })) }
            },
            Some('"') => {
                let mut s: String = String::new();
                loop {
                    match self.advance() {
                        Some('"') => break,
                        Some(c) => s.push(c),
                        None => return Err(self.error_at_token_start(LexErrorKind::UnterminatedString)),
                    }
                }
                Ok(Token::String(s))
            },
            Some(c) if c.is_ascii_digit() => {
                let mut s: String = c.to_string();
                let mut is_float: bool = false;
                
                loop {
                    match self.peek() {
                        Some(d) if d.is_ascii_digit() => { self.advance(); s.push(d); },
                        Some('.') if !is_float => { self.advance(); s.push('.'); is_float = true; },
                        _ => break,
                    }
                }

                if is_float {
                    match s.parse::<f64>() {
                        Ok(n) => Ok(Token::Float(n)),
                        Err(_) => Err(self.error_at_token_start(LexErrorKind::InvalidNumber(s))),
                    }
                }
                else {
                    match s.parse::<i64>() {
                        Ok(n) => Ok(Token::Int(n)),
                        Err(_) => Err(self.error_at_token_start(LexErrorKind::IntegerOutOfRange(s))),
                    }
                }
            },
            Some(c) if c.is_alphabetic() || c == '_' => {
                let mut s: String = c.to_string();

                loop {
                    match self.peek() {
                        Some(a) if a.is_alphanumeric() || a == '_' => { s.push(a); self.advance(); },
                        _ => break,
                    }
                }

                Ok(match s.as_str() {
                    "if" => Token::If,
                    "else" => Token::Else,
                    "for" => Token::For,
                    "while" => Token::While,
                    "class" => Token::Class,
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    "null" => Token::Null,
                    "void" => Token::Void,
                    "static" => Token::Static,
                    "int" => Token::DataType(DataType::Int),
                    "float" => Token::DataType(DataType::Float),
                    "string" => Token::DataType(DataType::String),
                    "bool" => Token::DataType(DataType::Bool),
                    "print" => Token::Print,
                    "fun" => Token::Fun,
                    "return" => Token::Return,
                    "continue" => Token::Continue,
                    "break" => Token::Break,
                    "override" => Token::Override,
                    _ => Token::Identifier(s),
                })
            },
            Some(' ') | Some('\t') | Some('\n') | Some('\r') => {
                loop {
                    match self.peek() {
                        Some(' ') |
                        Some('\t') |
                        Some('\n') |
                        Some('\r') => { self.advance(); },
                        _ => break,
                    }
                }

                self.read_next_token()
            }
            Some(invalid) => Err(self.error_here(LexErrorKind::InvalidCharacter(invalid))),
            None => Ok(Token::EOF),
        };
    }
    fn skip_comments(&mut self) -> Result<Token, LexError> {
        loop {
            match self.advance() {
                Some('\n') => {
                    if !(self.peek() == Some('/') && self.peek_ahead(1) == Some('/')) {
                        return self.read_next_token();
                    }             
                },
                Some(_) => (),
                None => return Ok(Token::EOF),
            }
        }
    }
}

/// Splits source text into tokens. `spans[i]` is where `tokens[i]` starts; the last token is
/// always `Token::EOF`.
pub fn lex_chars(chars: Chars<'_>) -> Result<(Vec<Token>, Vec<Span>), LexError> {
    let mut scanner: Scanner = Scanner::from_source(chars);
    let mut tokens: Vec<Token> = Vec::new();
    let mut spans: Vec<Span> = Vec::new();
    loop {
        let token: Token = scanner.read_next_token()?;
        let is_eof: bool = matches!(token, Token::EOF);

        tokens.push(token);
        spans.push(scanner.token_start);

        if is_eof {
            return Ok((tokens, spans));
        }
    }
}
