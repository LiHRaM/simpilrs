use crate::tokens::{Token, TokenType};
use crate::Result;
use crate::{
    scanner::Scanner,
    syntax::{Expr, Stmt},
};
use std::iter::{Iterator, Peekable};
use thiserror::Error;
use tracing::{event, Level};

#[derive(Error, Debug, Clone, PartialEq)]
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
    scanner: Peekable<Scanner>,
}

impl Iterator for Parser {
    type Item = Stmt;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(stmt) = self.statement() {
            Some(stmt)
        } else if self.is_at_end() {
            None
        } else {
            self.synchronize();
            self.next()
        }
    }
}

impl Parser {
    pub fn new(scanner: Scanner) -> Self {
        Self {
            scanner: scanner.peekable(),
        }
    }

    /// Once parsing has failed, try to advance to the next statement.
    fn synchronize(&mut self) {
        event!(Level::INFO, "call synchronize");
        self.advance();
        while !self.is_at_end() {
            if let Some(token) = self.peek_token() {
                match token.token_type {
                    TokenType::Assign
                    | TokenType::Store
                    | TokenType::Goto
                    | TokenType::Assert
                    | TokenType::If => return,
                    _ => {
                        self.advance();
                    }
                }
            }
        }
    }

    /// Attempts to parse a statement in Parser's current state.
    fn statement(&mut self) -> Result<Stmt> {
        let token = self.advance();
        match token {
            Some(token) => match token.token_type {
                TokenType::Identifier(_) => self.assign(),
                TokenType::Store => self.store(),
                TokenType::Goto => self.goto(),
                TokenType::Assert => self.assert(),
                TokenType::If => self.r#if(),
                token_type => Err(Box::new(ParseError::Stmt(format!(
                    "Found {:?}, expected statement.",
                    token_type
                )))),
            },
            None => Err(Box::new(ParseError::Stmt(
                "Unexpected end of file reached.".into(),
            ))),
        }
    }

    fn expression(&mut self) -> Result<Expr> {
        let token = self.advance();
        match token {
            Some(token) => match token.token_type {
                TokenType::Load => self.load(),
                TokenType::GetInput => Ok(Expr::GetInput("stdin".into())),
                TokenType::Identifier(i) => self.ident(i),
                TokenType::Value(v) => self.val(v),
                TokenType::Minus => self.unary(),
                t => Err(Box::new(ParseError::Expr(format!(
                    "Found {:?}, expected expression.",
                    t
                )))),
            },
            None => Err(Box::new(ParseError::Expr(
                "Unexpected end of file reached.".into(),
            ))),
        }
    }

    fn unary(&mut self) -> Result<Expr> {
        Ok(Expr::Unary(
            self.next_token().unwrap(),
            Box::new(self.expression()?),
        ))
    }

    fn val(&mut self, val: u32) -> Result<Expr> {
        let left = Expr::Val(val);
        if self.matches(BINARY_OPS.to_vec()) {
            let operator = self.advance().unwrap();
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
            let operator = self.advance().unwrap();
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
        let identifier = self.next_token().unwrap();
        let assign = self.advance().unwrap();
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
            match self.peek_token() {
                Some(t) => t.token_type == token_type,
                None => false,
            }
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

    fn advance(&mut self) -> Option<Token> {
        event!(Level::INFO, "call advance");
        self.next_token()
    }

    fn is_at_end(&mut self) -> bool {
        event!(Level::INFO, "call is_at_end");
        match self.peek_token() {
            Some(_) => false,
            None => true,
        }
    }

    fn peek_token(&mut self) -> Option<&Token> {
        event!(Level::INFO, "call peek");
        self.scanner.peek()
    }

    fn next_token(&mut self) -> Option<Token> {
        event!(Level::INFO, "call previous");
        self.scanner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    fn statement(src: &str) -> Result<Stmt> {
        Parser::new(Scanner::new(src)).statement()
    }

    fn expression(src: &str) -> Result<Expr> {
        Parser::new(Scanner::new(src)).expression()
    }

    #[test]
    fn parse_addition() {
        expression("1 + 1").unwrap();
    }

    #[test]
    fn parse_subtraction() {
        expression("1 - 1").unwrap();
    }

    #[test]
    fn parse_division() {
        expression("1 / 1").unwrap();
    }

    #[test]
    fn parse_multiplication() {
        expression("1 * 1").unwrap();
    }

    #[test]
    fn parse_unary_minus() {
        expression("-1").unwrap();
    }

    #[test]
    fn parse_unary_plus() {
        expression("+1").unwrap();
    }

    #[test]
    fn parse_assignment() {
        assert!(match statement("x := 1").unwrap() {
            Stmt::Assignment(_, _) => true,
            _ => false,
        })
    }

    #[test]
    fn parse_store() {
        statement("store(1, 1)").unwrap();
    }

    #[test]
    fn parse_goto() {
        statement("goto 1").unwrap();
    }

    #[test]
    fn parse_assert() {
        statement("assert 1").unwrap();
    }

    #[test]
    fn parse_if_then_else() {
        statement("if 1 then 2 else 3").unwrap();
    }

    #[test]
    fn parse_load() {
        statement("goto load(1)").unwrap();
    }

    #[test]
    fn parse_get_input() {
        statement("goto get_input(stdout)").unwrap();
    }
}
