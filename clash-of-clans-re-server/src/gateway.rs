use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, debug};
use sodiumoxide::crypto::box_;
use clash_of_clans_re_protocol::{Packet, MessageId, ProtocolCodec};
use crate::session::{SessionManager, SessionRef};
use crate::error::{Result};

pub struct Gateway {
    listener: TcpListener,
    session_manager: Arc<SessionManager>,
    server_longterm_public: [u8; 32],
    server_longterm_secret: [u8; 32],
}

impl Gateway {
    pub async fn bind(addr: SocketAddr, server_longterm_keypair: (box_::PublicKey, box_::SecretKey)) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        info!("Gateway listening on {}", addr);
        
        Ok(Self {
            listener,
            session_manager: Arc::new(SessionManager::new()),
            server_longterm_public: server_longterm_keypair.0.0,
            server_longterm_secret: server_longterm_keypair.1.0,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let server_longterm_public = self.server_longterm_public;
        let server_longterm_secret = self.server_longterm_secret;
        loop {
            let (stream, addr) = self.listener.accept().await?;
            let session_manager = self.session_manager.clone();
            let server_longterm_public = server_longterm_public;
            let server_longterm_secret = server_longterm_secret;
            
            tokio::spawn(async move {
                let server_longterm_keypair = (
                    box_::PublicKey(server_longterm_public),
                    box_::SecretKey(server_longterm_secret),
                );
                if let Err(e) = handle_connection(stream, addr, session_manager, server_longterm_keypair).await {
                    error!("Connection error from {}: {}", addr, e);
                }
            });
        }
    }

    pub fn session_manager(&self) -> Arc<SessionManager> {
        self.session_manager.clone()
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    session_manager: Arc<SessionManager>,
    server_longterm_keypair: (box_::PublicKey, box_::SecretKey),
) -> Result<()> {
    let session_id = format!("{}:{}", addr.ip(), addr.port());
    info!("New connection from {}", session_id);
    
    let session = session_manager.create_session(session_id.clone(), addr);
    let mut codec = ProtocolCodec::new();
    let mut buffer = [0u8; 4096];
    
    // Long-term server keypair (known to clients)
    let mut handshake_state = clash_of_clans_re_protocol::HandshakeState::with_keypair(server_longterm_keypair);
    
    // Ephemeral session keypair
    let server_session_keypair = clash_of_clans_re_protocol::crypto::generate_server_keypair();
    handshake_state.server_keypair = Some(server_session_keypair);
    
    loop {
        let n = match stream.read(&mut buffer).await {
            Ok(0) => {
                info!("Client {} disconnected", session_id);
                break;
            }
            Ok(n) => n,
            Err(e) => {
                error!("Read error from {}: {}", session_id, e);
                break;
            }
        };
        
        session.lock().await.push_data(&buffer[..n]);
        
        while let Some(packet) = session.lock().await.try_pop_packet()? {
            debug!("Received packet: {} ({} bytes)", packet.header.message_id, packet.payload.len());
            
            let response = process_packet(
                &mut handshake_state,
                &mut codec,
                &session,
                packet,
            ).await?;
            
            if let Some(response_packet) = response {
                let encoded = response_packet.encode();
                stream.write_all(&encoded).await?;
                debug!("Sent response: {} ({} bytes)", response_packet.header.message_id, response_packet.payload.len());
            }
        }
    }
    
    session_manager.remove_session(&session_id);
    Ok(())
}

async fn process_packet(
    handshake_state: &mut clash_of_clans_re_protocol::HandshakeState,
    codec: &mut ProtocolCodec,
    _session: &SessionRef,
    packet: Packet,
) -> Result<Option<Packet>> {
    match handshake_state.stage {
        clash_of_clans_re_protocol::HandshakeStage::Initial => {
            if packet.header.message_id == MessageId::ClientHello as u16 {
                // Client sent ClientHello (10100), respond with Encryption (20000) containing server random
                let response = handshake_state.create_server_hello()?;
                return Ok(Some(response));
            }
        }
        clash_of_clans_re_protocol::HandshakeStage::ClientHelloSent => {
            // Client should not send anything here
        }
        clash_of_clans_re_protocol::HandshakeStage::ServerHelloReceived => {
            if packet.header.message_id == MessageId::Login as u16 {
                let response = handshake_state.handle_login(&packet)?;
                return Ok(Some(response));
            }
        }
        clash_of_clans_re_protocol::HandshakeStage::LoginSent => {
            if packet.header.message_id == MessageId::Encryption as u16 {
                handshake_state.handle_encryption_response(&packet)?;
                
                if let Some(keys) = handshake_state.get_client_keys() {
                    codec.set_keys(keys.clone());
                }
            }
        }
        clash_of_clans_re_protocol::HandshakeStage::EncryptionReceived => {
            if packet.header.message_id == MessageId::LoginOk as u16 {
                let _decrypted = handshake_state.handle_login_ok(&packet)?;
                handshake_state.stage = clash_of_clans_re_protocol::HandshakeStage::Complete;
                
                // TODO: Process login data, create avatar, etc.
                info!("Handshake complete for session");
            }
        }
        clash_of_clans_re_protocol::HandshakeStage::LoginOkReceived => {
            // Handshake complete, process game packets
            if handshake_state.get_client_keys_mut().is_some() {
                let decrypted = codec.decrypt_packet(packet)?;
                return handle_game_packet(decrypted).await;
            }
        }
        clash_of_clans_re_protocol::HandshakeStage::Complete => {
            if handshake_state.get_client_keys_mut().is_some() {
                let decrypted = codec.decrypt_packet(packet)?;
                return handle_game_packet(decrypted).await;
            }
        }
    }
    
    Ok(None)
}

async fn handle_game_packet(
    packet: Packet,
) -> Result<Option<Packet>> {
    // TODO: Implement game packet handling
    debug!("Game packet: {}", packet.header.message_id);
    
    // Echo back for now
    Ok(Some(packet))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_gateway_bind() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let (pk, sk) = clash_of_clans_re_protocol::crypto::generate_server_keypair();
        let gateway = Gateway::bind(addr, (pk, sk)).await;
        assert!(gateway.is_ok());
    }
}