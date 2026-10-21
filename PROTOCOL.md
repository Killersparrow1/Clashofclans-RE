# Clashofclans-RE Protocol Specification

Version: 1.0  
Last Updated: 2024

---

## Overview

This document describes the Clash of Clans networking protocol as implemented in Clashofclans-RE.

### Protocol Basics

- **Transport**: TCP (port 9339)
- **Encryption**: NaCl/libsodium (ChaCha20-Poly1305)
- **Byte Order**: Big-endian (network order)
- **Packet Format**: Fixed 7-byte header + variable payload

---

## Packet Format

```
+00: u16   Message ID (big-endian)
+02: u24   Payload Length (big-endian, 3 bytes)
+05: u16   Version (usually 0)
+07: []    Encrypted Payload
```

### Header Details

| Field | Size | Description |
|-------|------|-------------|
| Message ID | 2 bytes | Identifies message type |
| Payload Length | 3 bytes | Length of encrypted payload (max 16MB) |
| Version | 2 bytes | Protocol version (typically 0) |

### Encryption

After handshake completion, all packets are encrypted using NaCl `crypto_box` (ChaCha20-Poly1305):

- **Nonce**: 24 bytes, incremented by 2 per packet
- **Separate nonces** for client→server (snonce) and server→client (rnonce)
- **Key**: Derived during handshake via ECDH

---

## Handshake Protocol

### Phase 1: ClientHello (10100)

Client → Server (unencrypted)

```
ClientHello {
    unknown1: u32 = 0x07896541
    unknown2: u32 = 0
    unknown3: u32 = 0
    unknown4: u32 = 1
    unknown5: u32 = 0
    unknown6: u32 = 0
    unknown7: u32 = 0xFD000000
    client_seed: [u8; 24]     // Random
    unknown8: u32 = 1
    unknown9: u32 = 0
    unknown10: u32 = 0
    unknown11: u32 = 0
    unknown12: u32 = 0
    unknown13: u32 = 0
    unknown13: u32 = 1
    device_model: string      // e.g., "iPad5,3"
    locale: string            // e.g., "en-US"
    unknown14: u32 = 6
    os_version: string        // e.g., "15.8.2"
    padding: [u32; 5] = 0
}
```

### Phase 2: ServerHello / Encryption (20000)

Server → Client (unencrypted)

```
ServerHello {
    server_random: [u8; 24]   // Random
}
```

### Phase 3: Login (10101)

Client → Server (encrypted with long-term shared key)

```
Login {
    server_random: [u8; 24]   // Echo from ServerHello
    client_nonce: [u8; 24]    // Client's ephemeral nonce
    client_pk: [u8; 32]       // Client's ephemeral public key
    encrypted_payload: []     // Encrypted with long-term shared key
}
```

**Long-term shared key derivation:**
```
shared_key = precompute(server_longterm_pk, client_ephemeral_sk)
```

### Phase 4: Encryption Response (20000)

Server → Client (encrypted with long-term shared key)

```
EncryptionResponse {
    server_nonce: [u8; 24]    // Server's ephemeral nonce
    session_pk: [u8; 32]      // Server's session public key
}
```

### Phase 5: Session Key Establishment

Both parties derive session shared key:

**Client:**
```
session_shared = precompute(session_pk, client_ephemeral_sk)
```

**Server:**
```
session_shared = precompute(client_pk, session_sk)
```

### Phase 6: Encrypted Channel

All subsequent packets use `session_shared` with incrementing nonces.

---

## Message IDs Reference

### Client → Server (10000-19999)

