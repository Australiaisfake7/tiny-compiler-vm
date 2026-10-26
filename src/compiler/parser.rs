use serde::{Deserialize, Serialize};

use super::lexer::{Token, DataType};
use crate::error::Span;
use std::{convert::TryFrom, fmt, rc::Rc};

#[derive(Debug)]
pub enum ParseErrorKind {
    /// `expected` describes what the grammar allows here ("';'", "an identifier", ...).
    UnexpectedToken { expected: &'static str, found: Token },
    InvalidAssignmentTarget(Box<Expression>),
    VoidNotAllowed,
    InvalidArraySize(i64),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::UnexpectedToken { expected, found } =>
                write!(f, "expected {expected}, found {found}"),
            ParseErrorKind::InvalidAssignmentTarget(target) =>
                write!(f, "'{target}' is not a valid assignment target"),
            ParseErrorKind::VoidNotAllowed => write!(f, "'void' is not allowed here"),
            ParseErrorKind::InvalidArraySize(size) =>
                write!(f, "array size must be at least 1, found {size}"),
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "syntax error at {}: {}", self.span, self.kind)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    LAnd,
    LOr,
    Coalesce,
}
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    LNot,
    Negate,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            BinaryOp::Add => "+", BinaryOp::Subtract => "-", BinaryOp::Multiply => "*", BinaryOp::Divide => "/",
            BinaryOp::Equal => "==", BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<", BinaryOp::LessEqual => "<=", BinaryOp::Greater => ">", BinaryOp::GreaterEqual => ">=",
            BinaryOp::LAnd => "&&", BinaryOp::LOr => "||", BinaryOp::Coalesce => "??",
        })
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            UnaryOp::LNot => "!",
            UnaryOp::Negate => "-",
        })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralType {
    String(Rc<str>),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}
impl fmt::Display for LiteralType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralType::String(value) => write!(f, "\"{value}\""),
            LiteralType::Int(value) => write!(f, "{value}"),
            LiteralType::Float(value) => write!(f, "{value:?}"),
            LiteralType::Bool(value) => write!(f, "{value}"),
            LiteralType::Null => write!(f, "null"),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct VariableData {
    pub data_type: DataType,
    pub value: LiteralType,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionData {
    pub data_type: DataType,
    pub parameters: Vec<(DataType, String)>,
    pub block: Vec<Statement>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOp,
        right: Box<Expression>,
    },
    Literal(LiteralType),
    Variable(String),
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        parameters: Vec<Expression>,
    },
    MemberAccess {
        class: Box<Expression>,
        member: String,
    },
    ArrayAccess {
        target: Box<Expression>,
        index: Box<Expression>,
    },
    ArrayLiteral(Vec<Expression>),
    Assert(Box<Expression>),
    SafeAccess {
        class: Box<Expression>,
        member: String,
    },
    Evaluated(DataType),
    ArrayFill {
        value: Box<Expression>,
        length: i64,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Box<Expression>),
    Declaration { name: String, value: Box<Expression>, data_type: DataType, is_static: bool },
    Block(Vec<Statement>),
    If { condition: Box<Expression>, block: Vec<Statement> },
    IfElse { condition: Box<Expression>, block1: Vec<Statement>, block2: Vec<Statement> },
    Print(Box<Expression>),
    While { condition: Box<Expression>, block: Vec<Statement> },
    For { initializer: Box<Statement>, condition: Box<Expression>, update: Box<Expression>, block: Vec<Statement> },
    Function { name: String, data: FunctionData, should_override: bool, is_static: bool },
    Return(Option<Box<Expression>>),
    Class { name: String, block: Vec<Statement>, parent: Option<String> },
    Continue, Break,
}

