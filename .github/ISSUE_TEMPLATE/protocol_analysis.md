---
name: Protocol Analysis
about: Share reverse engineering findings or protocol documentation
title: "[PROTOCOL] "
labels: ["protocol", "documentation"]
assignees: ""
---

## Protocol Element
**Message ID**: [e.g., 14106 (AttackHome)]
**Direction**: [Client→Server / Server→Client]
**Protocol Version**: [if known]

## Payload Structure
```text
Describe the payload structure:
- Field types (VInt, String, DataReference, etc.)
- Field order and names
- Conditional fields
- Example hex dump
```

## Encryption/Handshake Details
- Is this message encrypted?
- Which handshake stage?
- Nonce behavior

## Related Messages
- Request/Response pairs
- Related message IDs
- Dependencies

## Source
- **Method**: [Static analysis / Dynamic analysis / Traffic capture / Documentation]
- **Tool**: [IDA/Ghidra/Frida/Wireshark/Other]
- **Client Version**: [e.g., 16.253.2]
- **Platform**: [Android/iOS]

## Verification
- [ ] Tested against live client/server
- [ ] Matches existing documentation
- [ ] Test case added

## Additional Context
Any other relevant information, references, or related findings.