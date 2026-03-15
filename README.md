# AOXChain

AOXChain is being designed as an **interoperable relay-oriented coordination chain**, not as a pure throughput-first monolith. The strategic objective is to provide deterministic coordination, cross-chain compatibility, and robust identity/consensus primitives that can interoperate with heterogeneous execution ecosystems.

> Status: pre-mainnet engineering. Workspace compiles, CLI smoke path is operational, and mainnet-hardening tracks are documented.

## Strategic Vision

AOXChain is intended to:

1. Operate as a **relay and coordination layer** across multiple chains.
2. Prioritize **determinism, compatibility, and auditability** over short-term TPS optimization.
3. Support **future-proof identity and trust surfaces** (post-quantum capable key/cert/passport pipeline).
4. Provide a **multi-lane architecture** for heterogeneous contract/runtime ecosystems.

## Workspace Topology

- `aoxcore`: identity, genesis, transaction hashing, mempool.
- `aoxcunity`: consensus, quorum, proposer rotation, fork-choice, finalization surfaces.
- `aoxcvm`: lane execution compatibility abstractions.
- `aoxcnet`: networking/gossip/discovery shell.
- `aoxcmd`: operational CLI for bootstrap and deterministic smoke execution.
- `aoxcrpc`, `aoxcsdk`, `aoxckit`, and others: integration/tooling layers.

## Build and Core Validation

```bash
cargo check --workspace
cargo test -p aoxcmd
cargo test -p aoxcunity
```

## End-to-End CLI Bootstrap (Current Deterministic Path)

### 1) Chain vision introspection

```bash
cargo run -p aoxcmd -- vision
```

### 2) Genesis creation

```bash
cargo run -p aoxcmd -- genesis-init \
  --path AOXC_DATA/identity/genesis.json \
  --chain-num 1 \
  --block-time 6 \
  --treasury 1000000000
```

### 3) Key + identity material bootstrap

```bash
cargo run -p aoxcmd -- key-bootstrap \
  --password "change-me" \
  --base-dir AOXC_DATA/keys \
  --name relay-1 \
  --chain AOXC-MAIN \
  --role relay \
  --zone global \
  --issuer AOXC-ROOT-CA \
  --validity-secs 31536000
```

### 4) Node bootstrap validation

```bash
cargo run -p aoxcmd -- node-bootstrap
```

### 5) Produce and finalize one block (deterministic smoke)

```bash
cargo run -p aoxcmd -- produce-once --tx "relay-coordination-demo"
```

### 6) Network integration stub check

```bash
cargo run -p aoxcmd -- network-smoke
```

## Mainnet Hardening Priorities

- Transport-backed gossip and peer routing in `aoxcnet`.
- Multi-node integration tests (`proposal -> vote -> finalize` lifecycle).
- RPC/runtime persistent-state integration.
- Threat modeling, adversarial simulation, and external audit report.
- Release attestation and reproducible build pipeline.

## Engineering Documents

- `docs/AUDIT_READINESS_AND_OPERATIONS.md`
- `docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md`

## License

MIT (`LICENSE`).