/// Prints an expression in source syntax, for use in error messages.
impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn operand(f: &mut fmt::Formatter<'_>, expression: &Expression, wrap: bool) -> fmt::Result {
            if wrap { write!(f, "({expression})") } else { write!(f, "{expression}") }
        }
        fn list(f: &mut fmt::Formatter<'_>, items: &[Expression]) -> fmt::Result {
            for (i, item) in items.iter().enumerate() {
                if i > 0 { write!(f, "; ")?; }
                write!(f, "{item}")?;
            }
            Ok(())
        }
        // Operators bind looser than member access, calls and indexing.
        let compound = |e: &Expression| matches!(e, Expression::Binary { .. } | Expression::Assignment { .. });
        let compound_or_unary = |e: &Expression| compound(e) || matches!(e, Expression::Unary { .. });

        match self {
            Expression::Binary { left, operator, right } => {
                operand(f, left, compound(left))?;
                write!(f, " {operator} ")?;
                operand(f, right, compound(right))
            },
            Expression::Unary { operator, right } => {
                write!(f, "{operator}")?;
                operand(f, right, compound(right))
            },
            Expression::Literal(literal) => write!(f, "{literal}"),
            Expression::Variable(name) => write!(f, "{name}"),
            Expression::Assignment { target, value } => write!(f, "{target} = {value}"),
            Expression::Call { callee, parameters } => {
                operand(f, callee, compound_or_unary(callee))?;
                write!(f, "(")?;
                list(f, parameters)?;
                write!(f, ")")
            },
            Expression::MemberAccess { class, member } => {
                operand(f, class, compound_or_unary(class))?;
                write!(f, ".{member}")
            },
            Expression::ArrayAccess { target, index } => {
                operand(f, target, compound_or_unary(target))?;
                write!(f, "[{index}]")
            },
            Expression::ArrayLiteral(items) => {
                write!(f, "[")?;
                list(f, items)?;
                write!(f, "]")
            },
            Expression::Assert(expression) => {
                operand(f, expression, compound_or_unary(expression))?;
                write!(f, "!!")
            },
            Expression::SafeAccess { class, member } => {
                operand(f, class, compound_or_unary(class))?;
                write!(f, "?.{member}")
            },
            Expression::Evaluated(data_type) => write!(f, "<{data_type} value>"),
            Expression::ArrayFill { value, length } => {
                write!(f, "[")?;
                operand(f, value, compound(value))?;
                write!(f, ": ")?;
                write!(f, "{}", Expression::Literal(LiteralType::Int(*length)))?;
                write!(f, "]")
            }
        }
    }
}

impl Statement {
    /// A short description of the kind of statement, for use in error messages.
    pub fn description(&self) -> &'static str {
        match self {
            Statement::Expression(_) => "an expression statement",
            Statement::Declaration { .. } => "a variable declaration",
            Statement::Block(_) => "a block",
            Statement::If { .. } | Statement::IfElse { .. } => "an 'if' statement",
            Statement::Print(_) => "a 'print' statement",
            Statement::While { .. } => "a 'while' loop",
            Statement::For { .. } => "a 'for' loop",
            Statement::Function { .. } => "a function declaration",
            Statement::Return(_) => "a 'return' statement",
            Statement::Class { .. } => "a class declaration",
            Statement::Continue => "a 'continue' statement",
            Statement::Break => "a 'break' statement",
        }
    }
}

impl TryFrom<&Token> for BinaryOp {
    type Error = Token;

    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        return match token {
            Token::Plus => Ok(BinaryOp::Add),
            Token::Minus => Ok(BinaryOp::Subtract),
            Token::Asterisk => Ok(BinaryOp::Multiply),
            Token::Slash => Ok(BinaryOp::Divide),
            Token::Equal => Ok(BinaryOp::Equal),
            Token::NotEqual => Ok(BinaryOp::NotEqual),
            Token::Less => Ok(BinaryOp::Less),
            Token::LessEqual => Ok(BinaryOp::LessEqual),
            Token::Greater => Ok(BinaryOp::Greater),
            Token::GreaterEqual => Ok(BinaryOp::GreaterEqual),
            Token::LAnd => Ok(BinaryOp::LAnd),
            Token::LOr => Ok(BinaryOp::LOr),
            _ => Err(token.clone()),
        }
    }
}

