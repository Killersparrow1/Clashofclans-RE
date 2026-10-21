pub mod codec;
pub mod crypto;
pub mod error;
pub mod framing;
pub mod handshake;
pub mod messages;

pub use codec::{Decodable, Encodable};
pub use crypto::{ClientKeys, BASE_KEY, NONCE_LEN};
pub use error::{ProtocolError, Result};
pub use framing::{Packet, PacketBuffer, PacketHeader, HEADER_SIZE};
pub use handshake::{HandshakeState, HandshakeStage};
pub use messages::MessageId;

use bytes::Bytes;

pub struct ProtocolCodec {
    client_keys: Option<crate::crypto::ClientKeys>,
    is_encrypted: bool,
}

impl ProtocolCodec {
    pub fn new() -> Self {
        Self {
            client_keys: None,
            is_encrypted: false,
        }
    }

    pub fn set_keys(&mut self, keys: crate::crypto::ClientKeys) {
        self.client_keys = Some(keys);
        self.is_encrypted = true;
    }

    pub fn encrypt_packet(&mut self, packet: Packet) -> Result<Packet> {
        if !self.is_encrypted {
            return Ok(packet);
        }
        
        let keys = self.client_keys.as_mut()
            .ok_or_else(|| ProtocolError::CryptoError("No keys available".into()))?;
        
        let nonce = keys.next_client_nonce();
        let encrypted_payload = keys.encrypt(&packet.payload, &nonce)?;
        
        Ok(Packet::new(packet.header.message_id, Bytes::from(encrypted_payload), packet.header.version))
    }

    pub fn decrypt_packet(&mut self, packet: Packet) -> Result<Packet> {
        if !self.is_encrypted {
            return Ok(packet);
        }
        
        let keys = self.client_keys.as_mut()
            .ok_or_else(|| ProtocolError::CryptoError("No keys available".into()))?;
        
        let nonce = keys.next_server_nonce();
        let decrypted_payload = keys.decrypt(&packet.payload, &nonce)?;
        
        Ok(Packet::new(packet.header.message_id, Bytes::from(decrypted_payload), packet.header.version))
    }
}

impl Default for ProtocolCodec {
    fn default() -> Self {
        Self::new()
    }
}