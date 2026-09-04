use super::lexer::{Token, DataType};
use std::convert::TryFrom;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { expected: &'static str, got: Token },
    UnexpectedReadIndex(usize),
    UnexpectedAssignmentTarget(Box<Expression>),
}
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
}
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    LNot,
    Negate,
}
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
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

pub fn parse_tokens(tokens: Vec<Token>) -> Result<Vec<Statement>, ParseError> {
    let mut parser: Parser = Parser::from_tokens(tokens);

    let mut statements: Vec<Statement> = Vec::new();

    while !parser.match_advance(&[Token::EOF]) {
        statements.push(parser.statement()?);
    }

    Ok(statements)
}

struct Parser {
    tokens: Vec<Token>,
    position: usize
}

impl Default for Parser {
    fn default() -> Self {
        return Self {
            tokens: Vec::new(),
            position: 0
        };
    }
}

impl Parser {
    fn new() -> Self {
        return Default::default();
    }
    fn from_tokens(tokens: Vec<Token>) -> Self {
        let mut p: Self = Self::new();
        p.tokens = tokens;
        return p;
    }
    fn peek(&self) -> Result<&Token, ParseError> {
        self.tokens.get(self.position).ok_or(ParseError::UnexpectedReadIndex(self.position))
    }
    fn previous(&self) -> Result<&Token, ParseError> {
        if self.position > 0 {
            return self.tokens.get(self.position - 1).ok_or(ParseError::UnexpectedReadIndex(self.position - 1));
        }
        return Err(ParseError::UnexpectedReadIndex(usize::MAX));
    }
    fn advance(&mut self) -> Result<&Token, ParseError> {
        self.position += 1;
        self.previous()
    }
    fn match_advance(&mut self, tokens: &[Token]) -> bool {
        let disc = match self.peek() {
            Ok(v) => std::mem::discriminant(v),
            Err(_) => return false,
        };

        for token in tokens {
            if std::mem::discriminant(token) == disc {
                let _ = self.advance();
                return true;
            }
        }
        return false;
    }
    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_advance(&[Token::Let]) {
            return self.declaration(false);
        }
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
        if self.match_advance(&[Token::Let]) {
            return self.declaration(true);
        }
        if self.match_advance(&[Token::Fun]) {
            return self.function(true);
        }
        return Err(ParseError::UnexpectedToken { expected: "'let' or 'fun' after 'static'", got: self.peek()?.clone() });
    }

        let expr: Box<Expression> = self.expression()?;
        if !self.match_advance(&[Token::Semicolon]) {
            return Err(ParseError::UnexpectedToken { expected: "';'", got: self.peek()?.clone() })
        }

        return Ok(Statement::Expression(expr));
    }
    fn declaration(&mut self, is_static: bool) -> Result<Statement, ParseError> {
        let nullable: bool = self.match_advance(&[Token::Nullable]);

        let t: DataType = match self.advance()? {
            Token::DataType(d) => {
                let mut data_type: DataType = d.clone();
                if nullable { data_type = DataType::Nullable(Box::new(data_type)); }

                data_type
            },
            Token::Identifier(c) => {
                let mut data_type: DataType = DataType::Instance(c.clone());
                if nullable { data_type = DataType::Nullable(Box::new(data_type)); }

                data_type
            }
            token => return Err(ParseError::UnexpectedToken { expected: "Data Type", got: token.clone() })
        };
        let n: String = match self.advance()? {
            Token::Identifier(i) => i.clone(),
            token => return Err(ParseError::UnexpectedToken { expected: "Identifier", got: token.clone() })
        };
        let v: Box<Expression> = match self.advance()? {
            Token::Semicolon => return Ok(Statement::Declaration { name: n, value: Box::new(Expression::Literal(LiteralType::Null)), data_type: t, is_static }),
            Token::Assign => self.expression()?,
            token => return Err(ParseError::UnexpectedToken { expected: "';'", got: token.clone() })
        };
        if !self.match_advance(&[Token::Semicolon]) {
            return Err(ParseError::UnexpectedToken { expected: "';'", got: self.peek()?.clone() });
        }

        return Ok(Statement::Declaration { name: n, value: v, data_type: t, is_static });
    }
    fn block(&mut self) -> Result<Statement, ParseError> {
        let mut stmts: Vec<Statement> = Vec::new();

        loop {
            match self.peek()? {
                Token::RightBrace => { self.advance()?; return Ok(Statement::Block(stmts)); },
                Token::EOF => return Err(ParseError::UnexpectedToken { expected: "'}'", got: Token::EOF }),
                _ => stmts.push(self.statement()?),
            }
        }
    }
    fn if_else(&mut self) -> Result<Statement, ParseError> {
        if !self.match_advance(&[Token::LeftBracket]) {
            return Err(ParseError::UnexpectedToken { expected: "'('", got: self.peek()?.clone() });
        }

        let condition: Box<Expression> = self.expression()?;

        if !self.match_advance(&[Token::RightBracket]) {
            return Err(ParseError::UnexpectedToken { expected: "')'", got: self.peek()?.clone() });
        }
        if !self.match_advance(&[Token::LeftBrace]) {
            return Err(ParseError::UnexpectedToken { expected: "'{'", got: self.peek()?.clone() });
        }

        let block1: Vec<Statement> = match self.block()? {
            Statement::Block(b) => b,
            _ => unreachable!(),
        };

        if !self.match_advance(&[Token::Else]) {
            return Ok(Statement::If { condition: condition, block: block1 });
        }
        if !self.match_advance(&[Token::LeftBrace]) {
            return Err(ParseError::UnexpectedToken { expected: "'{'", got: self.peek()?.clone() });
        }

        let block2: Vec<Statement> = match self.block()? {
            Statement::Block(b) => b,
            _ => unreachable!(),
        };

        return Ok(Statement::IfElse { condition: condition, block1: block1, block2: block2 });
    }
    fn print(&mut self) -> Result<Statement, ParseError> {
        let expr: Box<Expression> = self.expression()?;

        if !self.match_advance(&[Token::Semicolon]) {
            return Err(ParseError::UnexpectedToken { expected: "';'", got: self.peek()?.clone() });
        }

        return Ok(Statement::Print(expr));
    }
    fn while_loop(&mut self) -> Result<Statement, ParseError> {
        if !self.match_advance(&[Token::LeftBracket]) {
            return Err(ParseError::UnexpectedToken { expected: "'('", got: self.peek()?.clone() });
        }

        let condition: Box<Expression> = self.expression()?;

        if !self.match_advance(&[Token::RightBracket]) {
            return Err(ParseError::UnexpectedToken { expected: "')'", got: self.peek()?.clone() });
        }
        if !self.match_advance(&[Token::LeftBrace]) {
            return Err(ParseError::UnexpectedToken { expected: "'{'", got: self.peek()?.clone() });
        }

        let block: Vec<Statement> = match self.block()? {
            Statement::Block(b) => b,
            _ => unreachable!(),
        };

        Ok(Statement::While { condition, block })
    }
    fn for_loop(&mut self) -> Result<Statement, ParseError> {
        if !self.match_advance(&[Token::LeftBracket]) {
            return Err(ParseError::UnexpectedToken { expected: "'('", got: self.peek()?.clone() });
        }

        let initializer: Statement = self.statement()?;
        let condition: Box<Expression> = self.expression()?;

        if !self.match_advance(&[Token::Semicolon]) {
            return Err(ParseError::UnexpectedToken { expected: "';'", got: self.peek()?.clone() });
        }

        let update: Box<Expression> = self.expression()?;
 
        if !self.match_advance(&[Token::RightBracket]) {
            return Err(ParseError::UnexpectedToken { expected: "')'", got: self.peek()?.clone() });
        }
        if !self.match_advance(&[Token::LeftBrace]) {
            return Err(ParseError::UnexpectedToken { expected: "'{'", got: self.peek()?.clone() });
        }

        let block: Vec<Statement> = match self.block()? {
            Statement::Block(b) => b,
            _ => unreachable!(),
        };

        Ok(Statement::For { initializer: Box::new(initializer), condition, update, block })
    }
    fn function(&mut self, is_static: bool) -> Result<Statement, ParseError> {
        let should_override: bool = self.match_advance(&[Token::Override]);

        let data_type: DataType = match self.peek()?.clone() {
            Token::DataType(d) => { self.advance()?; d },
            Token::Identifier(_) => DataType::Null,
            other => return Err(ParseError::UnexpectedToken { expected: "Function Name", got: other.clone() })
        };

        let name: String = match self.advance()? {
            Token::Identifier(i) => i.clone(),
            other => return Err(ParseError::UnexpectedToken { expected: "Function Name", got: other.clone() }),
        };

        if !self.match_advance(&[Token::LeftBracket]) {
            return Err(ParseError::UnexpectedToken { expected: "'('", got: self.peek()?.clone() });
        }

        let mut parameters: Vec<(DataType, String)> = Vec::new();

        loop {
            match self.peek()? {
                Token::DataType(_) => parameters.push(self.parameter()?),
                Token::RightBracket => break,
                other => return Err(ParseError::UnexpectedToken { expected: "Parameter", got: other.clone() }),
            }
        }

        if !self.match_advance(&[Token::RightBracket]) {
            return Err(ParseError::UnexpectedToken { expected: "')'", got: self.peek()?.clone() });
        }
        if !self.match_advance(&[Token::LeftBrace]) {
            return Err(ParseError::UnexpectedToken { expected: "'{'", got: self.peek()?.clone() });
        }

        let block: Vec<Statement> = match self.block()? {
            Statement::Block(b) => b,
            _ => unreachable!(),
        };

        Ok(Statement::Function { name, data: FunctionData { data_type, parameters, block }, should_override, is_static })
    }
    fn parameter(&mut self) -> Result<(DataType, String), ParseError> {
        let d: DataType = match self.advance()? {
            Token::DataType(d) => d.clone(),
            other => return Err(ParseError::UnexpectedToken { expected: "Data type", got: other.clone() }),
        };

        let s: String = match self.advance()? {
            Token::Identifier(i) => i.clone(),
            other => return Err(ParseError::UnexpectedToken { expected: "Identifier", got: other.clone() }),
        };

        if !self.match_advance(&[Token::Semicolon]) {
            return Err(ParseError::UnexpectedToken { expected: "';'", got: self.peek()?.clone() })
        }

        Ok((d, s))
    }
    fn return_statement(&mut self) -> Result<Statement, ParseError> {
        let expr: Option<Box<Expression>> = self.optional_expression()?;

        if !self.match_advance(&[Token::Semicolon]) {
            return Err(ParseError::UnexpectedToken { expected: "';'", got: self.peek()?.clone() });
        }
        Ok(Statement::Return(expr))
    }
    fn class(&mut self) -> Result<Statement, ParseError> {
        let name: String = match self.advance()? {
            Token::Identifier(i) => i.clone(),
            other => return Err(ParseError::UnexpectedToken { expected: "Identifier", got: other.clone() }),
        };
        let parent: Option<String> = if self.match_advance(&[Token::Colon]) {
            match self.advance()? {
                Token::Identifier(i) => Some(i.clone()),
                other => return Err(ParseError::UnexpectedToken { expected: "Identifier", got: other.clone() }),
            }
        }
        else {
            None
        };
        if !self.match_advance(&[Token::LeftBrace]) {
            return Err(ParseError::UnexpectedToken { expected: "'{'", got: self.peek()?.clone() });
        }

        match self.block()? {
            Statement::Block(b) => Ok(Statement::Class { name, block: b, parent }),
            _ => unreachable!(),
        }
    }
    fn continue_statement(&mut self) -> Result<Statement, ParseError> {
        if !self.match_advance(&[Token::Semicolon]) {
            return Err(ParseError::UnexpectedToken { expected: "';'", got: self.peek()?.clone() });
        }

        Ok(Statement::Continue)
    }
    fn break_statement(&mut self) -> Result<Statement, ParseError> {
        if !self.match_advance(&[Token::Semicolon]) {
            return Err(ParseError::UnexpectedToken { expected: "';'", got: self.peek()?.clone() });
        }

        Ok(Statement::Break)
    }
    fn optional_expression(&mut self) -> Result<Option<Box<Expression>>, ParseError> {
        match self.peek()? {
            Token::Int(_) |
            Token::Float(_) |
            Token::String(_) |
            Token::Null |
            Token::Bool(_) |
            Token::LNot |
            Token::Minus |
            Token::Identifier(_) |
            Token::LeftBracket => Ok(Some(self.expression()?)),
            _ => Ok(None),
        }
    }
    fn expression(&mut self) -> Result<Box<Expression>, ParseError> {
        return self.assignment();
    }
    fn assignment(&mut self) -> Result<Box<Expression>, ParseError> {
        let expr: Box<Expression> = self.logical_or()?;

        if self.match_advance(&[Token::Assign]) {
            let value: Box<Expression> = self.assignment()?;

            match &*expr {
                Expression::Variable(_) | Expression::MemberAccess { .. } => return Ok(Box::new(Expression::Assignment { target: expr, value })),
                _ => return Err(ParseError::UnexpectedAssignmentTarget(expr)),
            }
        }

        Ok(expr)
    }
    fn logical_or(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.logical_and()?;

        while self.match_advance(&[Token::LOr]) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Logical Or", got: token });
                }
            };
            let right: Box<Expression> = self.logical_and()?;
            expr = Box::new(Expression::Binary { 
                left: expr,
                operator: op,
                right: right });
        }

        return Ok(expr);
    }
    fn logical_and(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.equality()?;

        while self.match_advance(&[Token::LAnd]) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Logical And", got: token });
                }
            };
            let right: Box<Expression> = self.equality()?;
            expr = Box::new(Expression::Binary { 
                left: expr,
                operator: op,
                right: right });
        }

        return Ok(expr);
    }
    fn equality(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.inequality()?;

        while self.match_advance(&[Token::Equal, Token::NotEqual]) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Equality", got: token });
                }
            };
            let right: Box<Expression> = self.inequality()?;
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return Ok(expr);
    }
    fn inequality(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.term()?;

        while self.match_advance(&[Token::Less, Token::LessEqual, Token::Greater, Token::GreaterEqual]) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Inequality", got: token });
                }
            };
            let right: Box<Expression> = self.term()?;
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return Ok(expr);
    }
    fn term(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.factor()?;

        while self.match_advance(&[Token::Plus, Token::Minus]) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Term", got: token });
                }
            };
            let right: Box<Expression> = self.factor()?;
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return Ok(expr);
    }
    fn factor(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.unary()?;

        while self.match_advance(&[Token::Asterisk, Token::Slash]) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Factor", got: token });
                }
            };
            let right: Box<Expression> = self.unary()?;
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return Ok(expr);
    }
    fn unary(&mut self) -> Result<Box<Expression>, ParseError> {
        if self.match_advance(&[Token::LNot, Token::Minus]) {
            let op: UnaryOp = match UnaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Unary", got: token });
                }
            };
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
                    if !self.match_advance(&[Token::Semicolon]) {
                        return Err(ParseError::UnexpectedToken { expected: "';'", got: self.peek()?.clone() });
                    }
                }

                expr = Box::new(Expression::Call { callee: expr, parameters });
            }
            else if self.match_advance(&[Token::Dot]) {
                if let Token::Identifier(member) = self.advance()?.clone() {
                    expr = Box::new(Expression::MemberAccess { class: expr, member });
                }
            }
            else {
                break;
            }
        }

        Ok(expr)
    }
    fn primary(&mut self) -> Result<Box<Expression>, ParseError> {
        return match self.advance()? {
            Token::Bool(v) => Ok(Box::new(Expression::Literal(LiteralType::Bool(*v)))),
            Token::Int(v) => Ok(Box::new(Expression::Literal(LiteralType::Int(*v)))),
            Token::Float(v) => Ok(Box::new(Expression::Literal(LiteralType::Float(*v)))),
            Token::String(v) => Ok(Box::new(Expression::Literal(LiteralType::String(v.clone())))),
            Token::Null => Ok(Box::new(Expression::Literal(LiteralType::Null))),
            Token::Identifier(name) => Ok(Box::new(Expression::Variable(name.clone()))),
            Token::LeftBracket => {
                let expr: Box<Expression> = self.expression()?;
                match self.advance() {
                    Ok(t) if t == &Token::RightBracket => return Ok(expr),
                    Ok(t) => return Err(ParseError::UnexpectedToken { expected: ")", got: t.clone() }),
                    Err(e) => Err(e),
                }
            },
            other => Err(ParseError::UnexpectedToken { expected: "Primary", got: other.clone() })
        };
    }
}