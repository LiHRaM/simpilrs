use crate::tokens::{Token, TokenType};
use crate::Result;
use crate::{
    scanner::Scanner,
    syntax::{Expr, Stmt},
};
use std::{
    fmt::Display,
    iter::{Iterator, Peekable},
};
use thiserror::Error;
use tracing::{event, Level};

/// An enum used for error reporting.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// A statement is somehow invalid.
    #[error("Parsing statement failed.")]
    Stmt(&'static str),

    /// An expression is somehow invalid.
    #[error("Parsing expression failed.")]
    Expr(&'static str),

    /// A different token was expected.
    ///
    /// Note: typically several tokens are expected,
    /// we just use the Expr and Stmt error types for those.
    /// There is probably a better solution.
    #[error("Expected different token type.")]
    Expected(TokenType),
}

#[doc(hidden)]
fn err_expr<T>(msg: &'static str) -> Result<T> {
    Err(Box::new(ParseError::Expr(msg)))
}

#[doc(hidden)]
fn err_stmt<T>(msg: &'static str) -> Result<T> {
    Err(Box::new(ParseError::Stmt(msg)))
}

#[doc(hidden)]
fn err_expected<T>(expected: TokenType) -> Result<T> {
    Err(Box::new(ParseError::Expected(expected)))
}

#[doc(hidden)]
static BINARY_OPS: [TokenType; 4] = [
    TokenType::Plus,
    TokenType::Minus,
    TokenType::Star,
    TokenType::Slash,
];

/// Parser consumes a Scanner, turning the Tokens into a Syntax Tree.
/// The Parser can in turn be consumed by an Interpreter.
#[derive(Debug, Clone)]
pub struct Parser {
    scanner: Peekable<Scanner>,
}

impl Display for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let stmts: Vec<_> = self.clone().map(|stmt| format!("{}", stmt)).collect();
        write!(f, "{}", stmts.join(","))?;
        write!(f, "]")
    }
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
    /// Create a new parser from a Scanner, i.e. a stream of Tokens.
    pub fn new(scanner: Scanner) -> Self {
        Self {
            scanner: scanner.peekable(),
        }
    }

    /// Once parsing has failed, try to advance to the next statement.
    fn synchronize(&mut self) {
        event!(Level::INFO, "call synchronize");
        self.scanner.next();
        while !self.is_at_end() {
            if let Some(token) = self.scanner.peek() {
                match token.token_type {
                    TokenType::Assign
                    | TokenType::Store
                    | TokenType::Goto
                    | TokenType::Assert
                    | TokenType::If => return,
                    _ => {
                        self.scanner.next();
                    }
                }
            }
        }
    }

    /// Attempts to parse a statement.
    fn statement(&mut self) -> Result<Stmt> {
        let lhs = match self.scanner.next() {
            Some(token) => token,
            None => return err_stmt("Expected token, found EOF."),
        };

        match lhs.token_type {
            TokenType::Identifier(_) => self.assign(lhs),
            TokenType::Store => self.store(),
            TokenType::Goto => self.goto(),
            TokenType::Assert => self.assert(),
            TokenType::If => self.r#if(),
            _ => return err_stmt("Expected statement."),
        }
    }

    /// Attempt to parse an expression.
    fn expression(&mut self) -> Result<Expr> {
        let lhs = match self.scanner.peek() {
            Some(token) => token,
            None => return err_expr("Expected token, found EOF."),
        };

        match lhs.token_type {
            TokenType::Load => self.load(),
            TokenType::GetInput => Ok(Expr::GetInput("stdin".into())),
            TokenType::Identifier(_) | TokenType::Value(_) => self.ops(0),
            TokenType::Plus | TokenType::Minus => self.unary(),
            _ => return err_expr("Expected Load, GetInput, Identifier or Value."),
        }
    }

    fn binary_binding_power(token_type: &TokenType) -> Result<(u8, u8)> {
        let res = match token_type {
            TokenType::Plus | TokenType::Minus => (1, 2),
            TokenType::Star | TokenType::Slash => (3, 4),
            _ => return err_expr("Expected operator."),
        };
        Ok(res)
    }

    /// Attempt to parse a unary expression.
    /// TODO: Not really sure what we want here, to be honest.
    fn unary(&mut self) -> Result<Expr> {
        Ok(Expr::Unary(
            self.scanner.next().unwrap(),
            Box::new(self.expression()?),
        ))
    }

    /// Attempt to parse a series of operations.
    /// Use Pratt parsing as described in
    /// [SPPP](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html)
    /// to get the correct precedence and associativity.
    fn ops(&mut self, min_binding_power: u8) -> Result<Expr> {
        let mut lhs = {
            let parse_err: Result<Expr> = err_expr("Expected value or identifier.");
            match self.scanner.next() {
                Some(ref token) => match &token.token_type {
                    TokenType::Value(val) => Expr::Val(*val),
                    TokenType::Identifier(var) => Expr::Var(var.clone()),
                    _ => return parse_err,
                },
                None => return parse_err,
            }
        };
        loop {
            let op = match self.scanner.peek() {
                Some(op) => op.clone(),
                None => break,
            };

            if BINARY_OPS.contains(&op.token_type) {
                let (left_binding_power, right_binding_power) =
                    Self::binary_binding_power(&op.token_type)?;
                if left_binding_power < min_binding_power {
                    break;
                }

                self.scanner.next().unwrap();
                let rhs = self.ops(right_binding_power)?;

                lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    /// Attempt to parse the load expression.
    fn load(&mut self) -> Result<Expr> {
        self.scanner.next().unwrap();
        self.expect(TokenType::LeftParen)?;
        let inner = self.expression()?;
        self.expect(TokenType::RightParen)?;
        Ok(Expr::Load(Box::new(inner)))
    }

    /// Attempt to parse the assignment statement.
    fn assign(&mut self, identifier: Token) -> Result<Stmt> {
        let assign = self.scanner.next().unwrap();
        if assign.token_type == TokenType::Assign {
            let expr = self.expression()?;
            Ok(Stmt::Assignment(identifier, Box::new(expr)))
        } else {
            err_stmt("Invalid assignment.".into())
        }
    }

    /// Attempt to parse the store statement.
    fn store(&mut self) -> Result<Stmt> {
        self.expect(TokenType::LeftParen)?;
        let left = self.expression()?;
        self.expect(TokenType::Comma)?;
        let right = self.expression()?;
        self.expect(TokenType::RightParen)?;
        Ok(Stmt::Store(Box::new(left), Box::new(right)))
    }

    /// Attempt to parse the goto statement.
    fn goto(&mut self) -> Result<Stmt> {
        Ok(Stmt::Goto(Box::new(self.expression()?)))
    }

    /// Attempt to parse the assert statement.
    fn assert(&mut self) -> Result<Stmt> {
        Ok(Stmt::Assert(Box::new(self.expression()?)))
    }

    /// Attempt to parse the IfThenElse statement.
    fn r#if(&mut self) -> Result<Stmt> {
        let condition = self.expression()?;
        self.expect(TokenType::Then)?;
        self.expect(TokenType::Goto)?;
        let first = self.expression()?;
        self.expect(TokenType::Else)?;
        self.expect(TokenType::Goto)?;
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
            match self.scanner.peek() {
                Some(t) => t.token_type == token_type,
                None => false,
            }
        }
    }

    /// Expect the next token type to match `token_type`, throw an error if not.
    fn expect(&mut self, token_type: TokenType) -> Result<()> {
        event!(Level::INFO, "call expect");
        if !self.check(token_type.clone()) {
            err_expected(token_type)
        } else {
            self.scanner.next();
            Ok(())
        }
    }

    /// True if the stream has run dry.
    fn is_at_end(&mut self) -> bool {
        event!(Level::INFO, "call is_at_end");
        match self.scanner.peek() {
            Some(_) => false,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    fn statement(src: &str) -> String {
        format!("{}", Parser::new(Scanner::new(src)).statement().unwrap())
    }

    fn expression(src: &str) -> String {
        format!("{}", Parser::new(Scanner::new(src)).expression().unwrap())
    }

    #[test]
    fn parse_addition() {
        expression("1 + 1");
    }

    #[test]
    fn parse_subtraction() {
        expression("1 - 1");
    }

    #[test]
    fn parse_division() {
        expression("1 / 1");
    }

    #[test]
    fn parse_multiplication() {
        expression("1 * 1");
    }

    #[test]
    fn parse_unary_minus() {
        expression("-1");
    }

    #[test]
    fn parse_unary_plus() {
        expression("+1");
    }

    #[test]
    fn parse_assignment() {
        statement("x := 1");
    }

    #[test]
    fn parse_store() {
        statement("store(1, 1)");
    }

    #[test]
    fn parse_goto() {
        statement("goto 1");
    }

    #[test]
    fn parse_assert() {
        statement("assert 1");
    }

    #[test]
    fn parse_if_then_else() {
        statement("if 1 then goto 2 else goto 3");
    }

    #[test]
    fn parse_load() {
        statement("goto load(1)");
    }

    #[test]
    fn parse_get_input() {
        statement("goto get_input(stdout)");
    }

    #[test]
    fn parse_precedence_1() {
        assert_eq!(expression("1 * 1 + 1"), "((1, Star, 1), Plus, 1)");
    }

    #[test]
    fn parse_precedence_2() {
        assert_eq!(expression("1 + 1 * 1"), "(1, Plus, (1, Star, 1))");
    }
}
