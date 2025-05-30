// src/error.rs
use thiserror::Error;
use uniffi;

/// Custom error types for the emoji search engine (internal use)
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

/// Error type exposed through FFI
/// This implements all the necessary UniFFI traits
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    #[error("IO error")]
    Io,
    #[error("JSON parsing error")]
    Json,
    #[error("Search error: {0}")]
    Search(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// Convert from internal error to FFI error
impl From<EmojiSearchError> for FfiError {
    fn from(err: EmojiSearchError) -> Self {
        match err {
            EmojiSearchError::Io(_) => FfiError::Io,
            EmojiSearchError::Json(_) => FfiError::Json,
            EmojiSearchError::Search(msg) => FfiError::Search(msg),
            EmojiSearchError::InvalidInput(msg) => FfiError::InvalidInput(msg),
        }
    }
}

/// Type alias for internal Results
pub type Result<T> = std::result::Result<T, EmojiSearchError>;

/// Type alias for FFI Results that will cross language boundaries
pub type FfiResult<T> = std::result::Result<T, FfiError>;

/// Helper trait to convert internal results to FFI results
pub trait IntoFfiResult<T> {
    fn into_ffi_result(self) -> FfiResult<T>;
}

impl<T> IntoFfiResult<T> for Result<T> {
    fn into_ffi_result(self) -> FfiResult<T> {
        self.map_err(Into::into)
    }
}

impl From<std::io::Error> for FfiError {
    fn from(_: std::io::Error) -> Self {
        FfiError::Io
    }
}

impl From<serde_json::Error> for FfiError {
    fn from(_: serde_json::Error) -> Self {
        FfiError::Json
    }
}
