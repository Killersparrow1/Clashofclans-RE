# Clashofclans-RE

> **Clash of Clans Protocol Implementation in Rust**

A clean-room implementation of the Clash of Clans networking protocol with a working handshake, async TCP server, and test client. Built for research, education, and private server development.

---

## 🎯 Project Status

| Component | Status | Tests |
|-----------|--------|-------|
| **Protocol Handshake** | ✅ Complete | 20 passing |
| **Packet Framing/Codec** | ✅ Complete | 14 passing |
| **NaCl Crypto (libsodium)** | ✅ Complete | 2 passing |
| **TCP Gateway Server** | ✅ Complete | 4 passing |
| **Protocol Version Negotiation** | ✅ Complete (v1.0) | 1 passing |
| **Game Data Structures** | ✅ Complete | 6 passing |
| **CSV Asset Parsing** | ✅ Complete | 3 passing |
| **Fingerprint/SC Parsing** | ✅ Complete | 3 passing |
| **Key Extractor (Frida + Static)** | ✅ Complete | - |
| **Building Placement/Upgrade** | ✅ Complete | 3 passing |
| **Resource Management** | ✅ Complete | 3 passing |
| **Combat Simulation** | ✅ Complete | 3 passing |
| **Game Logic (Logic\*)** | ⚠️ Partial | - |
| **Asset Decryption** | ❌ Not started | - |

**Current capability**: Full 4-step handshake with version negotiation → encrypted channel → packet echo server + building placement, resource management, and combat simulation

---

## 📦 Architecture

```
Clashofclans-RE/
├── clash-of-clans-re-protocol/     # Core protocol (framing, crypto, codec, handshake)
├── clash-of-clans-re-assets/       # Asset decryption (CSV, SC, fingerprint)
├── clash-of-clans-re-server/       # Server library (gateway, sessions, state, logic, key_extractor)
├── clash-of-clans-re-server-bin/   # Server binary
├── clash-of-clans-re-client-test/  # Test client for verification
└── examples/                       # Keypair generator, etc.
```

### Protocol Flow

```
Client                          Server
  │                                │
  ├─ ClientHello (10100) ─────────►│
  │◄── Encryption/ServerHello (20000)│
  │                                │
  ├─ Login (10101) ───────────────►│
  │◄── Encryption Response (20000) │
  │                                │
  ├─ [Encrypted Channel Established]│
  │                                │
  ├─ Encrypted Game Packets ──────►│
  │◄── Encrypted Game Packets ─────│
```

---

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ (`rustup default stable`)
- libsodium (system package: `libsodium-dev` / `libsodium-devel`)

### Build & Run Server

```bash
git clone https://github.com/killersparrow1/Clashofclans-RE
cd Clashofclans-RE

# Generate a long-term keypair
cargo run --example gen_keypair
# Output:
# Public key:  17e185b2094afa806bbec2625dfea7b472283fc8e06ced1ab4ae06d50250fe37
# Secret key:  47c449ee26e87acaf4fb2646d81918df8c29ddb75bd114c0f2ce132927ae520f

# Run server
cargo run -p clash-of-clans-re-server-bin -- \
  --addr 127.0.0.1:9339 \
  --database sqlite:server.db \
  --public-key 17e185b2094afa806bbec2625dfea7b472283fc8e06ced1ab4ae06d50250fe37 \
  --secret-key 47c449ee26e87acaf4fb2646d81918df8c29ddb75bd114c0f2ce132927ae520f
```

### Run Test Client

```bash
cargo run -p clash-of-clans-re-client-test -- \
  --addr 127.0.0.1:9339 \
  --public-key 17e185b2094afa806bbec2625dfea7b472283fc8e06ced1ab4ae06d50250fe37
```

Expected output:
```
Connected!
Sending ClientHello...
Server hello processed
Sending Login...
Encryption response processed
Handshake complete!
SUCCESS: Handshake completed successfully!
```

### Run Tests

```bash
cargo test --workspace
```

### Extract Keys from APK (Optional)

```bash
# Using Frida (requires rooted device/emulator)
cargo run -p clash-of-clans-re-server-bin -- --help  # Shows key extractor options

# Frida-based extraction (runtime)
# Static analysis (apktool + jadx + objdump)
```

---

## 🔧 Protocol Details

Full specification: **[PROTOCOL.md](PROTOCOL.md)**

### Packet Format

```
+00: u16   Message ID (big-endian)
+02: u24   Payload length (big-endian, 3 bytes)
+05: u16   Version (usually 0)
+07: []    Encrypted payload
```

