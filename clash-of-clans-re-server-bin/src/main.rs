use clap::Parser;
use clash_of_clans_re_server::{Gateway, GameState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;
use clash_of_clans_re_protocol::crypto;

#[derive(Parser, Debug)]
#[command(name = "coc-server", version, about = "Clash of Clans Private Server")]
struct Args {
    /// Server bind address
    #[arg(short, long, default_value = "127.0.0.1:9339")]
    addr: SocketAddr,

    /// Database URL
    #[arg(short, long, default_value = "sqlite:clash_of_clans_re_server.db")]
    database: String,

    /// Server long-term public key (hex encoded, 32 bytes)
    #[arg(long)]
    public_key: Option<String>,

    /// Server long-term secret key (hex encoded, 32 bytes)
    #[arg(long)]
    secret_key: Option<String>,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&args.log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse or generate server long-term keypair
    let server_longterm_keypair = if let (Some(pk_hex), Some(sk_hex)) = (args.public_key, args.secret_key) {
        let pk_bytes = hex::decode(pk_hex)?;
        let sk_bytes = hex::decode(sk_hex)?;
        if pk_bytes.len() != 32 || sk_bytes.len() != 32 {
            anyhow::bail!("Public and secret keys must be 32 bytes each (64 hex chars)");
        }
        let pk: [u8; 32] = pk_bytes.try_into().unwrap();
        let sk: [u8; 32] = sk_bytes.try_into().unwrap();
        crypto::keypair_from_bytes(pk, sk)?
    } else {
        // Generate a new keypair for development
        let (pk, sk) = crypto::generate_server_keypair();
        println!("Generated server public key: {}", hex::encode(&pk.0));
        println!("Generated server secret key: {}", hex::encode(&sk.0));
        (pk, sk)
    };

    // Initialize game state
    let _game_state = GameState::new(&args.database).await?;

    // Start gateway
    let gateway = Gateway::bind(args.addr, server_longterm_keypair).await?;
    
    println!("CoC Server starting on {}", args.addr);
    println!("Database: {}", args.database);
    
    gateway.run().await?;

    Ok(())
}