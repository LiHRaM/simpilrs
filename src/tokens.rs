use std::fmt::{self, Display};

/// The TokenType encapsulates most information about a Token.
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    /// Tokens which are not recognized by the Scanner.
    Invalid(u8),
    /// Tokens such as whitespace, which are recognized but syntactically unimportant.
    Ignore,

    /// Left parenthesis.
    LeftParen,
    /// Right parenthesis.
    RightParen,
    /// Comma.
    Comma,
    /// Plus, the addition operator.
    Plus,
    /// Minus, the subtraction operator.
    Minus,
    /// Star, the multiplication operator.
    Star,
    /// Slash, the division operator.
    Slash,

    /// Assignment, i.e. `:=`.
    Assign,

    /// A 32-bit unsigned integer.
    Value(u32),
    /// A string identifier.
    Identifier(String),

    /// A statement keyword for storing a value in a registry.
    Store,
    /// A statement keyword for moving execution to a given statement.
    Goto,
    /// A statement keyword for asserting a condition.
    Assert,
    /// The first statement keyword for conditional evaluation.
    If,
    /// The second statement keyword for conditional evaluation.
    Then,
    /// The third statement keyword for conditional evaluation.
    Else,
    /// An expression keyword for loading the value from a registry.
    Load,
    /// A statement keyword for getting input from an external source, such as `stdin`.
    GetInput,
}

/// A wrapper for TokenType, including also the lexeme and line placement.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.token_type)
    }
}
