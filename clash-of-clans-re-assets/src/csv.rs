//! CSV Logic File Parsing
//! 
//! Clash of Clans stores game logic data in encrypted CSV files.
//! Format: Custom header + encrypted payload (AES) + optional LZMA compression

use super::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use aes::Aes128;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use cbc::Decryptor;
use crc32fast::Hasher;

/// Magic bytes for encrypted CSV files
pub const CSV_MAGIC: &[u8] = b"SCCV";
/// Alternative magic for newer versions
pub const CSV_MAGIC_V2: &[u8] = b"SCCV2";

/// Encrypted CSV file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub encrypted_size: u32,
    pub decrypted_size: u32,
    pub checksum: u32,
    pub encryption_type: u8,  // 0 = AES, 1 = custom
    pub compression_type: u8, // 0 = none, 1 = LZMA, 2 = zlib
    pub reserved: [u8; 16],
}

/// Parsed CSV table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvTable {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<CsvRow>,
}

/// Single CSV row (column name -> value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvRow {
    pub values: HashMap<String, String>,
}

impl CsvRow {
    pub fn get(&self, column: &str) -> Option<&String> {
        self.values.get(column)
    }
    
    pub fn get_int(&self, column: &str) -> Option<i64> {
        self.get(column)?.parse().ok()
    }
    
    pub fn get_float(&self, column: &str) -> Option<f64> {
        self.get(column)?.parse().ok()
    }
    
    pub fn get_bool(&self, column: &str) -> Option<bool> {
        self.get(column).map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    }
}

/// CSV parser for encrypted logic files
pub struct CsvParser {
    /// AES key for decryption (16 bytes for AES-128)
    aes_key: Option<[u8; 16]>,
    /// AES IV (16 bytes)
    aes_iv: Option<[u8; 16]>,
}

impl CsvParser {
    pub fn new() -> Self {
        Self {
            aes_key: None,
            aes_iv: None,
        }
    }
    
    pub fn with_key(mut self, key: [u8; 16], iv: [u8; 16]) -> Self {
        self.aes_key = Some(key);
        self.aes_iv = Some(iv);
        self
    }
    
    /// Parse encrypted CSV file
    pub fn parse_file(&self, data: &[u8]) -> Result<Vec<CsvTable>> {
        // 1. Read header
        let header = self.read_header(data)?;
        
        // 2. Extract encrypted payload
        let encrypted = &data[std::mem::size_of::<CsvHeader>()..];
        if encrypted.len() != header.encrypted_size as usize {
            return Err(AssetError::InvalidFormat(
                format!("Encrypted size mismatch: expected {}, got {}", 
                    header.encrypted_size, encrypted.len())
            ));
        }
        
        // 3. Decrypt
        let decrypted = self.decrypt_payload(encrypted, &header)?;
        
        // 4. Decompress if needed
        let decompressed = self.decompress(&decrypted, &header)?;
        
        // 5. Verify checksum
        self.verify_checksum(&decompressed, &header)?;
        
        // 6. Parse CSV content
        self.parse_csv_content(&decompressed)
    }
    
    /// Parse decrypted CSV content (plain text CSV)
    fn parse_csv_content(&self, data: &[u8]) -> Result<Vec<CsvTable>> {
        let text = std::str::from_utf8(data)
            .map_err(|e| AssetError::ParseError(e.to_string()))?;
        
        // Split by table separator (usually empty line or specific marker)
        let mut tables = Vec::new();
        let mut current_table: Option<CsvTable> = None;
        
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                if let Some(table) = current_table.take() {
                    if !table.rows.is_empty() {
                        tables.push(table);
                    }
                }
                continue;
            }
            
            // Check if this is a table header (starts with table name)
            if line.starts_with('[') && line.ends_with(']') {
                if let Some(table) = current_table.take() {
                    if !table.rows.is_empty() {
                        tables.push(table);
                    }
                }
                let name = line[1..line.len()-1].to_string();
                current_table = Some(CsvTable {
                    name,
                    columns: Vec::new(),
                    rows: Vec::new(),
                });
                continue;
            }
            
