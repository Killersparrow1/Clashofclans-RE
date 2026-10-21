use clash_of_clans_re_protocol::crypto;

fn main() {
    let (pk, sk) = crypto::generate_server_keypair();
    println!("Public key:  {}", hex::encode(&pk.0));
    println!("Secret key:  {}", hex::encode(&sk.0));
}