//! Key Extractor Module
//! 
//! Tools for extracting encryption keys from Clash of Clans APK.

pub mod frida;
pub mod static_analysis;

use crate::assets::AssetError;
use crate::logic::game_data::ExtractedKeys;

pub use frida::FridaKeyExtractor;
pub use static_analysis::StaticKeyExtractor;

pub struct KeyExtractorManager {
    frida_extractor: Option<super::key_extractor::frida::FridaKeyExtractor>,
    static_extractor: Option<super::key_extractor::static_analysis::StaticKeyExtractor>,
}

impl KeyExtractorManager {
    pub fn new() -> Self {
        Self {
            frida_extractor: None,
            static_extractor: None,
        }
    }
    
    pub fn with_frida(mut self, package_name: &str) -> Self {
        self.frida_extractor = Some(crate::key_extractor::frida::FridaKeyExtractor::new(package_name));
        self
    }
    
    pub fn with_static_analysis(mut self, apk_path: &str) -> Self {
        self.static_extractor = Some(crate::key_extractor::static_analysis::StaticKeyExtractor::new(apk_path));
        self
    }
    
    pub async fn extract_all(&self) -> Result<crate::logic::game_data::ExtractedKeys, crate::assets::AssetError> {
        // Try Frida first (requires rooted device/emulator)
        if let Some(ref extractor) = self.frida_extractor {
            match extractor.extract_keys().await {
                Ok(mut keys) => {
                    keys.extraction_method = "frida".to_string();
                    return Ok(keys);
                }
                Err(e) => {
                    log::warn!("Frida extraction failed: {}", e);
                }
            }
        }
        
        // Fallback to static analysis
        if let Some(ref extractor) = self.static_extractor {
            match extractor.extract_keys() {
                Ok(mut keys) => {
                    keys.extraction_method = "static_analysis".to_string();
                    return Ok(keys);
                }
                Err(e) => {
                    log::warn!("Static analysis failed: {}", e);
                }
            }
        }
        
        Err(crate::assets::AssetError::DecryptionFailed("All extraction methods failed".into()))
    }
}