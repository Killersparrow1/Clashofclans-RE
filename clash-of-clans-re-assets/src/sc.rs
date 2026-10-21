//! SC (Sprite Container) File Parsing
//! 
//! SC files contain game sprites, UI textures, and animations.

use super::*;
use serde::{Deserialize, Serialize};
use bytes::Buf;
use std::io::Cursor;

/// SC file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub texture_count: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub format: u8,      // 0 = RGBA8888, 1 = RGBA4444, 2 = RGB565, etc.
    pub compression: u8, // 0 = none, 1 = zlib, 3 = ETC1, etc.
    pub reserved: [u8; 16],
}

/// Texture entry in SC file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScTexture {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub rotated: bool,
    pub trimmed: bool,
    pub source_size: (u32, u32),
    pub sprite_source_size: (u32, u32, u32, u32), // x, y, w, h
    pub offset: (i32, i32),
    pub format: u8,
    pub data_offset: u64,
    pub data_size: u32,
}

/// SC file container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScFile {
    pub header: ScHeader,
    pub textures: Vec<ScTexture>,
    #[serde(skip)]
    pub texture_data: Vec<u8>,
}

impl ScFile {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < std::mem::size_of::<ScHeader>() {
            return Err(AssetError::InvalidFormat("SC file too small for header".into()));
        }
        
        let mut cursor = Cursor::new(&data[..std::mem::size_of::<ScHeader>()]);
        
        let mut magic = [0u8; 4];
        cursor.copy_to_slice(&mut magic);
        
        if magic != *b"SCF1" && magic != *b"SCF2" {
            return Err(AssetError::InvalidFormat(
                format!("Invalid SC magic: {:?}", magic)
            ));
        }
        
        let version = cursor.get_u32();
        let texture_count = cursor.get_u32();
        let atlas_width = cursor.get_u32();
        let atlas_height = cursor.get_u32();
        let format = cursor.get_u8();
        let compression = cursor.get_u8();
        let mut reserved = [0u8; 16];
        cursor.copy_to_slice(&mut reserved);
        
        let header = ScHeader {
            magic,
            version,
            texture_count,
            atlas_width,
            atlas_height,
            format,
            compression,
            reserved,
        };
        
        // Parse texture entries
        let mut textures = Vec::with_capacity(texture_count as usize);
        let mut offset = std::mem::size_of::<ScHeader>();
        
        for _ in 0..texture_count {
            if offset + 64 > data.len() {
                return Err(AssetError::InvalidFormat("Truncated texture entry".into()));
            }
            
            let entry_buf = &data[offset..offset + 64];
            let mut entry_buf = &entry_buf[..];
            
            // Parse texture entry (simplified format)
            let name_len = entry_buf.get_u16() as usize;
            offset += 2;
            
            let name = if offset + name_len <= data.len() {
                String::from_utf8(data[offset..offset + name_len].to_vec())
                    .map_err(|e| AssetError::ParseError(e.to_string()))?
            } else {
                return Err(AssetError::InvalidFormat("Truncated texture name".into()));
            };
            offset += name_len;
            
            // Skip remaining fixed-size fields for now
            offset += 60; // Remaining fields in 64-byte entry
            
            textures.push(ScTexture {
                name,
                x: 0, y: 0,
                width: 0, height: 0,
                rotated: false,
                trimmed: false,
                source_size: (0, 0),
                sprite_source_size: (0, 0, 0, 0),
                offset: (0, 0),
                format: 0,
                data_offset: 0,
                data_size: 0,
            });
        }
        
        // Remaining data is texture data
        let texture_data = data[offset..].to_vec();
        
        Ok(ScFile {
            header,
            textures,
            texture_data,
        })
    }
    
    /// Get texture by name
    pub fn get_texture(&self, name: &str) -> Option<&ScTexture> {
        self.textures.iter().find(|t| t.name == name)
    }
    
    /// Extract texture data
    pub fn extract_texture(&self, texture: &ScTexture) -> Option<&[u8]> {
        let start = texture.data_offset as usize;
        let end = start + texture.data_size as usize;
        if end <= self.texture_data.len() {
            Some(&self.texture_data[start..end])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sc_header_parsing() {
        // Minimal SC file with header only (big-endian)
        let mut data = Vec::new();
        data.extend_from_slice(b"SCF1");     // magic (4)
        data.extend_from_slice(&1u32.to_be_bytes()); // version (4)
        data.extend_from_slice(&0u32.to_be_bytes()); // texture_count (4)
        data.extend_from_slice(&1024u32.to_be_bytes()); // atlas_width (4)
        data.extend_from_slice(&1024u32.to_be_bytes()); // atlas_height (4)
        data.extend_from_slice(&0u8.to_be_bytes()); // format (1)
        data.extend_from_slice(&0u8.to_be_bytes()); // compression (1)
        data.extend_from_slice(&[0u8; 2]); // padding for alignment (2)
        data.extend_from_slice(&[0u8; 16]); // reserved (16)
        // Total: 4+4+4+4+4+1+1+2+16 = 40 bytes
        
        let sc = ScFile::from_bytes(&data).unwrap();
        assert_eq!(sc.header.version, 1);
        assert_eq!(sc.header.texture_count, 0);
    }
}