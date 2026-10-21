# Contributing to Clashofclans-RE

Thank you for your interest in contributing! This document outlines the guidelines for contributing to this project.

## 🚀 Getting Started

### Prerequisites
- Rust 1.75+ (`rustup default stable`)
- libsodium development headers (`libsodium-dev` / `libsodium-devel`)

### Development Setup
```bash
# Fork & clone
git clone https://github.com/killersparrow1/Clashofclans-RE
cd Clashofclans-RE

# Run tests to verify setup
cargo test

# Create feature branch
git checkout -b feat/your-feature-name
```

## 📝 Contribution Guidelines

### Code Standards
- **All PRs must pass**:
  - `cargo test` (all 20+ tests)
  - `cargo clippy -- -D warnings`
  - `cargo fmt --check`
- **Follow Rust API Guidelines**: naming, documentation, error handling
- **Use semantic commit messages**: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`

### Crypto/Protocol Changes
- **Any changes to handshake/crypto MUST include tests**
- **Document the reasoning** in commit message and PR description
- **Verify nonce synchronization** with integration test

### Testing Requirements
- **New protocol messages**: Add unit test in `tests/handshake_test.rs`
- **New server handlers**: Add integration test in `clash-of-clans-re-server`
- **Regression tests**: Add for any bug fixes

## 🔄 Pull Request Process

1. **Create focused PR**: One logical change per PR
2. **Write clear description**: What, why, how
3. **Link related issues**: `Fixes #123`, `Relates to #456`
4. **Ensure CI passes**: All checks green
5. **Request review**: Tag maintainers if needed

### PR Checklist
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] Tests added for new functionality
- [ ] Documentation updated (if applicable)
- [ ] Changelog entry added (for user-facing changes)

## 🐛 Issue Reporting

### Bug Reports
Use the bug report template. Include:
- Rust version (`rustc --version`)
- OS/architecture
- Steps to reproduce
- Expected vs actual behavior
- Logs/error messages

### Feature Requests
Use the feature request template. Include:
- Use case / motivation
- Proposed API/design
- Alternatives considered

### Protocol Analysis
For protocol reverse-engineering findings:
- Message ID, direction, payload structure
- Encryption/handshake details
- References to existing documentation

## 🏗️ Architecture Overview

### Crates
| Crate | Purpose |
|-------|---------|
| `clash-of-clans-re-protocol` | Core protocol: framing, codec, crypto, handshake |
| `clash-of-clans-re-server` | Server: gateway, sessions, persistence |
| `clash-of-clans-re-server-bin` | Server binary entry point |
| `clash-of-clans-re-client-test` | Test client for handshake verification |

### Key Files
| File | Purpose |
|------|---------|
| `clash-of-clans-re-protocol/src/handshake.rs` | Handshake state machine |
| `clash-of-clans-re-protocol/src/crypto.rs` | NaCl crypto, key derivation |
| `clash-of-clans-re-server/src/gateway.rs` | TCP gateway, packet pipeline |
| `clash-of-clans-re-server/src/state.rs` | SQLite persistence |

## 🔒 Security

### Reporting Vulnerabilities
**Do not open public issues** for security vulnerabilities.
Report via [GitHub Security Advisory](https://github.com/killersparrow1/Clashofclans-RE/security/advisories/new).

### Scope
- Crypto implementation flaws
- Handshake state machine bugs
- DoS vulnerabilities in packet parsing
- Authentication bypasses

## 📄 License

By contributing, you agree that your contributions will be licensed under the project's dual license (MIT OR Apache-2.0).

## 💬 Questions?

- **GitHub Discussions**: Architecture questions, protocol analysis
- **Issues**: Bug reports, feature requests
- **Security**: Private advisory for vulnerabilities

---

Thank you for contributing to Clashofclans-RE! 🎮