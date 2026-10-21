use bytes::{BufMut, Bytes, BytesMut};
use sodiumoxide::crypto::box_;
use crate::crypto::{ClientKeys, SERVER_KEY_LEN, NONCE_LEN, derive_nonce, handshake_server_random};
use crate::error::{ProtocolError, Result};
use crate::framing::{Packet};
use crate::messages::MessageId;

pub struct HandshakeState {
    pub client_keys: Option<ClientKeys>,
    pub server_keypair: Option<(box_::PublicKey, box_::SecretKey)>,
    pub server_longterm_secret: Option<[u8; 32]>,
    pub server_public_key: [u8; SERVER_KEY_LEN],
    pub server_nonce: [u8; 24],
    pub client_seed: [u8; 24],
    pub stage: HandshakeStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeStage {
    Initial,
    ClientHelloSent,
    ServerHelloReceived,
    LoginSent,
    EncryptionReceived,
    LoginOkReceived,
    Complete,
}

impl HandshakeState {
    pub fn new(server_public_key: [u8; SERVER_KEY_LEN]) -> Self {
        Self {
            client_keys: None,
            server_keypair: None,
            server_longterm_secret: None,
            server_public_key,
            server_nonce: [0u8; 24],
            client_seed: [0u8; 24],
            stage: HandshakeStage::Initial,
        }
    }

pub fn with_keypair(server_keypair: (box_::PublicKey, box_::SecretKey)) -> Self {
        let server_public_key = server_keypair.0.0;
        let server_secret = server_keypair.1.0;
        Self {
            client_keys: None,
            server_keypair: Some((server_keypair.0, box_::SecretKey(server_secret))),
            server_longterm_secret: Some(server_secret),
            server_public_key,
            server_nonce: [0u8; 24],
client_seed: [0u8; 24],
            stage: HandshakeStage::Initial,
        }
    }

    pub fn create_client_hello(&mut self) -> Result<Packet> {
        let mut payload = BytesMut::new();
        payload.put_u32(0x07896541);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000001);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000000);
        payload.put_u32(0xFD000000);
        
