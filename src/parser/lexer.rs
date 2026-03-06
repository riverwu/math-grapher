//! Lexer for mathematical expressions

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Invalid character: {0}")]
    InvalidCharacter(char),

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("Unterminated token at position {0}")]
    UnterminatedToken(usize),
}

/// Tokens produced by the lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    Identifier(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,

    // Comparison (for inequalities)
    Equals,
    NotEquals,
    Less,
    LessEq,
    Greater,
    GreaterEq,

    // Delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Pipe,  // |

    // Special
    Eof,
}

/// Lexer for tokenizing mathematical expressions
pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    current_pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            current_pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                break;
            }
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();

        let Some((pos, ch)) = self.chars.next() else {
            return Ok(Token::Eof);
        };
        self.current_pos = pos;

        match ch {
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Star),
            '/' => Ok(Token::Slash),
            '^' => Ok(Token::Caret),
            '%' => Ok(Token::Percent),
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '[' => Ok(Token::LBracket),
            ']' => Ok(Token::RBracket),
            ',' => Ok(Token::Comma),
            '|' => Ok(Token::Pipe),
            '=' => {
                // Check for ==
                if self.peek_char() == Some('=') {
                    self.chars.next();
                    Ok(Token::Equals)
                } else {
                    Ok(Token::Equals)
                }
            }
            '!' => {
                if self.peek_char() == Some('=') {
                    self.chars.next();
                    Ok(Token::NotEquals)
                } else {
                    Err(LexerError::InvalidCharacter('!'))
                }
            }
            '<' => {
                if self.peek_char() == Some('=') {
                    self.chars.next();
                    Ok(Token::LessEq)
                } else {
                    Ok(Token::Less)
                }
            }
            '>' => {
                if self.peek_char() == Some('=') {
                    self.chars.next();
                    Ok(Token::GreaterEq)
                } else {
                    Ok(Token::Greater)
                }
            }
            '0'..='9' | '.' => self.read_number(ch),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(ch),
            _ => Err(LexerError::InvalidCharacter(ch)),
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn skip_whitespace(&mut self) {
        while let Some((_, ch)) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self, first: char) -> Result<Token, LexerError> {
        let start = self.current_pos;
        let mut has_dot = first == '.';
        let mut has_exp = false;

        while let Some((pos, ch)) = self.chars.peek() {
            match ch {
                '0'..='9' => {
                    self.chars.next();
                }
                '.' if !has_dot && !has_exp => {
                    has_dot = true;
                    self.chars.next();
                }
                'e' | 'E' if !has_exp => {
                    has_exp = true;
                    self.chars.next();
                    // Allow optional sign after exponent
                    if let Some((_, '+' | '-')) = self.chars.peek() {
                        self.chars.next();
                    }
                }
                _ => {
                    self.current_pos = *pos;
                    break;
                }
            }
        }

        let end = self.chars.peek().map(|(pos, _)| *pos).unwrap_or(self.input.len());
        let num_str = &self.input[start..end];

        num_str
            .parse::<f64>()
            .map(Token::Number)
            .map_err(|_| LexerError::InvalidNumber(num_str.to_string()))
    }

    fn read_identifier(&mut self, _first: char) -> Result<Token, LexerError> {
        let start = self.current_pos;

        while let Some((_, ch)) = self.chars.peek() {
            if ch.is_alphanumeric() || *ch == '_' {
                self.chars.next();
            } else {
                break;
            }
        }

        let end = self.chars.peek().map(|(pos, _)| *pos).unwrap_or(self.input.len());
        let ident = &self.input[start..end];

        Ok(Token::Identifier(ident.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        Lexer::new(input).tokenize().unwrap()
    }

    #[test]
    fn test_numbers() {
        assert_eq!(tokenize("42"), vec![Token::Number(42.0)]);
        assert_eq!(tokenize("3.14"), vec![Token::Number(3.14)]);
        assert_eq!(tokenize("1e10"), vec![Token::Number(1e10)]);
        assert_eq!(tokenize("2.5e-3"), vec![Token::Number(2.5e-3)]);
    }

    #[test]
    fn test_identifiers() {
        assert_eq!(tokenize("x"), vec![Token::Identifier("x".to_string())]);
        assert_eq!(tokenize("sin"), vec![Token::Identifier("sin".to_string())]);
        assert_eq!(tokenize("var_1"), vec![Token::Identifier("var_1".to_string())]);
    }

    #[test]
    fn test_operators() {
        assert_eq!(tokenize("+-*/^"), vec![
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::Caret,
        ]);
    }

    #[test]
    fn test_expression() {
        assert_eq!(tokenize("x + 2"), vec![
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Number(2.0),
        ]);
    }

    #[test]
    fn test_function_call() {
        assert_eq!(tokenize("sin(x)"), vec![
            Token::Identifier("sin".to_string()),
            Token::LParen,
            Token::Identifier("x".to_string()),
            Token::RParen,
        ]);
    }

    #[test]
    fn test_comparison() {
        assert_eq!(tokenize("x >= 0"), vec![
            Token::Identifier("x".to_string()),
            Token::GreaterEq,
            Token::Number(0.0),
        ]);
    }
}
