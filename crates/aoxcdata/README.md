# aoxcdata

## Purpose

`aoxcdata` is responsible for the **storage layer (KV + state tree)** domain within the AOXChain workspace.

## Code Scope

- `store/kv_db.rs`
- `store/state_tree.rs`
- `store/column_families.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxcdata && cargo test -p aoxcdata
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
