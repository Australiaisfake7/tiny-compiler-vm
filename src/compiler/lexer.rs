use std::fmt;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBracket, RightBracket, LeftBrace, RightBrace,
    Dot, Minus, Plus, Slash, Asterisk, Semicolon, Colon, Question,

    Identifier(String), String(String), Int(i64), Float(f64), Bool(bool),

    Assign, Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    LAnd, LOr, LNot,

    Print, Fun, DataType(DataType), Static, Null, Void, Override,

    If, Else, For, While, Class, Return, Continue, Break,

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int, Float, String, Bool, Instance(String), Nullable(Box<DataType>), Null, Void,
}

#[derive(Debug)]
pub struct LexError {
    _message: String,
    _line: usize,
    _character: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (line {}, character {})", self._message, self._line, self._character)
    }
}

impl std::error::Error for LexError {}

struct Scanner<'a> {
    source: Chars<'a>,
    _line: usize,
    _character: usize,
}

impl<'a> Default for Scanner<'a> {
    fn default() -> Self {
        return Self {
            source: "".chars(),
            _line: 1,
            _character: 0,
        };
    }
}

impl<'a> Scanner<'a> {
    fn from_source(source: Chars<'a>) -> Self {
        return Self {
            source: source,
            ..Default::default()
        };
    }
    fn new() -> Self {
        return Default::default();
    }
    fn advance(&mut self) -> Option<char> {
        match self.peek() {
            Some('\n') => {
                self._line += 1;
                self._character = 0;
            },
            Some(_) => { self._character += 1; },
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
        return match self.advance() {
            Some('{') => Ok(Token::LeftBrace),
            Some('}') => Ok(Token::RightBrace),
            Some('(') => Ok(Token::LeftBracket),
            Some(')') => Ok(Token::RightBracket),
            Some('.') => Ok(Token::Dot),
            Some('-') => Ok(Token::Minus),
            Some('+') => Ok(Token::Plus),
            Some('*') => Ok(Token::Asterisk),
            Some(';') => Ok(Token::Semicolon),
            Some(':') => Ok(Token::Colon),
            Some('?') => Ok(Token::Question),
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
                else {
                    Err(LexError {
                        _message: "Expected '&&', single '&' is not valid.".to_string(),
                        _line: self._line,
                        _character: self._character,
                    })
                }
            },
            Some('|') => {
                if self.match_advance('|') { Ok(Token::LOr) }
                else {
                    Err(LexError {
                        _message: "Expected '||', single '|' is not valid.".to_string(),
                        _line: self._line,
                        _character: self._character,
                    })
                }
            },
            Some('"') => {
                let mut s: String = String::new();
                loop {
                    match self.advance() {
                        Some('"') => break,
                        Some(c) => s.push(c),
                        None => return Err(LexError {
                            _message: "Unfinished string literal".to_string(),
                            _line: self._line,
                            _character: self._character,
                        }),
                    }
                }
                Ok(Token::String(s))
            },
            Some(c) if c.is_ascii_digit() => {
                let _line: usize = self._line;
                let _character: usize = self._character;
                
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
                        Err(e) => Err(LexError { _message: e.to_string(), _line: _line, _character: _character }),
                    }
                }
                else {
                    match s.parse::<i64>() {
                        Ok(n) => Ok(Token::Int(n)),
                        Err(e) => Err(LexError { _message: e.to_string(), _line: _line, _character: _character }),
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
            Some(invalid) => Err(LexError { 
                _message: format!("Invalid character {} found", invalid),
                _line: self._line,
                _character: self._character 
            }),
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

pub fn lex_chars(chars: Chars<'_>) -> Result<Vec<Token>, LexError> {
    let mut scanner: Scanner = Scanner::from_source(chars);
    let mut tokens: Vec<Token> = Vec::new();
    loop {
        let token = scanner.read_next_token();
        match token {
            Ok(t) if matches!(t, Token::EOF) => { tokens.push(t); return Ok(tokens); },
            Ok(t) => tokens.push(t),
            Err(e) => return Err(e),
        }
    }
}