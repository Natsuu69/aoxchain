# aoxcvm

## Purpose

`aoxcvm` is responsible for the **multi-VM compatibility and lane routing layer** domain within the AOXChain workspace.

## Code Scope

- `compatibility/`
- `routing/`
- `host/`
- `system/`
- `lanes/`
- `context.rs`
- `gas.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxcvm && cargo test -p aoxcvm
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
