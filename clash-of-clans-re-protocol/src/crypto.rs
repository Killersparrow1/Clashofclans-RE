use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::generichash;
use sodiumoxide::randombytes;
use crate::error::{ProtocolError, Result};

pub const BASE_KEY: &[u8] = b"fhsd6f86f67rt8fw78fw789we78r9789wer6re";
pub const SERVER_KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;
pub const SHARED_KEY_LEN: usize = 32;
pub const PUBLIC_KEY_LEN: usize = 32;
pub const SECRET_KEY_LEN: usize = 32;

#[derive(Clone)]
pub struct ClientKeys {
    pub public_key: [u8; PUBLIC_KEY_LEN],
    pub secret_key: [u8; SECRET_KEY_LEN],
    pub shared_key: [u8; SHARED_KEY_LEN],
    pub nonce: [u8; NONCE_LEN],
    pub server_nonce: [u8; NONCE_LEN],
}

impl ClientKeys {
    pub fn new(server_public_key: &[u8; PUBLIC_KEY_LEN]) -> Result<Self> {
        let (pk, sk) = box_::gen_keypair();
        let server_pk = box_::PublicKey::from_slice(server_public_key)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid server public key".into()))?;
        let shared_key = box_::precompute(&server_pk, &sk);
        
        
        let mut nonce = [0u8; NONCE_LEN];
        randombytes::randombytes_into(&mut nonce);
        
        let client_nonce = derive_nonce(&pk.0, &shared_key.0);
        
        Ok(Self {
            public_key: pk.0,
            secret_key: sk.0,
            shared_key: shared_key.0,
            nonce: client_nonce,
            server_nonce: [0u8; NONCE_LEN],
        })
    }

    pub fn set_server_nonce(&mut self, server_nonce: [u8; NONCE_LEN]) {
        self.server_nonce = server_nonce;
    }

    pub fn next_client_nonce(&mut self) -> [u8; NONCE_LEN] {
        increment_nonce(&mut self.nonce);
        self.nonce
    }

    pub fn next_server_nonce(&mut self) -> [u8; NONCE_LEN] {
        increment_nonce(&mut self.server_nonce);
        self.server_nonce
    }

    pub fn encrypt(&self, data: &[u8], nonce: &[u8; NONCE_LEN]) -> Result<Vec<u8>> {
        let nonce_obj = box_::Nonce::from_slice(nonce)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid nonce length".into()))?;
        let shared = box_::PrecomputedKey::from_slice(&self.shared_key)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid shared key".into()))?;
        
        let encrypted = box_::seal_precomputed(data, &nonce_obj, &shared);
        Ok(encrypted)
    }

    pub fn decrypt(&self, data: &[u8], nonce: &[u8; NONCE_LEN]) -> Result<Vec<u8>> {
        let nonce_obj = box_::Nonce::from_slice(nonce)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid nonce length".into()))?;
        let shared = box_::PrecomputedKey::from_slice(&self.shared_key)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid shared key".into()))?;
        
        box_::open_precomputed(data, &nonce_obj, &shared)
            .map_err(|_| ProtocolError::DecryptionFailed("Failed to decrypt".into()))
    }
}

pub fn derive_nonce(client_pk: &[u8; PUBLIC_KEY_LEN], shared_key: &[u8; SHARED_KEY_LEN]) -> [u8; NONCE_LEN] {
    let mut input = Vec::with_capacity(PUBLIC_KEY_LEN + SHARED_KEY_LEN);
    input.extend_from_slice(client_pk);
    input.extend_from_slice(shared_key);
    
    let hash = generichash::hash(&input, Some(NONCE_LEN), None)
        .expect("blake2b hash failed");
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&hash.as_ref()[..NONCE_LEN]);
    nonce
}

fn increment_nonce(nonce: &mut [u8; NONCE_LEN]) {
    for byte in nonce.iter_mut().rev() {
        *byte = byte.wrapping_add(2);
        if *byte != 0 || *byte == 2 {
            break;
        }
    }
}

pub fn generate_server_keypair() -> (box_::PublicKey, box_::SecretKey) {
    box_::gen_keypair()
}

pub fn keypair_from_bytes(pk: [u8; 32], sk: [u8; 32]) -> std::result::Result<(box_::PublicKey, box_::SecretKey), ProtocolError> {
    let pk = box_::PublicKey::from_slice(&pk)
        .ok_or_else(|| ProtocolError::CryptoError("Invalid public key".into()))?;
    let sk = box_::SecretKey::from_slice(&sk)
        .ok_or_else(|| ProtocolError::CryptoError("Invalid secret key".into()))?;
    Ok((pk, sk))
}

pub fn handshake_server_random() -> [u8; 24] {
    let mut random = [0u8; 24];
    randombytes::randombytes_into(&mut random);
    random
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_increment() {
        let mut nonce = [0u8; 24];
        nonce[23] = 0xFE;
        increment_nonce(&mut nonce);
        assert_eq!(nonce[23], 0x00);
        assert_eq!(nonce[22], 0x02);
    }

    #[test]
    fn test_derive_nonce() {
        let pk = [1u8; 32];
        let shared = [2u8; 32];
        let nonce = derive_nonce(&pk, &shared);
        assert_eq!(nonce.len(), 24);
    }
}