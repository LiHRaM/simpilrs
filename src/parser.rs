use crate::syntax::{Expr, Stmt};
use crate::tokens::{Token, TokenType};
use crate::Result;
use thiserror::Error;
use tracing::{event, Level};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parsing statement failed.")]
    Stmt(String),
    #[error("Parsing expression failed.")]
    Expr(String),
    #[error("Expected different token type.")]
    Expected(TokenType),
}

static BINARY_OPS: [TokenType; 4] = [
    TokenType::Plus,
    TokenType::Minus,
    TokenType::Star,
    TokenType::Slash,
];

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(self) -> Vec<Stmt> {
        self.program()
    }

    fn program(mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();
        loop {
            if let Ok(stmt) = self.statement() {
                statements.push(stmt);
            } else if self.is_at_end() {
                break;
            } else {
                self.synchronize();
            }
        }
        statements
    }

    /// Once parsing has failed, try to advance to the next statement.
    fn synchronize(&mut self) {
        event!(Level::INFO, "call synchronize");
        self.advance();
        while !self.is_at_end() {
            match self.peek().token_type {
                TokenType::Assign
                | TokenType::Store
                | TokenType::Goto
                | TokenType::Assert
                | TokenType::If
                | TokenType::Eof => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Attempts to parse a statement in Parser's current state.
    fn statement(&mut self) -> Result<Stmt> {
        let token = self.advance();
        match &token.token_type {
            TokenType::Identifier(_) => self.assign(),
            TokenType::Store => self.store(),
            TokenType::Goto => self.goto(),
            TokenType::Assert => self.assert(),
            TokenType::If => self.r#if(),
            TokenType::Eof => Err(Box::new(ParseError::Stmt(
                "Unexpected end of file reached.".into(),
            ))),
            t => Err(Box::new(ParseError::Stmt(format!(
                "Found {:?}, expected statement.",
                t
            )))),
        }
    }

    fn expression(&mut self) -> Result<Expr> {
        let token = self.advance();
        match token.token_type {
            TokenType::Load => self.load(),
            TokenType::GetInput => Ok(Expr::GetInput("stdin".into())),
            TokenType::Identifier(i) => self.ident(i),
            TokenType::Value(v) => self.val(v),
            TokenType::Minus => self.unary(),
            t => Err(Box::new(ParseError::Expr(format!(
                "Found {:?}, expected expression.",
                t
            )))),
        }
    }

    fn unary(&mut self) -> Result<Expr> {
        Ok(Expr::Unary(self.previous(), Box::new(self.expression()?)))
    }

    fn val(&mut self, val: u32) -> Result<Expr> {
        let left = Expr::Val(val);
        if self.matches(BINARY_OPS.to_vec()) {
            let operator = self.advance();
            Ok(Expr::Binary(
                Box::new(left),
                operator,
                Box::new(self.expression()?),
            ))
        } else {
            Ok(left)
        }
    }

    fn ident(&mut self, ident: String) -> Result<Expr> {
        let left = Expr::Var(ident);
        if self.matches(BINARY_OPS.to_vec()) {
            let operator = self.advance();
            Ok(Expr::Binary(
                Box::new(left),
                operator,
                Box::new(self.expression()?),
            ))
        } else {
            Ok(left)
        }
    }

    fn matches(&mut self, token_types: Vec<TokenType>) -> bool {
        event!(Level::INFO, "call matches");
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn load(&mut self) -> Result<Expr> {
        self.expect(TokenType::LeftParen)?;
        let inner = self.expression()?;
        self.expect(TokenType::RightParen)?;
        Ok(Expr::Load(Box::new(inner)))
    }

    fn assign(&mut self) -> Result<Stmt> {
        let identifier = self.previous();
        let assign = self.advance();
        if assign.token_type == TokenType::Assign {
            let expr = self.expression()?;
            Ok(Stmt::Assignment(identifier, Box::new(expr)))
        } else {
            Err(Box::new(ParseError::Stmt("Invalid assignment.".into())))
        }
    }

    fn store(&mut self) -> Result<Stmt> {
        self.expect(TokenType::LeftParen)?;
        let left = self.expression()?;
        self.expect(TokenType::Comma)?;
        let right = self.expression()?;
        self.expect(TokenType::RightParen)?;
        Ok(Stmt::Store(Box::new(left), Box::new(right)))
    }

    fn goto(&mut self) -> Result<Stmt> {
        Ok(Stmt::Goto(Box::new(self.expression()?)))
    }

    fn assert(&mut self) -> Result<Stmt> {
        Ok(Stmt::Assert(Box::new(self.expression()?)))
    }

    fn r#if(&mut self) -> Result<Stmt> {
        let condition = self.expression()?;
        self.expect(TokenType::Then)?;
        let first = self.expression()?;
        self.expect(TokenType::Else)?;
        let second = self.expression()?;
        Ok(Stmt::IfThenElse(
            Box::new(condition),
            Box::new(first),
            Box::new(second),
        ))
    }

    /// True if the next token matches token_type.
    fn check(&mut self, token_type: TokenType) -> bool {
        event!(Level::INFO, "call check");
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn expect(&mut self, token_type: TokenType) -> Result<()> {
        event!(Level::INFO, "call expect");
        if !self.check(token_type.clone()) {
            Err(Box::new(ParseError::Expected(token_type)))
        } else {
            self.advance();
            Ok(())
        }
    }

    fn advance(&mut self) -> Token {
        event!(Level::INFO, "call advance");
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        event!(Level::INFO, "call is_at_end");
        match self.peek().token_type {
            TokenType::Eof => true,
            _ => false,
        }
    }

    fn peek(&self) -> Token {
        event!(Level::INFO, "call peek");
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        event!(Level::INFO, "call previous");
        self.tokens[self.current - 1].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    fn statements(src: &str) -> Vec<Stmt> {
        let tokens = Scanner::new(src.into()).scan_tokens().unwrap();
        Parser::new(tokens).parse()
    }

    #[test]
    fn test_assignment_parses() {
        assert_eq!(statements("x := 1").len(), 1);
    }

    #[test]
    fn test_assignment_structure() {
        for st in statements("x := 1") {
            assert!(match st {
                Stmt::Assignment(_, _) => true,
                _ => false,
            })
        }
    }

    #[test]
    fn test_store_parses() {
        let statements = statements("store(1, 1)");
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn test_goto_parses() {
        let statements = statements("goto 1");
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn test_assert_parses() {
        let statements = statements("assert 1");
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn test_if_then_else_parses() {
        let statements = statements("if 1 then 2 else 3");
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn test_load_parses() {
        let statements = statements("goto load(1)");
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn test_load_structure() {
        let statements = statements("goto load(1)");
        for st in statements {
            assert!(match st {
                Stmt::Goto(expr) => match expr.as_ref() {
                    Expr::Load(_) => true,
                    _ => false,
                },
                _ => false,
            })
        }
    }

    #[test]
    fn test_binary_parses() {
        let statements = statements("goto 1 + 1");
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn test_binary_structure() {
        let statements = statements("goto 1 + 1");
        for st in statements {
            println!("{:#?}", st);
            assert!(match st {
                Stmt::Goto(expr) => match expr.as_ref() {
                    Expr::Binary(_, _, _) => true,
                    _ => false,
                },
                _ => false,
            })
        }
    }

    #[test]
    fn test_unary_parses() {
        let statements = statements("goto -1");
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn test_unary_structure() {
        let statements = statements("goto -1");
        for st in statements {
            assert!(match st {
                Stmt::Goto(expr) => match expr.as_ref() {
                    Expr::Unary(_, _) => true,
                    _ => false,
                },
                _ => false,
            })
        }
    }

    #[test]
    fn test_get_input_parses() {
        let statements = statements("goto get_input(stdout)");
        assert_eq!(statements.len(), 1);
    }
}