        sodiumoxide::randombytes::randombytes_into(&mut self.client_seed);
        payload.extend_from_slice(&self.client_seed);
        payload.put_u32(0x00000001);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000001);
        payload.extend_from_slice(b"iPad5,3");
        payload.put_u32(0x0005656E);
        payload.put_u32(0x2D555300);
        payload.put_u32(0x00000006);
        payload.extend_from_slice(b"15.8.2");
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000000);
        payload.put_u32(0x00000000);

        let packet = Packet::new(MessageId::ClientHello as u16, payload.freeze(), 0);
        self.stage = HandshakeStage::ClientHelloSent;
        Ok(packet)
    }

    pub fn handle_server_hello(&mut self, packet: &Packet) -> Result<()> {
        if packet.header.message_id != MessageId::Encryption as u16 {
            return Err(ProtocolError::HandshakeError("Expected Encryption packet".into()));
        }
        
        if packet.payload.len() < 24 {
            return Err(ProtocolError::HandshakeError("Server hello too short".into()));
        }
        
        self.server_nonce.copy_from_slice(&packet.payload[..24]);
        self.stage = HandshakeStage::ServerHelloReceived;
        Ok(())
    }

    pub fn create_server_hello(&mut self) -> Result<Packet> {
        let server_random = handshake_server_random();
        self.server_nonce = server_random;
        
        let mut payload = BytesMut::new();
        payload.extend_from_slice(&server_random);
        
        let packet = Packet::new(MessageId::Encryption as u16, payload.freeze(), 0);
        self.stage = HandshakeStage::ServerHelloReceived;
        Ok(packet)
    }

    pub fn create_login(&mut self) -> Result<Packet> {
        let client_keys = ClientKeys::new(&self.server_public_key)?;
        let client_nonce = derive_nonce(&client_keys.public_key, &client_keys.shared_key);
        
        
        let mut payload = BytesMut::new();
        payload.extend_from_slice(&self.server_nonce);
        payload.extend_from_slice(&client_nonce);
        payload.extend_from_slice(&client_keys.public_key);
        
        let shared = box_::PrecomputedKey::from_slice(&client_keys.shared_key)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid shared key".into()))?;
        let nonce_obj = box_::Nonce::from_slice(&client_nonce)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid nonce".into()))?;
        
        let mut login_payload = BytesMut::new();
        login_payload.put_u64(0);
        login_payload.put_u64(0);
        login_payload.put_u32(0);
        login_payload.put_u32(0);
        login_payload.put_u32(0);
        login_payload.put_u32(0);
        login_payload.put_u32(0);
        
        let encrypted = box_::seal_precomputed(&login_payload, &nonce_obj, &shared);
        
        let mut final_payload = BytesMut::new();
        final_payload.extend_from_slice(&self.server_nonce);
        final_payload.extend_from_slice(&client_nonce);
        final_payload.extend_from_slice(&client_keys.public_key);
        final_payload.extend_from_slice(&encrypted);
        
        let packet = Packet::new(MessageId::Login as u16, final_payload.freeze(), 0);
        self.client_keys = Some(client_keys);
        self.stage = HandshakeStage::LoginSent;
        Ok(packet)
    }

    pub fn handle_encryption_response(&mut self, packet: &Packet) -> Result<()> {
        if packet.header.message_id != MessageId::Encryption as u16 {
            return Err(ProtocolError::HandshakeError("Expected Encryption response".into()));
        }
        
        let client_keys = self.client_keys.as_mut()
            .ok_or_else(|| ProtocolError::HandshakeError("No client keys".into()))?;
        
        let nonce = client_keys.nonce;
        
        let decrypted = client_keys.decrypt(&packet.payload, &nonce)?;
        
        
        if decrypted.len() < 48 {
            return Err(ProtocolError::HandshakeError("Encryption response too short".into()));
        }
        
        client_keys.set_server_nonce(decrypted[..24].try_into().unwrap());
        let server_pk = &decrypted[24..56];
        
        let session_pk_obj = box_::PublicKey::from_slice(server_pk)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid session public key".into()))?;
        let new_shared_key = box_::precompute(
            &session_pk_obj,
            &box_::SecretKey::from_slice(&client_keys.secret_key).unwrap(),
        );
        
        client_keys.shared_key = new_shared_key.0;
        
        let new_nonce = derive_nonce(&client_keys.public_key, &client_keys.shared_key);
        client_keys.nonce = new_nonce;
        
        self.stage = HandshakeStage::EncryptionReceived;
        Ok(())
    }

