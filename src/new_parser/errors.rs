use std::{error::Error, fmt::Display};

use crate::lexer::{lexer::Token, types::Types};

#[derive(Debug)]
pub struct ParserError {
    pub msg: String,
    pub line: usize,
    pub column: usize,
}

impl Error for ParserError {}

impl Default for ParserError {
    fn default() -> Self {
        Self {
            msg: "Unknown error while parsing".to_string(),
            line: 0,
            column: 0,
        }
    }
}

impl ParserError {
    pub fn new(msg: &str, token: Token) -> Self {
        Self {
            msg: msg.to_string(),
            line: token.line,
            column: token.column,
        }
    }

    pub fn expected_token_err(token: Token, expected: Types) -> Self {
        Self::new(&format!("Expected token {:?}, got {:?}", expected, token.r#type), token)
    }

    pub fn unexpected_token_err(token: Token) -> Self {
        Self::new(&format!("Unexpected token {:?}", token.r#type), token)
    }

    pub fn unexpected_eof(token: Token) -> Self {
        Self::new(&format!("Expected token {:?}, got eof", token.r#type), token)
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

pub type Result<T> = std::result::Result<T, ParserError>;
