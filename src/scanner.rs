use crate::tokens::{Token, TokenType};
use crate::Result;

pub struct Scanner {
    source: Vec<u8>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source: source.as_bytes().to_owned(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), self.line));
        Ok(self.tokens)
    }

    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();
        let token_type = match c {
            b'(' => TokenType::LeftParen,
            b')' => TokenType::RightParen,
            b',' => TokenType::Comma,
            b'+' => TokenType::Plus,
            b'-' => TokenType::Minus,
            b'*' => TokenType::Star,
            b'/' => TokenType::Slash,
            b':' => {
                if self.matches(b'=') {
                    TokenType::Assign
                } else {
                    TokenType::Invalid(c)
                }
            }
            b' ' | b'\r' | b'\t' => TokenType::Ignore,
            b'\n' => {
                self.line += 1;
                TokenType::Ignore
            }
            b'0'..=b'9' => {
                let mut nums = vec![c];
                loop {
                    let next = self.peek();
                    match next {
                        b'0'..=b'9' => {
                            nums.push(next);
                            self.advance();
                        }
                        _ => break,
                    }
                }
                let nums: &str = &String::from_utf8(nums)?;
                let nums: i32 = str::parse(nums)?;
                TokenType::Value(nums)
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let mut ident = vec![c];
                loop {
                    let next = self.peek();
                    match next {
                        b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' => {
                            ident.push(next);
                            self.advance();
                        },
                        _ => break,
                    };
                }
                let ident= String::from_utf8(ident)?;
                match ident.as_ref() {
                    "store" => TokenType::Store,
                    "goto" => TokenType::Goto,
                    "assert" => TokenType::Assert,
                    "if" => TokenType::If,
                    "then" => TokenType::Then,
                    "else" => TokenType::Else,
                    "load" => TokenType::Load,
                    "get_input" => TokenType::GetInput,
                    _ => TokenType::Identifier(ident),
                }
            }
            _ => {
                crate::error(self.line, "Unexpected character.");
                TokenType::Invalid(c)
            }
        };
        self.add_token(token_type)?;
        Ok(())
    }

    /// Checks if expected matches the current character.
    /// Advances if it does.
    fn matches(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    /// True of we've reached the end of the source.
    fn is_at_end(&self) -> bool {
        self.current as usize >= self.source.len()
    }

    /// Returns the current character and increases the count by one.
    fn advance(&mut self) -> u8 {
        let char = self.source[self.current];
        self.current += 1;
        char
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source[self.current]
        }
    }

    /// Adds a given token to the token vector.
    fn add_token(&mut self, token_type: TokenType) -> Result<()> {
        match token_type {
            TokenType::Ignore => (),
            _ => {
                self.tokens.push(Token {
                    token_type,
                    lexeme: String::from_utf8(self.source[self.start..self.current].to_owned())?,
                    line: self.line,
                });
            }
        }

        Ok(())
    }
}
