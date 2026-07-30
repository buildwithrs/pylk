use thiserror::Error;

use crate::lexer::{Token, TokenType};

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("invalid string literal: {0}")]
    InvalidString(String),

    #[error("invalid number literal: {0}")]
    InvalidNumber(String),

    #[error("invalid token: {0}")]
    InvalidToken(String),

    #[error("unsupport token: {0}")]
    UnsupportToken(char),
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("expect token type: {0}, but found: {1}")]
    ExpectTokenType(TokenType, TokenType),

    #[error("program end")]
    EOF,

    #[error("token is not identifier")]
    NotIdent,

    #[error("invalid assign target: {0}")]
    InvalidAssignTarget(Token),

    #[error("no assign target")]
    NoAssignTarget,

    #[error("unsupport: {0}")]
    UnsupportToken(Token),

    #[error("expect token: {0}, but found: {1}")]
    ExpectToken(Token, Token),
}
