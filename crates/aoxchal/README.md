# aoxchal

## Purpose

`aoxchal` is responsible for the **hardware abstraction (HAL) support utilities** domain within the AOXChain workspace.

## Code Scope

- `cpu_opt.rs`
- `mem_manager.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxchal
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
