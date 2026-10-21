use sodiumoxide::crypto::box_;
use hex;

fn main() {
    sodiumoxide::init().unwrap();
    let (pk, sk) = box_::gen_keypair();
    println!("Public key: {}", hex::encode(&pk.0));
    println!("Secret key: {}", hex::encode(&sk.0));
}
