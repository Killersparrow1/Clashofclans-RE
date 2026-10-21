use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(#[from] coc_protocol::error::ProtocolError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}

pub type Result<T> = std::result::Result<T, ServerError>;