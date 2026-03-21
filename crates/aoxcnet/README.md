# aoxcnet

## Purpose

`aoxcnet` is the security-first networking plane for AOXChain. The crate is responsible for:

- peer admission and certificate policy enforcement,
- replay-resistant session establishment,
- consensus gossip wrapping,
- discovery and seed rotation,
- sync request scheduling,
- transport abstraction for secure and smoke-test paths,
- metrics and audit-oriented diagnostics.

## Design Principles

- **Mutual trust is explicit.** Peers are admitted under a defined security policy.
- **Replay resistance is mandatory.** Session-bound nonces and frame digests are enforced.
- **Interop is typed.** External domains are represented with explicit identifiers.
- **Deterministic diagnostics matter.** Failures produce stable error codes and metrics.
- **Unsafe modes are explicit.** `Insecure` exists only for deterministic local validation.

## Local Validation

```bash
cargo check -p aoxcnet
cargo test -p aoxcnet -- --nocapture
```