| ID | Name | Description |
|----|------|-------------|
| 10100 | ClientHello | Initial handshake packet |
| 10101 | Login | Login with credentials |
| 10102 | LoginUsingSession | Resume existing session |
| 10103 | CreateAccount | Register new account |
| 10107 | ClientCapabilities | Device capabilities |
| 10108 | KeepAlive | Heartbeat |
| 10112 | AuthenticationCheck | Auth verification |
| 10113 | SetDeviceToken | Push notification token |
| 10116 | ResetAccount | Reset account data |
| 10117 | ReportUser | Report abusive user |
| 10118 | AccountSwitched | Account switch notification |
| 10150 | AppleBillingRequest | iOS purchase |
| 10151 | GoogleBillingRequest | Android purchase |
| 10200 | CreateAvatar | Create new player |
| 10201 | SelectAvatar | Select existing player |
| 10206 | SendChatToAvatar | Private chat |
| 10212 | ChangeAvatarName | Rename player |
| 10501 | AcceptFriend | Accept friend request |
| 10502 | AddFriend | Send friend request |
| 10503 | AskForAddableFriends | Get friend suggestions |
| 10504 | AskForFriendList | Get friend list |
| 10506 | RemoveFriend | Remove friend |
| 10507 | AddFriendByEmail | Add friend via email |
| 10509 | AddFriendByAvatarNameAndCode | Add friend by name+code |
| 10512 | AskForPlayingGamecenterFriends | GameCenter friends |
| 10513 | AskForPlayingFacebookFriends | Facebook friends |
| 10901 | AskForMailList | Get mail |
| 10904 | TakeMailAttachments | Claim mail rewards |
| 14101 | AttackResult | Battle result |
| 14102 | EndClientTurn | End turn (async battles) |
| 14104 | AskForTargetHomeList | Get attack targets |
| 14106 | AttackHome | Initiate attack |
| 14108 | ChangeHomeName | Rename village |
| 14113 | VisitHome | Visit another village |
| 14114 | HomeBattleReplay | Request replay |
| 14123 | AttackMatchedHome | Multiplayer attack |
| 14134 | AttackNpc | Single-player attack |
| 14201 | BindFacebookAccount | Link Facebook |
| 14211 | UnbindFacebookAccount | Unlink Facebook |
| 14212 | BindGamecenterAccount | Link GameCenter |
| 14262 | BindGoogleServiceAccount | Link Google Play |
| 14301 | CreateAlliance | Create clan |
| 14302 | AskForAllianceData | Get clan info |
| 14303 | AskForJoinableAlliancesList | Search clans |
| 14305 | JoinAlliance | Join clan |
| 14306 | ChangeAllianceMemberRole | Change member rank |
| 14307 | KickAllianceMember | Kick member |
| 14308 | LeaveAlliance | Leave clan |
| 14309 | AskForAllianceUnitDonations | Request troops |
| 14310 | DonateAllianceUnit | Donate troops |
| 14315 | ChatToAllianceStream | Clan chat |
| 14316 | ChangeAllianceSettings | Modify clan settings |
| 14317 | RequestJoinAlliance | Request to join |
| 14321 | RespondToAllianceJoinRequest | Accept/reject request |
| 14322 | SendAllianceInvitation | Invite to clan |
| 14323 | JoinAllianceUsingInvitation | Join via invite |
| 14324 | SearchAlliances | Search clans |
| 14325 | AskForAvatarProfile | Get player profile |
| 14330 | SendAllianceMail | Clan mail |
| 14331 | HomeShareReplay | Share replay |
| 14401 | AskForAllianceRankingList | Clan leaderboard |
| 14403 | AskForAvatarRankingList | Player leaderboard |
| 14404 | AskForAvatarLocalRankingList | Local leaderboard |
| 14405 | AskForAvatarStream | Activity feed |
| 14418 | RemoveAvatarStreamEntry | Delete feed entry |
| 14503 | AskForLeagueMemberList | League members |
| 14715 | SendGlobalChatLine | Global chat |
| 16000 | LogicDeviceLinkCodeRequest | Device link code |
| 16001 | LogicDeviceLinkMenuClosed | Device link closed |
| 16002 | LogicDeviceLinkEnterCode | Enter link code |
| 16003 | LogicDeviceLinkConfirmYes | Confirm link |

