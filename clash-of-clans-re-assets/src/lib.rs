//! Clash of Clans Asset Decryption and Parsing
//! 
//! Handles decryption and parsing of game assets:
//! - CSV logic files (buildings, characters, spells, etc.)
//! - SC sprite files (UI, textures)
//! - Fingerprint.json for version verification

pub mod csv;
pub mod fingerprint;
pub mod sc;

pub use csv::{CsvParser, CsvRow, CsvTable};
pub use fingerprint::{Fingerprint, AssetEntry};
pub use sc::ScFile;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
    #[error("Checksum mismatch")]
    ChecksumMismatch,
}

pub type Result<T> = std::result::Result<T, AssetError>;