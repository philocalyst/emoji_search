// src/error.rs
use thiserror::Error;

/// Custom error types for the emoji search engine
#[derive(Error, Debug)]
pub enum EmojiSearchError {
    /// IO errors when reading data files
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing errors
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Search-related errors
    #[error("Search error: {0}")]
    Search(String),

    /// Invalid input errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Type alias for Result with EmojiSearchError
pub type Result<T> = std::result::Result<T, EmojiSearchError>;