### Server → Client (20000+)

| ID | Name | Description |
|----|------|-------------|
| 20000 | Encryption | Handshake encryption packets |
| 20101 | CreateAccountResult | Account creation result |
| 20103 | LoginFailed | Login failure |
| 20104 | LoginOk | Login success |
| 20105 | FriendList | Friend list |
| 20106 | FriendListUpdate | Friend list delta |
| 20107 | AddableFriends | Friend suggestions |
| 20108 | AddFriendFailed | Friend request failed |
| 20109 | FriendOnlineStatus | Friend status change |
| 20110 | FriendLoggedIn | Friend login notification |
| 20111 | FriendLoggedOut | Friend logout notification |
| 20117 | ReportUserStatus | Report result |
| 20118 | ChatAccountBanStatus | Chat ban status |
| 20121 | BillingRequestFailed | Purchase failed |
| 20151 | AppleBillingProcessedByServer | iOS purchase processed |
| 20152 | GoogleBillingProcessedByServer | Android purchase processed |
| 20161 | ShutdownStarted | Server shutdown notice |
| 20171 | PersonalBreakStarted | Personal break started |
| 20201 | AvatarData | Player data |
| 20202 | CreateAvatarFailed | Avatar creation failed |
| 20203 | CreateAvatarOk | Avatar created |
| 20205 | AvatarNameChangeFailed | Rename failed |
| 20801 | Notification | General notification |
| 20903 | MailList | Mail list |
| 24101 | OwnHomeData | Full village data |
| 24103 | AttackHomeFailed | Attack failed |
| 24104 | OutOfSync | State desync |
| 24105 | TargetHomeList | Attack targets |
| 24106 | AttackReportList | Battle logs |
| 24107 | EnemyHomeData | Target village data |
| 24109 | HomeStatusList | Village status |
| 24111 | AvailableServerCommand | Server commands |
| 24112 | WaitingToGoHome | Return to village |
| 24113 | VisitedHomeData | Visited village |
| 24114 | HomeBattleReplayData | Replay data |
| 24115 | ServerError | Server error |
| 24116 | HomeBattleReplayFailed | Replay failed |
| 24133 | NpcData | NPC village data |
| 24201 | FacebookAccountBound | Facebook linked |
| 24202 | FacebookAccountAlreadyBound | Already linked |
| 24211 | GamecenterAccountBound | GameCenter linked |
| 24212 | GamecenterAccountAlreadyBound | Already linked |
| 24214 | FacebookAccountUnbound | Facebook unlinked |
| 24261 | GoogleServiceAccountBound | Google Play linked |
| 24262 | GoogleServiceAccountAlreadyBound | Already linked |
| 24301 | AllianceData | Clan data |
| 24302 | AllianceJoinFailed | Join failed |
| 24303 | AllianceJoinOk | Join success |
| 24304 | JoinableAllianceList | Search results |
| 24305 | AllianceLeaveOk | Left clan |
| 24306 | ChangeAllianceMemberRoleOk | Role changed |
| 24307 | KickAllianceMemberOk | Member kicked |
| 24308 | AllianceMember | Member info |
| 24309 | AllianceMemberRemoved | Member removed |
| 24310 | AllianceList | Clan list |
| 24311 | AllianceStream | Clan chat stream |
| 24312 | AllianceStreamEntry | Chat entry |
| 24318 | AllianceStreamEntryRemoved | Chat deleted |
| 24319 | AllianceJoinRequestOk | Request sent |
| 24320 | AllianceJoinRequestFailed | Request failed |
| 24321 | AllianceInvitationSendFailed | Invite failed |
| 24322 | AllianceInvitationSentOk | Invite sent |
| 24332 | AllianceCreateFailed | Creation failed |
| 24333 | AllianceChangeFailed | Settings failed |
| 24334 | AvatarProfile | Player profile |
| 24401 | AllianceRankingList | Clan rankings |
| 24403 | AvatarRankingList | Player rankings |
| 24404 | AvatarLocalRankingList | Local rankings |
| 24411 | AvatarStream | Activity stream |
| 24412 | AvatarStreamEntry | Stream entry |
| 24418 | AvatarStreamEntryRemoved | Entry removed |
| 24503 | LeagueMemberList | League members |
| 24715 | GlobalChatLine | Global chat message |
| 25892 | Disconnected | Disconnect notice |
| 26002 | LogicDeviceLinkCodeResponse | Link code response |
| 26003 | LogicDeviceLinkNewDeviceLinked | Device linked |
| 26004 | LogicDeviceLinkCodeDeactivated | Code expired |
| 26005 | LogicDeviceLinkResponse | Link response |
| 26007 | LogicDeviceLinkDone | Link complete |
| 26008 | LogicDeviceLinkError | Link error |

