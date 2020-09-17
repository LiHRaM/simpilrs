#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Other lexemes
    Invalid(u8),
    Ignore,

    // Single-character tokens.
    LeftParen, RightParen, Comma,
    Plus, Minus, Star, Slash,

    // One or two character tokens.
    Assign,

    // Literals.
    Value(u32), Identifier(String),

    // Keywords.
    Store, Goto,
    Assert, If, Then,
    Else, Load, GetInput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize
}