            // Parse CSV line
            if let Some(ref mut table) = current_table {
                if table.columns.is_empty() {
                    // First non-empty line after table name = column headers
                    table.columns = line.split(',').map(|s| s.trim().to_string()).collect();
                } else {
                    // Data row
                    let values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
                    if values.len() == table.columns.len() {
                        let mut row_map = HashMap::new();
                        for (col, val) in table.columns.iter().zip(values.iter()) {
                            row_map.insert(col.clone(), val.clone());
                        }
                        table.rows.push(CsvRow { values: row_map });
                    }
                }
            }
        }
        
        // Don't forget last table
        if let Some(table) = current_table {
            if !table.rows.is_empty() {
                tables.push(table);
            }
        }
        
        Ok(tables)
    }
    
    fn read_header(&self, data: &[u8]) -> Result<CsvHeader> {
        if data.len() < std::mem::size_of::<CsvHeader>() {
            return Err(AssetError::InvalidFormat("File too small for header".into()));
        }
        
        let mut cursor = Cursor::new(&data[..std::mem::size_of::<CsvHeader>()]);
        
        let mut magic = [0u8; 4];
        cursor.copy_to_slice(&mut magic);
        
        if magic != CSV_MAGIC && magic != CSV_MAGIC_V2 {
            return Err(AssetError::InvalidFormat(
                format!("Invalid magic: {:?}", magic)
            ));
        }
        
        let version = cursor.get_u32();
        let encrypted_size = cursor.get_u32();
        let decrypted_size = cursor.get_u32();
        let checksum = cursor.get_u32();
        let encryption_type = cursor.get_u8();
        let compression_type = cursor.get_u8();
        let mut reserved = [0u8; 16];
        cursor.copy_to_slice(&mut reserved);
        
        Ok(CsvHeader {
            magic,
            version,
            encrypted_size,
            decrypted_size,
            checksum,
            encryption_type,
            compression_type,
            reserved,
        })
    }
    
    fn decrypt_payload(&self, encrypted: &[u8], header: &CsvHeader) -> Result<Vec<u8>> {
        match header.encryption_type {
            0 => self.decrypt_aes(encrypted),
            1 => self.decrypt_custom(encrypted),
            _ => Err(AssetError::DecryptionFailed(
                format!("Unknown encryption type: {}", header.encryption_type)
            )),
        }
    }
    
    fn decrypt_aes(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        let key = self.aes_key.ok_or(AssetError::DecryptionFailed("AES key not set".into()))?;
        let iv = self.aes_iv.ok_or(AssetError::DecryptionFailed("AES IV not set".into()))?;
        
        let cipher = Decryptor::<Aes128>::new(&key.into(), &iv.into());
        let mut decrypted = encrypted.to_vec();
        
        // PKCS7 padding removal
        let decrypted = cipher.decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(&mut decrypted)
            .map_err(|e| AssetError::DecryptionFailed(e.to_string()))?;
        
        Ok(decrypted.to_vec())
    }
    
    fn decrypt_custom(&self, _encrypted: &[u8]) -> Result<Vec<u8>> {
        Err(AssetError::DecryptionFailed("Custom encryption not implemented".into()))
    }
    
    fn decompress(&self, data: &[u8], header: &CsvHeader) -> Result<Vec<u8>> {
        match header.compression_type {
            0 => Ok(data.to_vec()), // No compression
            1 => self.decompress_lzma(data), // LZMA
            2 => self.decompress_zlib(data), // zlib
            _ => Err(AssetError::DecompressionFailed(
                format!("Unknown compression type: {}", header.compression_type)
            )),
        }
    }
    
    fn decompress_zlib(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::ZlibDecoder;
        use std::io::Read;
        
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| AssetError::DecompressionFailed(e.to_string()))?;
        Ok(decompressed)
    }
    
    fn decompress_lzma(&self, _data: &[u8]) -> Result<Vec<u8>> {
        Err(AssetError::DecompressionFailed("LZMA not implemented".into()))
    }
    
    fn verify_checksum(&self, data: &[u8], header: &CsvHeader) -> Result<()> {
        let mut hasher = Hasher::new();
        hasher.update(data);
        let checksum = hasher.finalize();
        
        if checksum != header.checksum {
            return Err(AssetError::ChecksumMismatch);
        }
        Ok(())
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to extract specific tables by name
pub fn find_table<'a>(tables: &'a [CsvTable], name: &str) -> Option<&'a CsvTable> {
    tables.iter().find(|t| t.name.eq_ignore_ascii_case(name))
}

/// Common CSV table names in CoC
pub mod table_names {
    pub const BUILDINGS: &str = "buildings";
    pub const CHARACTERS: &str = "characters";
    pub const SPELLS: &str = "spells";
    pub const BUILDING_LEVELS: &str = "building_levels";
    pub const CHARACTER_LEVELS: &str = "character_levels";
    pub const SPELL_LEVELS: &str = "spell_levels";
    pub const TRAPS: &str = "traps";
    pub const DECORATIONS: &str = "decorations";
    pub const OBSTACLES: &str = "obstacles";
    pub const GEMS: &str = "gems";
    pub const ACHIEVEMENTS: &str = "achievements";
    pub const LEAGUES: &str = "leagues";
    pub const ALLIANCE_RANKS: &str = "alliance_ranks";
    pub const ALLIANCE_ROLES: &str = "alliance_roles";
    pub const ALLIANCE_BADGES: &str = "alliance_badges";
    pub const ALLIANCE_LEVELS: &str = "alliance_levels";
    pub const SHOP_OFFERS: &str = "shop_offers";
    pub const RESOURCES: &str = "resources";
    pub const TOWNHALL_LEVELS: &str = "townhall_levels";
    pub const BUILDER_BASE: &str = "builder_base";
    pub const EVENTS: &str = "events";
    pub const SEASONS: &str = "seasons";
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_csv_row_parsing() {
        let mut row = CsvRow { values: HashMap::new() };
        row.values.insert("id".to_string(), "1".to_string());
        row.values.insert("name".to_string(), "Cannon".to_string());
        row.values.insert("enabled".to_string(), "1".to_string());
        
        assert_eq!(row.get_int("id"), Some(1));
        assert_eq!(row.get("name"), Some(&"Cannon".to_string()));
        assert_eq!(row.get_bool("enabled"), Some(true));
    }
}