---

## Data Types Encoding

### Variable-Length Integer (VInt)
- Zigzag encoding for signed integers
- 7 bits per byte, MSB = continuation flag

### Variable-Length Long (VLong)
- Same as VInt but 64-bit

### Strings
- Length-prefixed UTF-8 (VInt length + bytes)

### Data Reference
- Two VInts: (table_id, instance_id)

### LogicLong
- Two i32: (high, low) for 64-bit IDs

### Booleans
- Single byte (0/1), packed for consecutive bools

### Byte Arrays
- Length-prefixed (VInt length + bytes)

---

## Protocol Version Negotiation

### Current Implementation
- Version field in header is typically `0`
- No formal negotiation - both sides assume compatible version

### Proposed Version Negotiation (Future)

**ClientHello Extension:**
```
ClientHello {
    ...
    supported_versions: [u16]  // List of supported protocol versions
}
```

**ServerHello Extension:**
```
ServerHello {
    server_random: [u8; 24],
    selected_version: u16,      // Highest mutually supported
    min_version: u16,           // Minimum required version
}
```

**Version Format:**
- Major.Minor (u16: major << 8 | minor)
- Example: v1.2 = 0x0102

**Compatibility Rules:**
- Same major version = compatible
- Server rejects if client's max < server's min
- Unknown fields ignored for forward compatibility

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 20103 | LoginFailed | Invalid credentials/version |
| 24103 | AttackHomeFailed | Attack validation failed |
| 24104 | OutOfSync | Client/server state mismatch |
| 24302 | AllianceJoinFailed | Clan full/banned/etc |
| 24332 | AllianceCreateFailed | Invalid name/taken |
| 24333 | AllianceChangeFailed | Invalid settings |

---

## Security Considerations

1. **Replay Protection**: Nonces incremented by 2, never reused
2. **Forward Secrecy**: Session keys ephemeral, long-term keys only for handshake
3. **Integrity**: ChaCha20-Poly1305 provides authenticated encryption
4. **Version Downgrade**: Server should reject obsolete versions

---

## Implementation Notes

### Crate: `clash-of-clans-re-protocol`
- `framing.rs`: Packet framing/buffering
- `codec.rs`: Data type serialization
- `crypto.rs`: NaCl crypto operations
- `handshake.rs`: Handshake state machine
- `messages.rs`: 151 message IDs

### Crate: `clash-of-clans-re-server`
- `gateway.rs`: TCP server, packet pipeline
- `session.rs`: Session management
- `state.rs`: SQLite persistence

### Testing
```bash
# Run all tests
cargo test --workspace

# Run integration handshake test
cargo test -p clash-of-clans-re-protocol test_full_handshake
```

---

## References

- [clanner/cocdp](https://github.com/clanner/cocdp) - Community protocol docs
- [Ultrapowa Clash Server](https://github.com/ogclash/ultrapower-clash-server) - C# reference
- [magic-rs](https://github.com/thexeondev/magic-rs) - libg.so bindings
- [Giovanni Rocca's RE series](http://www.giovanni-rocca.com/reverse-engineering-supercell-part-4/)