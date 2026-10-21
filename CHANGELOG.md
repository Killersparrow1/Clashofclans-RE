# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-10-21

### Added
- Protocol version negotiation (v1.0)
- ClientHello extended with max/min supported versions
- ServerHello extended with selected/min versions
- Version compatibility validation
- PROTOCOL.md documentation with full protocol specification

### Changed
- Extended handshake with version negotiation fields
- HandshakeState now tracks negotiated_version

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
