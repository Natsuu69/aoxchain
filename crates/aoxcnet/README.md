# aoxcnet

## Purpose

`aoxcnet` is responsible for the **networking layer: discovery, gossip, sync, transport** domain within the AOXChain workspace.

## Code Scope

- `discovery/`
- `gossip/`
- `p2p/`
- `sync/`
- `transport/`
- `config.rs`
- `metrics.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxcnet
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
