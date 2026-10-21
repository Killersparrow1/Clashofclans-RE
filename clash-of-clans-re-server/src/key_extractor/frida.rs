//! Frida-based Key Extractor
//! 
//! Uses Frida to hook into the running Clash of Clans process
//! and extract encryption keys from memory.

use crate::logic::game_data::ExtractedKeys;
use crate::assets::AssetError;
use std::process::Command;

pub struct FridaKeyExtractor {
    package_name: String,
    script_path: Option<String>,
}

impl FridaKeyExtractor {
    pub fn new(package_name: &str) -> Self {
        Self {
            package_name: package_name.to_string(),
            script_path: None,
        }
    }
    
    pub fn with_script(mut self, script_path: &str) -> Self {
        self.script_path = Some(script_path.to_string());
        self
    }
    
    pub async fn extract_keys(&self) -> Result<crate::logic::game_data::ExtractedKeys, crate::assets::AssetError> {
        let script = self.get_frida_script();
        
        // Write script to temp file
        let script_file = format!("/tmp/coc_key_extractor_{}.js", chrono::Utc::now().timestamp());
        tokio::fs::write(&script_file, script).await
            .map_err(|e| crate::assets::AssetError::Io(e))?;
        
        // Run Frida
        let output = Command::new("frida")
            .args([
                "-U",                    // USB device
                "-f", &self.package_name, // spawn app
                "-l", &script_file,      // load script
                "--no-pause",            // don't pause after spawn
            ])
            .output()
            .map_err(|e| crate::assets::AssetError::DecryptionFailed(format!("Frida failed: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::assets::AssetError::DecryptionFailed(
                format!("Frida error: {}", stderr)
            ));
        }
        
        // Parse output
        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_frida_output(&stdout)
    }
    
    fn get_frida_script(&self) -> String {
        r#"
// Clash of Clans Key Extractor - Frida Script
// Hooks into the game to extract AES keys from memory

Java.perform(function() {
    console.log("[*] Starting CoC key extraction...");
    
    // Hook 1: AES Cipher initialization
    try {
        var Cipher = Java.use("javax.crypto.Cipher");
        Cipher.init.overload('int', 'java.security.Key', 'java.security.spec.AlgorithmParameterSpec').implementation = function(opmode, key, params) {
            var algorithm = this.getAlgorithm();
            if (algorithm.indexOf("AES") !== -1) {
                console.log("[+] AES Cipher init detected:");
                console.log("    Algorithm: " + algorithm);
                console.log("    Mode: " + opmode);
                console.log("    Key: " + bytesToHex(key.getEncoded()));
                if (params) {
                    var iv = params.getIV();
                    if (iv) {
                        console.log("    IV: " + bytesToHex(iv));
                    }
                }
                
                // Check if this is CSV decryption
                var stack = Java.use("java.lang.Thread").currentThread().getStackTrace();
                var stackStr = "";
                for (var i = 0; i < Math.min(stack.length, 10); i++) {
                    stackStr += "    " + stack[i].toString() + "\n";
                }
                console.log("    Stack:\n" + stackStr);
            }
            return this.init(opmode, key, params);
        };
        
        // Hook 2: SecretKeySpec constructor
        var SecretKeySpec = Java.use("javax.crypto.spec.SecretKeySpec");
        SecretKeySpec.$init.overload('[B', 'java.lang.String').implementation = function(keyBytes, algorithm) {
            console.log("[+] SecretKeySpec created:");
            console.log("    Algorithm: " + algorithm);
            console.log("    Key: " + bytesToHex(keyBytes));
            var stack = Java.use("java.lang.Thread").currentThread().getStackTrace();
            for (var i = 0; i < Math.min(stack.length, 15); i++) {
                console.log("    " + stack[i].toString());
            }
            return this.$init(keyBytes, algorithm);
        };
        
        // Hook 3: IvParameterSpec constructor
        var IvParameterSpec = Java.use("javax.crypto.spec.IvParameterSpec");
        IvParameterSpec.$init.overload('[B').implementation = function(ivBytes) {
            console.log("[+] IvParameterSpec created:");
            console.log("    IV: " + bytesToHex(ivBytes));
            var stack = Java.use("java.lang.Thread").currentThread().getStackTrace();
            for (var i = 0; i < Math.min(stack.length, 10); i++) {
                console.log("    " + stack[i].toString());
            }
            return this.$init(ivBytes);
        };
        
        // Hook 4: Native library loading (libg.so)
        var System = Java.use("java.lang.System");
        System.loadLibrary.overload('java.lang.String').implementation = function(libName) {
            console.log("[+] Loading native library: " + libName);
            if (libName === "g") {
                console.log("    [!!!] libg.so loaded - this is the game engine!");
            }
            return this.loadLibrary(libName);
        };
        
        // Hook 5: File reading (for fingerprint.json, CSV files)
        var FileInputStream = Java.use("java.io.FileInputStream");
        FileInputStream.$init.overload('java.io.File').implementation = function(file) {
            var path = file.getAbsolutePath();
            if (path.includes(".csv") || path.includes("fingerprint")) {
                console.log("[+] Reading file: " + path);
            }
            return this.$init(file);
        };
        
        // Hook 6: Native crypto functions (JNI)
        var System = Java.use("java.lang.System");
        System.load.overload('java.lang.String').implementation = function(libName) {
            console.log("[+] System.load: " + libName);
            return this.load(libName);
        };
        
        console.log("[*] All hooks installed. Waiting for game activity...");
    });
    
    function bytesToHex(bytes) {
        var result = "";
        for (var i = 0; i < bytes.length; i++) {
            result += ("0" + (bytes[i] & 0xFF).toString(16)).slice(-2);
        }
        return result;
    }
    
    // Send results back to host
    function sendKey(type, key, iv) {
        send({
            type: "key_extracted",
            key_type: type,
            key: bytesToHex(key),
            iv: iv ? bytesToHex(iv) : null,
            timestamp: new Date().toISOString()
        });
    }
} catch (e) {
    console.log("[!] Error: " + e);
}
"#.to_string()
    }
    
    fn parse_frida_output(&self, output: &str) -> Result<crate::logic::game_data::ExtractedKeys, crate::assets::AssetError> {
        let mut keys = crate::logic::game_data::ExtractedKeys::new();
        keys.extraction_method = "frida".to_string();
        
        for line in output.lines() {
            if line.contains("[+] SecretKeySpec created:") {
                // Parse key from log
                if let Some(key_start) = line.find("Key: ") {
                    let key_hex = &line[key_start + 5..key_start + 37];
                    if key_hex.len() == 32 {
                        if let Ok(key) = hex::decode(key_hex) {
                            let mut arr = [0u8; 16];
                            arr.copy_from_slice(&key);
                            keys.csv_aes_key = arr;
                        }
                    }
                }
            }
            if line.contains("[+] IvParameterSpec created:") {
                if let Some(iv_start) = line.find("IV: ") {
                    let iv_hex = &line[iv_start + 4..iv_start + 36];
                    if iv_hex.len() == 32 {
                        if let Ok(iv) = hex::decode(iv_hex) {
                            let mut arr = [0u8; 16];
                            arr.copy_from_slice(&iv);
                            keys.csv_aes_iv = arr;
                        }
                    }
                }
            }
        }
        
        if !keys.is_valid() {
            return Err(crate::assets::AssetError::DecryptionFailed("No valid keys extracted".into()));
        }
        
        Ok(keys)
    }
}