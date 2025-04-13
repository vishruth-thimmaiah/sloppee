#![allow(dead_code)]

use errors::{ParserError, Result};
use nodes::ASTNodes;

use lexer::{lexer::Token, types::Types};

mod basics;
mod block;
mod expr;
mod stmt;
mod func;

mod errors;
mod nodes;
#[cfg(test)]
mod test;

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Iterator for Parser {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tokens.len() {
            // Return the token at the current index and increment the index
            let token = self.tokens[self.index].clone();
            self.index += 1;
            Some(token)
        } else {
            // No more tokens to return
            None
        }
    }
}

impl Parser {
    pub fn new(lexer_tokens: Vec<Token>) -> Self {
        Self {
            tokens: lexer_tokens,
            index: 0,
        }
    }

    pub(crate) fn peek(&self) -> Option<Token> {
        self.tokens.get(self.index).cloned()
    }

    pub(crate) fn current(&self) -> Option<Token> {
        self.tokens.get(self.index - 1).cloned()
    }

    pub(crate) fn prev(&mut self) -> Option<Token> {
        self.index -= 1;
        self.current()
    }

    pub(crate) fn next_with_type(&mut self, token_type: Types) -> Result<Token> {
        let token = self.next().unwrap();
        if token.r#type != token_type {
            return Err(ParserError::expected_token_err(token, token_type));
        }
        Ok(token)
    }

    pub(crate) fn current_with_type(&mut self, token_type: Types) -> Result<Token> {
        let token = self.current().unwrap();
        if token.r#type != token_type {
            return Err(ParserError::expected_token_err(token, token_type));
        }
        Ok(token)
    }
    
    pub(crate) fn next_if_type(&mut self, token_type: Types) -> Option<Token> {
        let token = self.peek().unwrap();
        if token.r#type == token_type {
            self.next();
            return Some(token);
        }
        None
    }

    pub(crate) fn peek_with_type(&self, token_type: Types) -> Result<Token> {
        let token = self.peek().unwrap();
        if token.r#type != token_type {
            return Err(ParserError::expected_token_err(token, token_type));
        }
        Ok(token)
    }

    pub fn parse(&mut self) -> Result<Vec<ASTNodes>> {
        self.parse_source()
    }
}