impl TryFrom<&Token> for UnaryOp {
    type Error = Token;

    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        return match token {
            Token::LNot => Ok(UnaryOp::LNot),
            Token::Minus => Ok(UnaryOp::Negate),
            _ => Err(token.clone()),
        }
    }
}

/// Parses a token stream. `spans[i]` must be where `tokens[i]` starts, and the last token must be
/// `Token::EOF` (which is what the lexer produces).
pub fn parse_tokens(tokens: Vec<Token>, spans: Vec<Span>) -> Result<Vec<Statement>, ParseError> {
    let mut parser: Parser = Parser::from_tokens(tokens, spans);

    let mut statements: Vec<Statement> = Vec::new();

    while !parser.match_advance(&[Token::EOF]) {
        statements.push(parser.statement()?);
    }

    Ok(statements)
}

struct Parser {
    tokens: Vec<Token>,
    spans: Vec<Span>,
    position: usize
}

impl Parser {
    fn from_tokens(tokens: Vec<Token>, spans: Vec<Span>) -> Self {
        assert!(!tokens.is_empty() && tokens.len() == spans.len(), "every token needs a span, and the stream ends with EOF");
        Self { tokens, spans, position: 0 }
    }
    /// Reading past the end of the stream yields the final `EOF` token, so the cursor never fails.
    fn index(&self, position: usize) -> usize {
        position.min(self.tokens.len() - 1)
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.index(self.position)]
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.index(self.position.saturating_sub(1))]
    }
    fn advance(&mut self) -> &Token {
        let consumed: usize = self.index(self.position);
        self.position += 1;
        &self.tokens[consumed]
    }
    fn match_advance(&mut self, tokens: &[Token]) -> bool {
        let disc = std::mem::discriminant(self.peek());

        for token in tokens {
            if std::mem::discriminant(token) == disc {
                self.advance();
                return true;
            }
        }
        return false;
    }
    /// Consumes `token`, or fails with an error about the current token.
    fn expect(&mut self, token: Token, expected: &'static str) -> Result<(), ParseError> {
        if self.match_advance(&[token]) {
            Ok(())
        }
        else {
            Err(self.unexpected(expected))
        }
    }
    fn expect_identifier(&mut self, expected: &'static str) -> Result<String, ParseError> {
        match self.advance() {
            Token::Identifier(name) => Ok(name.clone()),
            _ => Err(self.unexpected_previous(expected)),
        }
    }
    /// An error about the token that has not been consumed yet.
    fn unexpected(&self, expected: &'static str) -> ParseError {
        let position: usize = self.index(self.position);
        self.error_at(position, ParseErrorKind::UnexpectedToken { expected, found: self.tokens[position].clone() })
    }
    /// An error about the token that was just consumed.
    fn unexpected_previous(&self, expected: &'static str) -> ParseError {
        let position: usize = self.index(self.position.saturating_sub(1));
        self.error_at(position, ParseErrorKind::UnexpectedToken { expected, found: self.tokens[position].clone() })
    }
    fn error_at(&self, position: usize, kind: ParseErrorKind) -> ParseError {
        ParseError { kind, span: self.spans[self.index(position)] }
    }
    fn error_previous(&self, kind: ParseErrorKind) -> ParseError {
        self.error_at(self.position.saturating_sub(1), kind)
    }
    fn is_declaration(&self) -> bool {
        let mut i: usize = self.position;

        match self.tokens.get(i) {
            Some(Token::DataType(_)) => return true,
            Some(Token::Identifier(_)) => i += 1,
            _ => return false,
        }

        loop {
            match self.tokens.get(i) {
                Some(Token::Question) => i += 1,
                Some(Token::LeftSquare) => {
                    if !matches!(self.tokens.get(i + 1), Some(Token::Int(_))) || !matches!(self.tokens.get(i + 2), Some(Token::RightSquare)) {
                        return false;
                    }
                    i += 3;
                },
                Some(Token::Identifier(_)) => return true,
                Some(_) | None => return false,
            }
        }
    }
    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_advance(&[Token::LeftBrace]) {
            return self.block();
        }
        if self.match_advance(&[Token::If]) {
            return self.if_else();
        }
        if self.match_advance(&[Token::Print]) {
            return self.print();
        }
        if self.match_advance(&[Token::While]) {
            return self.while_loop();
        }
        if self.match_advance(&[Token::For]) {
            return self.for_loop();
        }
        if self.match_advance(&[Token::Fun]) {
            return self.function(false);
        }
        if self.match_advance(&[Token::Return]) {
            return self.return_statement();
        }
        if self.match_advance(&[Token::Class]) {
            return self.class();
        }
        if self.match_advance(&[Token::Continue]) {
            return self.continue_statement();
        }
        if self.match_advance(&[Token::Break]) {
            return self.break_statement();
        }
        if self.match_advance(&[Token::Static]) {
            if self.match_advance(&[Token::Fun]) {
                return self.function(true);
            }
            if self.is_declaration() {
                return self.declaration(true);
            }
            return Err(self.unexpected("a declaration or 'fun' after 'static'"));
        }
        if self.is_declaration() {
            return self.declaration(false);
        }

        let expr: Box<Expression> = self.expression()?;
        self.expect(Token::Semicolon, "';'")?;

        return Ok(Statement::Expression(expr));
    }
    fn declaration(&mut self, is_static: bool) -> Result<Statement, ParseError> {
        let d: DataType = self.get_data_type(false)?;

        let n: String = self.expect_identifier("a variable name")?;
        let v: Box<Expression> = match self.advance() {
            Token::Semicolon => return Ok(Statement::Declaration { name: n, value: Box::new(Expression::Literal(LiteralType::Null)), data_type: d, is_static }),
            Token::Assign => self.expression()?,
            _ => return Err(self.unexpected_previous("'=' or ';'")),
        };
        self.expect(Token::Semicolon, "';'")?;

        return Ok(Statement::Declaration { name: n, value: v, data_type: d, is_static });
    }
    fn block(&mut self) -> Result<Statement, ParseError> {
        let mut stmts: Vec<Statement> = Vec::new();

        loop {
            match self.peek() {
                Token::RightBrace => { self.advance(); return Ok(Statement::Block(stmts)); },
                Token::EOF => return Err(self.unexpected("'}'")),
                _ => stmts.push(self.statement()?),
            }
        }
    }
    fn if_else(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::LeftBracket, "'('")?;

        let condition: Box<Expression> = self.expression()?;

        self.expect(Token::RightBracket, "')'")?;
        self.expect(Token::LeftBrace, "'{'")?;

        let block1: Vec<Statement> = match self.block()? {
            Statement::Block(b) => b,
            _ => unreachable!(),
        };

        if !self.match_advance(&[Token::Else]) {
            return Ok(Statement::If { condition: condition, block: block1 });
        }

        if self.match_advance(&[Token::If]) {
            return Ok(Statement::IfElse { condition, block1, block2: vec![self.if_else()?] });
        }

        self.expect(Token::LeftBrace, "'{'")?;

        let block2: Vec<Statement> = match self.block()? {
            Statement::Block(b) => b,
            _ => unreachable!(),
        };

        return Ok(Statement::IfElse { condition: condition, block1: block1, block2: block2 });
    }
    fn print(&mut self) -> Result<Statement, ParseError> {
        let expr: Box<Expression> = self.expression()?;

        self.expect(Token::Semicolon, "';'")?;

        return Ok(Statement::Print(expr));
    }
    fn while_loop(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::LeftBracket, "'('")?;

        let condition: Box<Expression> = self.expression()?;

        self.expect(Token::RightBracket, "')'")?;
        self.expect(Token::LeftBrace, "'{'")?;

        let block: Vec<Statement> = match self.block()? {
            Statement::Block(b) => b,
            _ => unreachable!(),
        };

        Ok(Statement::While { condition, block })
    }
    fn for_loop(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::LeftBracket, "'('")?;

        let initializer: Statement = if self.match_advance(&[Token::Semicolon]) {
            Statement::Block(Vec::new())
        }
        else if self.is_declaration() {
            self.declaration(false)?
        }
        else if starts_expression(self.peek()) {
            let s: Statement = Statement::Expression(self.expression()?);
            self.expect(Token::Semicolon, "';'")?;
            s
        }
        else {
            return Err(self.unexpected("a declaration or expression in the for loop initializer"));
        };
        let condition: Box<Expression> = if self.match_advance(&[Token::Semicolon]) {
            Box::new(Expression::Literal(LiteralType::Bool(true)))
        }
        else {
            let e: Box<Expression> = self.expression()?;
            self.expect(Token::Semicolon, "';'")?;
            e
        };

        let update: Box<Expression> = if self.match_advance(&[Token::RightBracket]) {
            Box::new(Expression::Literal(LiteralType::Null))
        }
        else {
            let e :Box<Expression> = self.expression()?;
            self.expect(Token::RightBracket, "')'")?;
            e
        };
 
        self.expect(Token::LeftBrace, "'{'")?;

        let block: Vec<Statement> = match self.block()? {
            Statement::Block(b) => b,
            _ => unreachable!(),
        };

        Ok(Statement::For { initializer: Box::new(initializer), condition, update, block })
    }
    fn function(&mut self, is_static: bool) -> Result<Statement, ParseError> {
        let should_override: bool = self.match_advance(&[Token::Override]);

        let data_type: DataType = self.get_data_type(true)?;

        let name: String = self.expect_identifier("a function name")?;

        self.expect(Token::LeftBracket, "'('")?;

        let mut parameters: Vec<(DataType, String)> = Vec::new();

        loop {
            match self.peek() {
                Token::RightBracket => break,
                _ => parameters.push(self.parameter()?),
            }
        }

        self.expect(Token::RightBracket, "')'")?;
        self.expect(Token::LeftBrace, "'{'")?;

        let block: Vec<Statement> = match self.block()? {
            Statement::Block(b) => b,
            _ => unreachable!(),
        };

        Ok(Statement::Function { name, data: FunctionData { data_type, parameters, block }, should_override, is_static })
    }
    fn parameter(&mut self) -> Result<(DataType, String), ParseError> {
        let data_type: DataType = self.get_data_type(false)?;

        let s: String = self.expect_identifier("a parameter name")?;

        if self.peek() != &Token::RightBracket && !self.match_advance(&[Token::Semicolon]) {
            return Err(self.unexpected("';' or ')'"));
        }

        Ok((data_type, s))
    }
    fn return_statement(&mut self) -> Result<Statement, ParseError> {
        let expr: Option<Box<Expression>> = self.optional_expression()?;

        self.expect(Token::Semicolon, "';'")?;
        Ok(Statement::Return(expr))
    }
    fn class(&mut self) -> Result<Statement, ParseError> {
        let name: String = self.expect_identifier("a class name")?;
        let parent: Option<String> = if self.match_advance(&[Token::Colon]) {
            Some(self.expect_identifier("a parent class name")?)
        }
        else {
            None
        };
        self.expect(Token::LeftBrace, "'{'")?;

        match self.block()? {
            Statement::Block(b) => Ok(Statement::Class { name, block: b, parent }),
            _ => unreachable!(),
        }
    }
    fn continue_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Semicolon, "';'")?;

        Ok(Statement::Continue)
    }
    fn break_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Semicolon, "';'")?;

        Ok(Statement::Break)
    }
    fn optional_expression(&mut self) -> Result<Option<Box<Expression>>, ParseError> {
        if starts_expression(self.peek()) {
            Ok(Some(self.expression()?))
        }
        else {
            Ok(None)
        }
    }
    fn expression(&mut self) -> Result<Box<Expression>, ParseError> {
        return self.assignment();
    }
    fn assignment(&mut self) -> Result<Box<Expression>, ParseError> {
        let expr: Box<Expression> = self.null_coalesce()?;

        if self.match_advance(&[Token::Assign]) {
            let assign_position: usize = self.position - 1;
            let value: Box<Expression> = self.assignment()?;

            match &*expr {
                Expression::Variable(_) | Expression::MemberAccess { .. } | Expression::ArrayAccess { .. } => return Ok(Box::new(Expression::Assignment { target: expr, value })),
                _ => return Err(self.error_at(assign_position, ParseErrorKind::InvalidAssignmentTarget(expr))),
            }
        }

        Ok(expr)
    }
    fn null_coalesce(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr :Box<Expression> = self.logical_or()?;

        while self.match_advance(&[Token::QuestionQuestion]) {
            let right: Box<Expression> = self.logical_or()?;
            expr = Box::new(Expression::Binary { left: expr, operator: BinaryOp::Coalesce, right });
        }

        Ok(expr)
    }
    /// Parses a left-associative chain `operand (operator operand)*`.
    fn binary_chain(&mut self, operators: &[Token], operand: fn(&mut Self) -> Result<Box<Expression>, ParseError>) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = operand(self)?;

        while self.match_advance(operators) {
            let operator: BinaryOp = BinaryOp::try_from(self.previous())
                .map_err(|_| self.unexpected_previous("a binary operator"))?;
            let right: Box<Expression> = operand(self)?;
            expr = Box::new(Expression::Binary { left: expr, operator, right });
        }

        Ok(expr)
    }
    fn logical_or(&mut self) -> Result<Box<Expression>, ParseError> {
        self.binary_chain(&[Token::LOr], Self::logical_and)
    }
    fn logical_and(&mut self) -> Result<Box<Expression>, ParseError> {
        self.binary_chain(&[Token::LAnd], Self::equality)
    }
    fn equality(&mut self) -> Result<Box<Expression>, ParseError> {
        self.binary_chain(&[Token::Equal, Token::NotEqual], Self::inequality)
    }
    fn inequality(&mut self) -> Result<Box<Expression>, ParseError> {
        self.binary_chain(&[Token::Less, Token::LessEqual, Token::Greater, Token::GreaterEqual], Self::term)
    }
    fn term(&mut self) -> Result<Box<Expression>, ParseError> {
        self.binary_chain(&[Token::Plus, Token::Minus], Self::factor)
    }
    fn factor(&mut self) -> Result<Box<Expression>, ParseError> {
        self.binary_chain(&[Token::Asterisk, Token::Slash], Self::unary)
    }
    fn unary(&mut self) -> Result<Box<Expression>, ParseError> {
        if self.match_advance(&[Token::LNot, Token::Minus]) {
            let op: UnaryOp = UnaryOp::try_from(self.previous())
                .map_err(|_| self.unexpected_previous("a unary operator"))?;
            let right: Box<Expression> = self.unary()?;
            let expr: Box<Expression> = Box::new(Expression::Unary {
                operator: op,
                right: right
            });
            return Ok(expr);
        }

        return Ok(self.call()?);
    }
    fn call(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.primary()?;

        loop {
            if self.match_advance(&[Token::LeftBracket]) {
                let mut parameters: Vec<Expression> = Vec::new();

                loop {
                    if self.match_advance(&[Token::RightBracket]) {
                        break;
                    }

                    parameters.push(*self.expression()?);
                    if self.match_advance(&[Token::RightBracket]) {
                        break;
                    }
                    self.expect(Token::Semicolon, "';' or ')'")?;
                }

                expr = Box::new(Expression::Call { callee: expr, parameters });
            }
            else if self.match_advance(&[Token::Dot]) {
                let member: String = self.expect_identifier("a member name")?;
                expr = Box::new(Expression::MemberAccess { class: expr, member });
            }
            else if self.match_advance(&[Token::QuestionDot]) {
                let member: String = self.expect_identifier("a member name")?;
                expr = Box::new(Expression::SafeAccess { class: expr, member });
            }
            else if self.match_advance(&[Token::LeftSquare]) {
                let index: Box<Expression> = self.expression()?;

                self.expect(Token::RightSquare, "']'")?;

                expr = Box::new(Expression::ArrayAccess { target: expr, index });
            }
            else if self.match_advance(&[Token::Assert]) {
                expr = Box::new(Expression::Assert(expr));
            }
            else {
                break;
            }
        }

        Ok(expr)
    }
    fn primary(&mut self) -> Result<Box<Expression>, ParseError> {
        return match self.advance() {
            Token::Bool(v) => Ok(Box::new(Expression::Literal(LiteralType::Bool(*v)))),
            Token::Int(v) => Ok(Box::new(Expression::Literal(LiteralType::Int(*v)))),
            Token::Float(v) => Ok(Box::new(Expression::Literal(LiteralType::Float(*v)))),
            Token::String(v) => Ok(Box::new(Expression::Literal(LiteralType::String(Rc::from(v.as_str()))))),
            Token::Null => Ok(Box::new(Expression::Literal(LiteralType::Null))),
            Token::Identifier(name) => Ok(Box::new(Expression::Variable(name.clone()))),
            Token::LeftBracket => {
                let expr: Box<Expression> = self.expression()?;
                self.expect(Token::RightBracket, "')'")?;
                Ok(expr)
            },
            Token::LeftSquare => {
                let mut vec: Vec<Expression> = vec![*self.expression()?];

                loop {
                    match self.advance() {
                        Token::Semicolon => vec.push(*self.expression()?),
                        Token::RightSquare => break,
                        Token::Colon if vec.len() == 1 => {
                            let len: i64 = match &*self.expression()? {
                                Expression::Literal(LiteralType::Int(i)) if *i > 0_i64 => *i,
                                _ => return Err(self.unexpected_previous("Positive Integer Literal")),
                            };

                            self.expect(Token::RightSquare, "']'")?;

                            return Ok(Box::new(Expression::ArrayFill { value: Box::new(vec.pop().unwrap()), length: len }))
                        }
                        _ => return Err(self.unexpected_previous("';' or ']'")),
                    }
                }

                Ok(Box::new(Expression::ArrayLiteral(vec)))
            },
            _ => Err(self.unexpected_previous("an expression")),
        };
    }
    fn get_data_type(&mut self, void_allowed: bool) -> Result<DataType, ParseError> {
        let token: &Token = self.advance();

        let mut d: DataType = match token {
            Token::Void => {
                if void_allowed {
                    DataType::Void
                }
                else {
                    return Err(self.error_previous(ParseErrorKind::VoidNotAllowed));
                }
            }
            Token::DataType(d) => d.clone(),
            Token::Identifier(s) => DataType::Instance(s.clone()),
            _ => return Err(self.unexpected_previous("a data type")),
        };

        loop {
            if self.match_advance(&[Token::Question]) {
                if d == DataType::Void {
                    return Err(self.error_previous(ParseErrorKind::VoidNotAllowed));
                }
                d = DataType::Nullable(Box::new(d));
            }
            else if self.match_advance(&[Token::LeftSquare]) {
                let len: usize = match self.advance() {
                    Token::Int(l) => {
                        let size: i64 = *l;
                        if size < 1 { return Err(self.error_previous(ParseErrorKind::InvalidArraySize(size))) }
                        size as usize
                    },
                    _ => return Err(self.unexpected_previous("an integer literal")),
                };

                self.expect(Token::RightSquare, "']'")?;

                d = DataType::Array(Box::new(d), len);
            }
            else {
                break;
            }
        }

        Ok(d)
    }
}



fn starts_expression(token: &Token) -> bool {
    matches!(token, Token::Bool(_) | Token::Int(_) | Token::Float(_) | Token::String(_) | Token::Null | Token::Identifier(_) | Token::LeftBracket | Token::LNot | Token::Minus | Token::LeftSquare)
}