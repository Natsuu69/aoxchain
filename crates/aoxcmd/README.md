# aoxcmd

## Purpose

`aoxcmd` is responsible for the **node lifecycle and operations CLI** domain within the AOXChain workspace.

## Code Scope

- `app/`
- `node/`
- `runtime/`
- `telemetry/`
- `economy/`
- `main.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxcmd
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
