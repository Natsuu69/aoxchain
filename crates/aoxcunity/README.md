# aoxcunity

## Purpose

`aoxcunity` is responsible for the **consensus engine** domain within the AOXChain workspace.

## Code Scope

- `quorum.rs`
- `vote.rs`
- `fork_choice.rs`
- `rotation.rs`
- `proposer.rs`
- `seal.rs`
- `state.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxcunity && cargo test -p aoxcunity
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