### Message IDs (Partial)

| Range | Direction | Examples |
|-------|-----------|----------|
| 10100-19999 | Client→Server | ClientHello(10100), Login(10101), AttackHome(14106) |
| 20000+ | Server→Client | Encryption(20000), LoginOk(20104), OwnHomeData(24101) |

### Handshake Crypto (NaCl/libsodium)

1. **Server** generates long-term keypair `(pk_longterm, sk_longterm)`
2. **Client** sends `ClientHello` with random seed
3. **Server** responds with `ServerHello` (24-byte random)
4. **Client** generates ephemeral keypair, computes shared key:  
   `shared = precompute(pk_server_longterm, sk_client)`
6. **Client** sends `Login` encrypted with `shared`, includes client_pk
7. **Server** computes: `shared = precompute(pk_client, sk_server_longterm)`
8. **Server** generates session keypair, sends `Encryption Response` encrypted with `shared`, includes session_pk
9. **Both** derive session shared key: `precompute(session_pk, sk_client) = precompute(pk_client, session_sk)`
10. **All subsequent traffic** uses session shared key with incrementing nonces

### Nonce Management
- Two independent nonces: `snonce` (client→server), `rnonce` (server→client)
- Both start from derived values, increment by 2 per packet
- 24-byte nonces (ChaCha20-Poly1305)

---

## 📁 Crate Documentation

### `clash-of-clans-re-protocol`
Core protocol implementation:
- `framing` - Packet header/payload handling, buffering
- `codec` - VInt, VLong, String, DataReference, Boolean serialization
- `crypto` - ClientKeys, key derivation, encryption/decryption
- `handshake` - HandshakeState machine, ServerHello/Login handling
- `messages` - 151 MessageId enum with direction helpers

### `clash-of-clans-re-assets`
Asset decryption and parsing:
- `csv` - Encrypted CSV parser (AES-128-CBC, LZMA/zlib, CRC32)
- `fingerprint` - Fingerprint.json parser (CRC32 verification)
- `sc` - SC sprite container parser

### `clash-of-clans-re-server`
Server infrastructure:
- `gateway` - TCP listener, connection handling, packet pipeline
- `session` - Session management with DashMap
- `state` - SQLite persistence (players, alliances, home data)
- `logic` - Game data structures (BuildingData, CharacterData, etc.)
- `key_extractor` - Frida + Static analysis key extraction

### `clash-of-clans-re-server-bin`
- CLI with `--addr`, `--database`, `--public-key`, `--secret-key`
- Generates ephemeral session keypair per connection

### `clash-of-clans-re-client-test`
- Full handshake implementation for testing/verification
- Useful for protocol debugging

---

## 🔌 Extending the Server

### Adding a New Message Handler
```rust
// In gateway.rs process_packet()
async fn handle_game_packet(packet: Packet) -> Result<Option<Packet>> {
    match packet.header.message_id {
        MessageId::AttackHome as u16 => {
            let attack = decode_attack(packet.payload)?;
            let result = process_attack(attack).await?;
            Ok(Some(encode_attack_result(result)))
        }
        _ => Ok(None),
    }
}
```

### Adding Game Logic
Implement in `clash-of-clans-re-server/src/logic/`:
```rust
// logic/buildings.rs
pub struct LogicBuilding {
    pub building_id: i32,
    pub level: i32,
    pub position: (i32, i32),
}

// logic/combat.rs
pub async fn process_attack(attack: Attack) -> AttackResult { ... }
```

---

## 📊 Database Schema

```sql
-- players table
CREATE TABLE players (
    id INTEGER PRIMARY KEY,
    device_id TEXT UNIQUE NOT NULL,
    avatar_name TEXT,
    avatar_id INTEGER UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- player_data (serialized home JSON)
CREATE TABLE player_data (
    player_id INTEGER PRIMARY KEY,
    home_json TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (player_id) REFERENCES players(id)
);

-- alliances & members
CREATE TABLE alliances ( ... );
CREATE TABLE alliance_members ( ... );
```

---

## ⚖️ Legal & Ethical

### ⚠️ Important Disclaimer
```
This project is for EDUCATIONAL and RESEARCH purposes only.

- DO NOT connect to Supercell production servers
- DO NOT use for cheating, automation, or botting on live games
- DO NOT distribute patched APKs publicly
- Respect Supercell's Fan Content Policy: https://supercell.com/fan-content-policy/
- Use only in isolated environments (local LAN, VMs, test devices)
```

