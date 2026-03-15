# aoxcmd

## Purpose

Node orchestration command surface and deterministic bootstrap/produce operational workflows.

## Production Intent

This crate is part of the AOXChain relay-oriented mainnet roadmap. Its interfaces are expected to evolve toward:

- deterministic behavior in consensus-critical paths,
- explicit and typed error surfaces,
- testable integration boundaries with other workspace crates,
- audit-friendly documentation and change control.

## Local Development

From repository root:

```bash
cargo check -p aoxcmd
```

## Integration Notes

- Keep API changes synchronized with dependent crates in the same pull request.
- For consensus/network/identity touching changes, include tests or deterministic command paths.
- Avoid introducing implicit defaults in critical runtime logic; prefer explicit parameters.

## Extended CLI (Economy + Compatibility)

- `compat-matrix`: lane/transport compatibility görünümü.
- `economy-init`: hazine bakiyesi ile ekonomi durumunu başlatır.
- `treasury-transfer`: hazineden hesaba transfer yapar.
- `stake-delegate` / `stake-undelegate`: stake pozisyonu yönetir.
- `economy-status`: bakiye ve stake özetini döndürür.
