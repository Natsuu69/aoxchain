# aoxcmd (`aoxc` binary)

## Purpose

Node orchestration command surface and deterministic bootstrap/produce operational workflows.

## Binary name

The CLI is exposed as **`aoxc`**.

From repository root:

```bash
cargo run -p aoxcmd --bin aoxc -- version
```

For production-style local packaging:

```bash
make package-bin
./bin/aoxc version
```

## Production intent

This crate is part of the AOXChain relay-oriented mainnet roadmap. Its interfaces are expected to evolve toward:

- deterministic behavior in consensus-critical paths,
- explicit and typed error surfaces,
- testable integration boundaries with other workspace crates,
- audit-friendly documentation and change control.

## Integration notes

- Keep API changes synchronized with dependent crates in the same pull request.
- For consensus/network/identity touching changes, include tests or deterministic command paths.
- Avoid introducing implicit defaults in critical runtime logic; prefer explicit parameters.