### License
Dual-licensed under **MIT OR Apache-2.0**  
See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE)

---

## 🗺️ Roadmap

### Phase 1: Protocol Polish — ✅ **COMPLETE** (v0.2.0)
- [x] Handshake implementation
- [x] Packet framing & codec
- [x] Nonce synchronization
- [x] Packet documentation (151 messages in `PROTOCOL.md`)
- [x] Protocol version negotiation (v1.0)

### Phase 2: Game Logic Foundation — ✅ **COMPLETE** (v0.2.0)
- [x] CSV asset decryption (buildings.csv, characters.csv, etc.)
- [x] LogicBuilding / LogicCharacter data structures
- [x] CSV parser with AES-128-CBC, LZMA/zlib, CRC32
- [x] Fingerprint.json / SC file parsing
- [x] Key extractor module (Frida + Static analysis)

### Phase 3: Core Gameplay — 🟡 **IN PROGRESS** (v0.3.0)
- [x] CSV key extraction framework (Frida + Static analysis)
- [x] Building placement & upgrade logic (grid collision, validation)
- [x] Resource management (production, storage, spending, looting)
- [x] Combat simulation (A* pathfinding, troop deployment, targeting, attacks)
- [ ] CSV key extraction from APK (Frida/Static)
- [ ] Real CSV parsing (buildings.csv, characters.csv)
- [ ] Spell effects & hero abilities
- [ ] Clan Castle donations

### Phase 4: Multiplayer & Social
- [ ] Alliance system (chat, donations, wars)
- [ ] Matchmaking (trophy-based)
- [ ] Replay system

### Phase 5: Client & Assets
- [ ] Asset decryption (.sc sprites, .csv data)
- [ ] Renderer (wgpu/bevy)
- [ ] Native Linux/macOS/Windows client

---

## 🤝 Contributing

### Getting Started
```bash
# 1. Fork & clone
git clone https://github.com/killersparrow1/Clashofclans-RE
cd Clashofclans-RE

# 2. Run tests
cargo test

# 3. Create feature branch
git checkout -b feat/your-feature

# 4. Make changes, add tests
# 5. Ensure all tests pass
cargo test
cargo clippy -- -D warnings
cargo fmt --check

# 6. Submit PR
```

### Contribution Guidelines
- **All PRs must pass `cargo test` and `cargo clippy`**
- **Follow Rust API guidelines** (naming, docs, errors)
- **Add tests for new protocol messages**
- **Document crypto/handshake changes carefully**
- **No breaking changes without major version bump**

### Good First Issues
| Issue | Difficulty | Area |
|-------|------------|------|
| Document MessageId enum with comments | Easy | `clash-of-clans-re-protocol/src/messages/` |
| Add packet logging feature flag | Easy | `clash-of-clans-re-server/src/gateway.rs` |
| Implement LoginOk response payload | Medium | `clash-of-clans-re-protocol/src/handshake.rs` |
| CSV key extraction from APK | Hard | `clash-of-clans-re-server/src/key_extractor/` |

---

## 📚 References & Credits

### Protocol Documentation
- [clanner/cocdp](https://github.com/clanner/cocdp) - Protocol specification
- [Ultrapowa Clash Server (UCS)](https://github.com/ogclash/ultrapower-clash-server) - C# reference implementation
- [magic-rs](https://github.com/thexeondev/magic-rs) - Rust libg.so bindings

### Tools & Libraries
- **sodiumoxide** - NaCl/libsodium bindings
- **tokio** - Async runtime
- **sqlx** - SQLite async
- **dashmap** - Concurrent HashMap
- **aes/cbc** - AES encryption
- **crc32fast** - CRC32 checksums

### Research Posts
- [Giovanni Rocca's Supercell RE series](http://www.giovanni-rocca.com/reverse-engineering-supercell-part-4/)
- [Hacc Wiki - Packets](https://hacc.natesworks.com/guides/server/packets/)

---

## 💬 Community

- **Issues**: Bug reports, feature requests, protocol questions
- **Discussions**: Architecture decisions, protocol analysis
- **Security**: Report vulnerabilities privately via GitHub Security Advisory

---

## 📄 License

```
MIT License / Apache License 2.0

Copyright (c) 2024 Clashofclans-RE contributors

Permission is hereby granted...
```

---

## 🌟 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=killersparrow1/Clashofclans-RE&type=Date)](https://star-history.com/#killersparrow1/Clashofclans-RE&Date)

---

**Built with ❤️ for the reverse engineering & Rust communities**