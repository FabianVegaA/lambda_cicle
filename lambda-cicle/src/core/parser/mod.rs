pub mod grammar;
pub mod lexer;

use crate::core::ast::{Decl, Term};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] lexer::LexError),
    #[error("Parse error: {0}")]
    SyntaxError(String),
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Expected {expected}, found {found}")]
    ExpectedToken { expected: String, found: String },
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
}

pub fn parse(source: &str) -> Result<Term, ParseError> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = grammar::Parser::new(&tokens);
    parser
        .parse()
        .map_err(|e| ParseError::SyntaxError(e.to_string()))
}

pub fn parse_program(source: &str) -> Result<Vec<Decl>, ParseError> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = grammar::Parser::new(&tokens);
    parser
        .parse_program()
        .map_err(|e| ParseError::SyntaxError(e.to_string()))
}
