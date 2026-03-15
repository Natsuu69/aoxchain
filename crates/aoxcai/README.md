# aoxcai

## Purpose

`aoxcai` is responsible for the **AI policy/engine integration layer** domain within the AOXChain workspace.

## Code Scope

- `backend/`
- `policy/`
- `engine.rs`
- `registry.rs`
- `traits.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxcai
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
