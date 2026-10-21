//! Fingerprint.json Parsing
//! 
//! Fingerprint.json contains version info and asset checksums for the client.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crc32fast::Hasher;

/// Root fingerprint structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fingerprint {
    pub version: String,
    pub build: String,
    pub assets: Vec<AssetEntry>,
    pub sha: String,
    pub content_version: Option<String>,
}

/// Individual asset entry with checksum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    pub name: String,
    pub sha: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub compressed: Option<bool>,
}

impl Fingerprint {
    /// Parse fingerprint.json from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(data)
            .map_err(|e| AssetError::ParseError(e.to_string()))?;
        
        serde_json::from_str(text)
            .map_err(|e| AssetError::Json(e))
    }
    
    /// Find asset by name
    pub fn find_asset(&self, name: &str) -> Option<&AssetEntry> {
        self.assets.iter().find(|a| a.name == name)
    }
    
    /// Get all assets of a specific type
    pub fn assets_by_type(&self, asset_type: &str) -> Vec<&AssetEntry> {
        self.assets.iter()
            .filter(|a| a.asset_type == asset_type)
            .collect()
    }
    
    /// Verify asset checksum
    pub fn verify_asset(&self, name: &str, data: &[u8]) -> Result<bool> {
        let entry = self.find_asset(name)
            .ok_or_else(|| AssetError::ParseError(format!("Asset not found: {}", name)))?;
        
        let mut hasher = Hasher::new();
        hasher.update(data);
        let checksum = format!("{:08x}", hasher.finalize());
        
        Ok(checksum == entry.sha)
    }
    
    /// Get total size of all assets
    pub fn total_size(&self) -> u64 {
        self.assets.iter().map(|a| a.size).sum()
    }
}

/// Fingerprint cache for quick lookups
#[derive(Debug, Default)]
pub struct FingerprintCache {
    entries: HashMap<String, AssetEntry>,
}

impl FingerprintCache {
    pub fn from_fingerprint(fp: &Fingerprint) -> Self {
        let mut entries = HashMap::new();
        for asset in &fp.assets {
            entries.insert(asset.name.clone(), asset.clone());
        }
        Self { entries }
    }
    
    pub fn get(&self, name: &str) -> Option<&AssetEntry> {
        self.entries.get(name)
    }
    
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fingerprint_parsing() {
        let json = r#"{
            "version": "1.0",
            "build": "12345",
            "sha": "abc123",
            "assets": [
                {
                    "name": "buildings.csv",
                    "sha": "abcdef1234567890",
                    "size": 1024,
                    "type": "csv"
                }
            ]
        }"#;
        
        let fp = Fingerprint::from_bytes(json.as_bytes()).unwrap();
        assert_eq!(fp.version, "1.0");
        assert_eq!(fp.assets.len(), 1);
        assert_eq!(fp.assets[0].name, "buildings.csv");
    }
}