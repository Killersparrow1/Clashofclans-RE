# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial protocol implementation with full handshake support
- Packet framing (msg_id, payload_len, version)
- Codec for VInt, VLong, String, DataReference, Boolean
- NaCl/libsodium crypto: key derivation, encryption, decryption
- Handshake state machine (ClientHello → ServerHello → Login → Encryption Response)
- Async TCP gateway server with Tokio
- Session management with DashMap
- SQLite persistence (players, alliances, home data)
- Test client for handshake verification
- 20+ passing tests

### Changed
- N/A

### Fixed
- N/A

### Security
- N/A

## [0.1.0] - 2024-10-21

### Added
- Initial release with complete handshake implementation
- Packet framing and codec
- NaCl crypto implementation
- Async TCP server with SQLite persistence
- Test client for handshake verification

---

## Version History Template

### [X.Y.Z] - YYYY-MM-DD

#### Added
- New features

#### Changed
- Changes to existing functionality

#### Deprecated
- Soon-to-be removed features

#### Removed
- Removed features

#### Fixed
- Bug fixes

#### Security
- Vulnerability fixes