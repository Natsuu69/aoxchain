# AOXCNET

AOXCNET is the networking surface of AOXChain. It provides the peer identity,
secure session, and gossip routing abstractions used by node orchestration.

## Security-oriented design goals

1. **Mutual-auth by default** (`SecurityMode::MutualAuth`).
2. **Certificate validity enforcement** during peer admission.
3. **Session-ticket gating** before secure message broadcast.
4. **Audit-strict mode** (`SecurityMode::AuditStrict`) for production policy gates.

## Core modules

- `config`: network + security profile settings.
- `gossip::peer`: peer identity and certificate fingerprinting.
- `p2p`: secure admission/session model and transport shell.
- `gossip::consensus_gossip`: consensus message propagation API for node layer.

## CLI compatibility

`aoxcmd network-smoke` remains backward compatible and can run without a live
transport backend. Secure p2p session flow is available for deterministic
validation by registering peers + establishing sessions in process.

## Audit usage checklist

- Enforce `SecurityMode::AuditStrict` in production configs.
- Reject expired or not-yet-valid certificates.
- Verify session count and peer inventory from telemetry.
- Add integration tests for replay and partition scenarios before mainnet.
