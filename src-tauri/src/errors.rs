use serde::Serialize;
use thiserror::Error;

/// Unified error type for the application
#[derive(Debug, Error, Serialize)]
pub enum EmojiError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("JSON error: {0}")]
    Json(String),

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Permission error: {0}")]
    Permission(String),

    #[error("Positioning error: {0}")]
    Positioning(String),

    #[error("Window handle error")]
    WindowHandle,

    #[error("Monitor not found")]
    MonitorNotFound,

    #[error("Panel error: {0}")]
    Panel(String),

    #[error("Emoji not found: {0}")]
    EmojiNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Tauri error: {0}")]
    Tauri(String),
}

impl From<EmojiError> for String {
    fn from(error: EmojiError) -> Self {
        error.to_string()
    }
}

impl From<std::io::Error> for EmojiError {
    fn from(error: std::io::Error) -> Self {
        EmojiError::Io(error.to_string())
    }
}

impl From<serde_json::Error> for EmojiError {
    fn from(error: serde_json::Error) -> Self {
        EmojiError::Json(error.to_string())
    }
}

// Helper trait for converting lock errors
pub trait LockResultExt<T> {
    fn map_lock_err(self) -> Result<T, EmojiError>;
}

impl<T> LockResultExt<T> for Result<T, std::sync::PoisonError<T>> {
    fn map_lock_err(self) -> Result<T, EmojiError> {
        self.map_err(|e| EmojiError::Lock(e.to_string()))
    }
}

impl<T, U> LockResultExt<T> for Result<T, std::sync::TryLockError<U>> {
    fn map_lock_err(self) -> Result<T, EmojiError> {
        self.map_err(|e| EmojiError::Lock(e.to_string()))
    }
}
