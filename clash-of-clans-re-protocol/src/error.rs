use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid packet: {0}")]
    InvalidPacket(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Invalid message ID: {0}")]
    InvalidMessageId(u16),

    #[error("Buffer too small: expected {expected}, got {actual}")]
    BufferTooSmall { expected: usize, actual: usize },

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Handshake error: {0}")]
    HandshakeError(String),

    #[error("Nonce error: {0}")]
    NonceError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;