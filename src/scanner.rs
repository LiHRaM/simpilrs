use std::fmt::{self, Display};

use crate::tokens::{Token, TokenType};
use crate::Result;

/// The Scanner turns a stream of bytes into [`Token`](tokens/struct.Token.html)s.
///
/// Example use:
///
/// ```
/// let input = "goto 1";
/// let tokens: Vec<_> =
///     Scanner::new(input)
///       .collect();
/// ```
/// `goto 1` is the simpIL equivalent of an infinite loop.
/// Here, we eagerly compute the token stream by calling `collect`.
///
/// Otherwise, you can treat Scanner as `impl Iterator<Item = Token>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Scanner {
    source: Vec<u8>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

impl Display for Scanner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let tokens: Vec<_> = self.clone().map(|token| format!("{}", token)).collect();
        write!(f, "{}", tokens.join(","))?;
        write!(f, "]")
    }
}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_at_end() {
            None
        } else {
            match self.scan_token() {
                Ok(t) => Some(t),
                Err(_) => None,
            }
        }
    }
}

impl Scanner {
    /// Construct an instance of the Scanner.
    /// The string reference is turned into bytes internally.
    pub fn new(source: &str) -> Self {
        Self {
            source: source.as_bytes().to_owned(),
            start: 0,
            current: 0,
            line: 1,
            column: 0,
        }
    }

    /// Returns the next token, skipping invalid tokens and whitespace.
    fn scan_token(&mut self) -> Result<Token> {
        loop {
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
                    self.column = 1;
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
                    let nums: u32 = str::parse(nums)?;
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
                            }
                            _ => break,
                        };
                    }
                    let ident = String::from_utf8(ident)?;
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
                _ => TokenType::Invalid(c),
            };

            match token_type {
                TokenType::Ignore => (),
                TokenType::Invalid(c) => crate::report(
                    self.line,
                    self.column,
                    &format!("Invalid Token '{}'", c as char),
                ),
                _ => {
                    return Ok(Token {
                        token_type,
                        lexeme: String::from_utf8(
                            self.source[self.start..self.current].to_owned(),
                        )?,
                        line: self.line,
                    });
                }
            }
        }
    }

    /// True of the current character matches the input.
    /// If true, it advances.
    fn matches(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    /// True if we've reached the end of the source code.
    fn is_at_end(&self) -> bool {
        self.current as usize >= self.source.len()
    }

    /// Returns the next character and increments the counter.
    fn advance(&mut self) -> u8 {
        let char = self.source[self.current];
        self.current += 1;
        self.column += 1;
        char
    }

    /// Returns the value of the next character without advancing.
    fn peek(&self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source[self.current]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: &str) -> String {
        format!("{}", Scanner::new(input))
    }

    #[test]
    fn scan_value() {
        assert_eq!(&tokens("1"), "[Value(1)]")
    }

    #[test]
    fn scan_assignment() {
        assert_eq!(tokens("val := 2"), r#"[Identifier("val"),Assign,Value(2)]"#)
    }
}
