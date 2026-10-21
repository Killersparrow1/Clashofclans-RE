//! Static Analysis Key Extractor
//! 
//! Extracts encryption keys from APK using static analysis.
//! Uses apktool, jadx, and binary analysis.

use crate::logic::game_data::ExtractedKeys;
use crate::assets::AssetError;
use std::process::Command;
use std::path::Path;

pub struct StaticKeyExtractor {
    apk_path: String,
    output_dir: String,
}

impl StaticKeyExtractor {
    pub fn new(apk_path: &str) -> Self {
        Self {
            apk_path: apk_path.to_string(),
            output_dir: format!("/tmp/coc_analysis_{}", chrono::Utc::now().timestamp()),
        }
    }
    
    pub fn extract_keys(&self) -> Result<crate::logic::game_data::ExtractedKeys, crate::assets::AssetError> {
        let mut keys = crate::logic::game_data::ExtractedKeys::new();
        keys.extraction_method = "static_analysis".to_string();
        
        // 1. Decompile APK
        self.decompile_apk()?;
        
        // 2. Search for AES keys in DEX files
        self.search_dex_keys(&mut keys)?;
        
        // 3. Search native libraries
        self.search_native_libs(&mut keys)?;
        
        // 4. Search resources (fingerprint.json, etc.)
        self.search_resources(&mut keys)?;
        
        // 5. Cleanup
        self.cleanup()?;
        
        if !keys.is_valid() {
            return Err(crate::assets::AssetError::DecryptionFailed("No valid keys found in static analysis".into()));
        }
        
        Ok(keys)
    }
    
    fn decompile_apk(&self) -> Result<(), crate::assets::AssetError> {
        std::fs::create_dir_all(&self.output_dir)
            .map_err(|e| crate::assets::AssetError::Io(e))?;
        
        // Use apktool to decompile
        let output = Command::new("apktool")
            .args(["d", &self.apk_path, "-o", &self.output_dir, "-f"])
            .output()
            .map_err(|e| crate::assets::AssetError::DecryptionFailed(format!("apktool failed: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::assets::AssetError::DecryptionFailed(
                format!("apktool failed: {}", stderr)
            ));
        }
        
        Ok(())
    }
    
    fn search_dex_keys(&self, mut keys: &mut crate::logic::game_data::ExtractedKeys) -> Result<(), crate::assets::AssetError> {
        // Search for AES key patterns in smali files
        let output = Command::new("grep")
            .args([
                "-r", "-i", "aes", 
                &format!("{}/smali", self.output_dir)
            ])
            .output()
            .map_err(|e| crate::assets::AssetError::Io(e))?;
        
        let _stdout = String::from_utf8_lossy(&output.stdout);
        
        // Look for SecretKeySpec instantiations
        let output = Command::new("grep")
            .args([
                "-r", "SecretKeySpec",
                &format!("{}/smali", self.output_dir)
            ])
            .output()
            .map_err(|e| crate::assets::AssetError::Io(e))?;
        
        let _stdout = String::from_utf8_lossy(&output.stdout);
        
        // Use jadx for better decompilation
        let output = Command::new("jadx")
            .args([
                "--deobf",
                "-d", &format!("{}/jadx_out", self.output_dir),
                &format!("{}/classes.dex", self.output_dir)
            ])
            .output()
            .map_err(|e| crate::assets::AssetError::Io(e))?;
        
        // Search jadx output for crypto constants
        self.search_jadx_output(&mut keys)?;
        
        Ok(())
    }
    
    fn search_jadx_output(&self, mut keys: &mut crate::logic::game_data::ExtractedKeys) -> Result<(), crate::assets::AssetError> {
        let jadx_dir = format!("{}/jadx_out", self.output_dir);
        
        if !Path::new(&jadx_dir).exists() {
            return Ok(());
        }
        
        // Search for AES key patterns
        let patterns = [
            "AES/CBC/PKCS5Padding",
            "AES/CBC/PKCS7Padding", 
            "SecretKeySpec",
            "IvParameterSpec",
            "Cipher.getInstance",
            "AES/CBC",
        ];
        
        for pattern in &patterns {
            let output = Command::new("grep")
                .args(["-r", pattern, &jadx_dir])
                .output()
                .map_err(|e| crate::assets::AssetError::Io(e))?;
            
            let _stdout = String::from_utf8_lossy(&output.stdout);
            // Parse findings
        }
        
        // Search for hardcoded byte arrays (potential keys)
        let output = Command::new("grep")
            .args(["-r", "new byte\\[" , &jadx_dir])
            .output()
            .map_err(|e| crate::assets::AssetError::Io(e))?;
        
        let _stdout = String::from_utf8_lossy(&output.stdout);
        
        Ok(())
    }
    
    fn search_native_libs(&self, keys: &mut crate::logic::game_data::ExtractedKeys) -> Result<(), crate::assets::AssetError> {
        let lib_dir = format!("{}/lib", self.output_dir);
        
        if !Path::new(&lib_dir).exists() {
            return Ok(());
        }
        
        // Search libg.so for crypto constants
        let libg_path = format!("{}/arm64-v8a/libg.so", lib_dir);
        if Path::new(&libg_path).exists() {
            self.analyze_libg(&libg_path, keys)?;
        }
        
        // Also check 32-bit
        let libg32_path = format!("{}/armeabi-v7a/libg.so", lib_dir);
        if Path::new(&libg32_path).exists() {
            self.analyze_libg(&libg32_path, keys)?;
        }
        
        Ok(())
    }
    
    fn analyze_libg(&self, lib_path: &str, _keys: &mut crate::logic::game_data::ExtractedKeys) -> Result<(), crate::assets::AssetError> {
        // Use strings to find crypto constants
        let output = Command::new("strings")
            .args([lib_path])
            .output()
            .map_err(|e| crate::assets::AssetError::Io(e))?;
        
        let _stdout = String::from_utf8_lossy(&output.stdout);
        
        Ok(())
    }
    
    fn search_resources(&self, mut keys: &mut crate::logic::game_data::ExtractedKeys) -> Result<(), crate::assets::AssetError> {
        let res_dir = format!("{}/res", self.output_dir);
        
        if !Path::new(&res_dir).exists() {
            return Ok(());
        }
        
        // Search for fingerprint.json
        let output = Command::new("find")
            .args([&res_dir, "-name", "fingerprint.json"])
            .output()
            .map_err(|e| crate::assets::AssetError::Io(e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for path in stdout.lines() {
            if let Ok(content) = std::fs::read_to_string(path) {
                // Parse fingerprint.json
                if let Ok(fp) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(sha) = fp.get("sha").and_then(|v| v.as_str()) {
                        keys.fingerprint_sha = sha.to_string();
                    }
                    if let Some(ver) = fp.get("version").and_then(|v| v.as_str()) {
                        keys.client_version = ver.to_string();
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), crate::assets::AssetError> {
        std::fs::remove_dir_all(&self.output_dir)
            .map_err(|e| crate::assets::AssetError::Io(e))?;
        Ok(())
    }
}