use clash_of_clans_re_protocol::{HandshakeState, Packet, MessageId, crypto};

#[test]
fn test_full_handshake() {
    sodiumoxide::init().unwrap();
    
    // Generate server long-term keypair
    let server_longterm_keypair = crypto::generate_server_keypair();
    let server_public_key = server_longterm_keypair.0.0;
    
    // Client side
    let mut client_state = HandshakeState::new(server_public_key);
    let client_hello = client_state.create_client_hello().unwrap();
    assert_eq!(client_hello.header.message_id, MessageId::ClientHello as u16);
    
    // Server side
    let mut server_state = HandshakeState::with_keypair(server_longterm_keypair);
    let server_hello = server_state.create_server_hello().unwrap();
    assert_eq!(server_hello.header.message_id, MessageId::Encryption as u16);
    
    // Client receives ServerHello
    client_state.handle_server_hello(&server_hello).unwrap();
    
    // Client creates Login
    let login = client_state.create_login().unwrap();
    assert_eq!(login.header.message_id, MessageId::Login as u16);
    
    // Server receives Login
    let server_encryption_response = server_state.handle_login(&login).unwrap();
    assert_eq!(server_encryption_response.header.message_id, MessageId::Encryption as u16);
    
    // Client receives Encryption response
    client_state.handle_encryption_response(&server_encryption_response).unwrap();
    
    // Server session keypair should be set
    assert!(server_state.server_keypair.is_some());
    
    // Client keys should be set
    assert!(client_state.client_keys.is_some());
    assert!(server_state.client_keys.is_some());
    
    // Test encryption/decryption roundtrip
    let test_data = b"Hello, CoC!";
    let client_keys = client_state.get_client_keys_mut().unwrap();
    let client_nonce = client_keys.next_client_nonce();
    let encrypted = client_keys.encrypt(test_data, &client_nonce).unwrap();
    
    let server_keys = server_state.get_client_keys_mut().unwrap();
    // Server decrypts client->server packets with client's snonce (nonce field)
    let server_client_nonce = server_keys.next_client_nonce();
    let decrypted = server_keys.decrypt(&encrypted, &server_client_nonce).unwrap();
    
    assert_eq!(test_data, decrypted.as_slice());
}