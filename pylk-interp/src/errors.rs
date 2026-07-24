use thiserror::Error;

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
