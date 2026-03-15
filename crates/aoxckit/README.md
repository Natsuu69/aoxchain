# aoxckit

## Purpose

`aoxckit` is responsible for the **keyforge and operational cryptography tooling** domain within the AOXChain workspace.

## Code Scope

- `keyforge/cmd_key.rs`
- `keyforge/cmd_cert.rs`
- `keyforge/cmd_passport.rs`
- `keyforge/cmd_zkp_setup.rs`
- `main.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxckit
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
