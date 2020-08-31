#[derive(Debug)]
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
    Value(i32), Identifier(String),

    // Keywords.
    Store, Goto,
    Assert, If, Then,
    Else, Load, GetInput,

    Eof // End of file
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}

