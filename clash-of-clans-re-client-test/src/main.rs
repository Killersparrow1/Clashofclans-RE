use clap::Parser;
use coc_protocol::{HandshakeState, Packet, ProtocolCodec};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;
use bytes::Bytes;
use hex;

#[derive(Parser, Debug)]
#[command(name = "coc-client-test", version, about = "CoC Protocol Test Client")]
struct Args {
    /// Server address
    #[arg(short, long, default_value = "127.0.0.1:9339")]
    addr: SocketAddr,

    /// Server public key (hex encoded, 32 bytes)
    #[arg(short, long)]
    public_key: String,

    /// Log level
    #[arg(short, long, default_value = "debug")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&args.log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let server_public_key = hex::decode(&args.public_key)?;
    if server_public_key.len() != 32 {
        anyhow::bail!("Public key must be 32 bytes (64 hex chars)");
    }
    let server_public_key: [u8; 32] = server_public_key.try_into().unwrap();

    println!("Connecting to {}", args.addr);
    let mut stream = TcpStream::connect(args.addr).await?;
    println!("Connected!");

    let mut handshake_state = HandshakeState::new(server_public_key);
    let mut codec = ProtocolCodec::new();
    let mut buffer = [0u8; 4096];

    // Step 1: Send ClientHello
    println!("Sending ClientHello...");
    let client_hello = handshake_state.create_client_hello()?;
    stream.write_all(&client_hello.encode()).await?;
    
    // Read ServerHello (Encryption packet with server random)
    let n = stream.read(&mut buffer).await?;
    let response = Packet::decode(Bytes::copy_from_slice(&buffer[..n]))?;
    println!("Received: {} ({} bytes)", response.header.message_id, response.payload.len());
    
    handshake_state.handle_server_hello(&response)?;
    println!("Server hello processed");

    // Step 2: Send Login
    println!("Sending Login...");
    let login = handshake_state.create_login()?;
    stream.write_all(&login.encode()).await?;
    
    // Read Encryption Response
    let n = stream.read(&mut buffer).await?;
    let response = Packet::decode(Bytes::copy_from_slice(&buffer[..n]))?;
    println!("Received: {} ({} bytes)", response.header.message_id, response.payload.len());
    
    handshake_state.handle_encryption_response(&response)?;
    
    if let Some(keys) = handshake_state.get_client_keys() {
        codec.set_keys(keys.clone());
    }
    println!("Encryption response processed");
    println!("Handshake complete!");
    
    // Handshake complete! The server doesn't implement game logic yet,
    // so we just verify the handshake completed successfully.
    println!("SUCCESS: Handshake completed successfully!");
    
    // Give server a moment to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(())
}