pub fn create_server_encryption_response(&mut self) -> Result<Packet> {
        let client_keys = self.client_keys.as_mut()
            .ok_or_else(|| ProtocolError::HandshakeError("No client keys".into()))?;
        
        let server_nonce = handshake_server_random();
        client_keys.set_server_nonce(server_nonce);
        
        let server_keypair = self.server_keypair.as_ref()
            .ok_or_else(|| ProtocolError::HandshakeError("No server session keypair".into()))?;
        
        let mut payload = BytesMut::new();
        payload.extend_from_slice(&server_nonce);
        payload.extend_from_slice(&server_keypair.0.0);
        
        
        let nonce_obj = box_::Nonce::from_slice(&client_keys.nonce)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid nonce".into()))?;
        let shared_key = box_::PrecomputedKey::from_slice(&client_keys.shared_key)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid shared key".into()))?;
        
        let encrypted = box_::seal_precomputed(&payload, &nonce_obj, &shared_key);
        
        
        let client_pk_obj = box_::PublicKey::from_slice(&client_keys.public_key)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid client public key".into()))?;
        
        let session_shared = box_::precompute(&client_pk_obj, &server_keypair.1);
        client_keys.shared_key = session_shared.0;
        
        let new_nonce = derive_nonce(&client_keys.public_key, &client_keys.shared_key);
        client_keys.nonce = new_nonce;
        
let packet = Packet::new(MessageId::Encryption as u16, Bytes::from(encrypted), 0);
        self.stage = HandshakeStage::LoginSent;
        Ok(packet)
    }
    
    pub fn handle_login(&mut self, packet: &Packet) -> Result<Packet> {
        if packet.header.message_id != MessageId::Login as u16 {
            return Err(ProtocolError::HandshakeError("Expected Login packet".into()));
        }
        
        // Login packet format: server_random (24) + client_nonce (24) + client_pk (32) + encrypted_payload
        if packet.payload.len() < 80 {
            return Err(ProtocolError::HandshakeError("Login packet too short".into()));
        }
        
        let server_random = &packet.payload[..24];
        let client_nonce = &packet.payload[24..48];
        let client_pk = &packet.payload[48..80];
        let encrypted_payload = &packet.payload[80..];
        
        
        // Verify server_random matches what we sent
        if server_random != self.server_nonce {
            return Err(ProtocolError::HandshakeError("Server random mismatch".into()));
        }
        
        // Compute shared key using client's public key and server's long-term secret key
        let client_pk_obj = box_::PublicKey::from_slice(client_pk)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid client public key".into()))?;
        let server_secret_bytes = self.server_longterm_secret
            .as_ref()
            .ok_or_else(|| ProtocolError::HandshakeError("No server long-term secret".into()))?;
        let server_secret = box_::SecretKey(*server_secret_bytes);
        let shared_key = box_::precompute(&client_pk_obj, &server_secret);
        
        // Decrypt the login payload
        let nonce_obj = box_::Nonce::from_slice(client_nonce)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid client nonce".into()))?;
        let shared = box_::PrecomputedKey::from_slice(&shared_key.0)
            .ok_or_else(|| ProtocolError::CryptoError("Invalid shared key".into()))?;
        
        let _decrypted_payload = box_::open_precomputed(encrypted_payload, &nonce_obj, &shared)
            .map_err(|_| ProtocolError::DecryptionFailed("Failed to decrypt login payload".into()))?;
        
        // Store client's public key and shared key
        self.client_keys = Some(crate::crypto::ClientKeys {
            public_key: client_pk.try_into().unwrap(),
            secret_key: server_secret.0,
            shared_key: shared_key.0,
            nonce: client_nonce.try_into().unwrap(),
            server_nonce: [0u8; NONCE_LEN],
        });
        
        // Create server's encryption response using session keypair
        self.create_server_encryption_response()
    }

    pub fn handle_login_ok(&mut self, packet: &Packet) -> Result<Bytes> {
        if packet.header.message_id != MessageId::LoginOk as u16 {
            return Err(ProtocolError::HandshakeError("Expected LoginOk".into()));
        }
        
        let client_keys = self.client_keys.as_mut()
            .ok_or_else(|| ProtocolError::HandshakeError("No client keys".into()))?;
        
        let mut nonce = client_keys.server_nonce;
        increment_nonce(&mut nonce);
        
        let decrypted = client_keys.decrypt(&packet.payload, &nonce)?;
        client_keys.server_nonce = nonce;
        
        self.stage = HandshakeStage::LoginOkReceived;
        Ok(Bytes::from(decrypted))
    }

    pub fn is_complete(&self) -> bool {
        self.stage == HandshakeStage::Complete
    }

    pub fn get_client_keys(&self) -> Option<&ClientKeys> {
        self.client_keys.as_ref()
    }

    pub fn get_client_keys_mut(&mut self) -> Option<&mut ClientKeys> {
        self.client_keys.as_mut()
    }
}

fn increment_nonce(nonce: &mut [u8; NONCE_LEN]) {
    for byte in nonce.iter_mut().rev() {
        *byte = byte.wrapping_add(2);
        if *byte != 0 || *byte == 2 {
            break;
        }
    }
}

pub fn create_server_handshake(_server_keypair: &(box_::PublicKey, box_::SecretKey)) -> (Packet, [u8; 24]) {
    let server_random = handshake_server_random();
    let mut payload = BytesMut::new();
    payload.extend_from_slice(&server_random);
    
    let packet = Packet::new(MessageId::Encryption as u16, payload.freeze(), 0);
    (packet, server_random)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sodiumoxide::init;

    #[test]
    fn test_handshake_state_creation() {
        let _ = init();
        let server_pk = [1u8; 32];
        let state = HandshakeState::new(server_pk);
        assert_eq!(state.stage, HandshakeStage::Initial);
    }

    #[test]
    fn test_client_hello_creation() {
        let _ = init();
        let server_pk = [1u8; 32];
        let mut state = HandshakeState::new(server_pk);
        let packet = state.create_client_hello().unwrap();
        assert_eq!(packet.header.message_id, 10100);
        assert_eq!(state.stage, HandshakeStage::ClientHelloSent);
    }
}