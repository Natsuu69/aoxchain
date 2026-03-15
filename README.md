<div align="center">

# 🔷 AOXChain

**Interoperability-first relay chain architecture for deterministic cross-chain coordination.**

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-000000?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Workspace](https://img.shields.io/badge/Workspace-Multi%20Crate-6f42c1)](Cargo.toml)

</div>

---

> ⚠️ **Project status**
>
> AOXChain is under active development. It is **not mainnet-ready** without independent audits,
> economic attack simulation, stress/chaos testing, and long-running production evidence.

## 1) What is AOXChain?

AOXChain is a relay-chain-oriented Rust workspace designed for deterministic cross-chain coordination.

Core focus areas:
- interoperability across heterogeneous chains,
- auditable consensus and identity surfaces,
- multi-lane execution model (EVM, WASM, Sui Move, Cardano adapters),
- operationally testable node workflows,
- audit-readiness and disciplined change management.

## 2) Networking stack (libp2p status)

Current networking in this repository uses **internal gossip/discovery/sync abstractions** in `aoxcnet`.
There is **no direct `libp2p` dependency wired as the active transport layer** in the current codebase.
`aoxcnet` is structured so socket/QUIC transport integration can be expanded later.

## 3) Production readiness gap checklist

To move from dev/test readiness to production readiness, complete at least:

1. **Independent security audits** (consensus, identity, crypto boundaries, networking),
2. **Threat modeling + abuse cases** for P2P, RPC, key lifecycle, and operator workflows,
3. **Supply-chain hardening** (SBOM, signed releases, provenance verification, artifact policy),
4. **Operational hardening** (SLOs, alerting, incident runbooks, backup/restore drills),
5. **Performance and chaos testing** (fault injection, partition tests, replay/reorg scenarios),
6. **Economic security validation** (incentive model, adversarial simulation, stress assumptions),
7. **Release process discipline** (version/tag policy, changelog gates, rollback playbooks).

## 4) Quick start checks

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 5) Build a production-style local binary (`aoxc`)

```bash
# Builds release binary and packages it into ./bin/aoxc
make package-bin

# Run local smoke flow via packaged binary (not cargo run)
make run-local
```

This produces:
- `target/release/aoxc`
- `bin/aoxc`

## 6) Binary provenance and version tracking

`aoxc` now exposes build provenance metadata:
- semantic version,
- git commit hash,
- dirty/clean source state,
- `SOURCE_DATE_EPOCH`,
- optional embedded certificate SHA-256.

```bash
# Build and print provenance metadata
./bin/aoxc version

# Optional: embed a certificate fingerprint at build time
AOXC_EMBED_CERT_PATH=AOXC_DATA/keys/validator-1/certificate.json make package-bin
./bin/aoxc version
# 0) Binary provenance (version + git hash + optional embedded cert digest)
cargo run -p aoxcmd -- version

# Optional: embed a certificate fingerprint at build time
AOXC_EMBED_CERT_PATH=AOXC_DATA/keys/validator-1/certificate.json cargo run -p aoxcmd -- version

# 1) Vision summary
cargo run -p aoxcmd -- vision

# 2) Generate genesis
cargo run -p aoxcmd -- genesis-init \
  --path AOXC_DATA/identity/genesis.json \
  --chain-num 1 \
  --block-time 6 \
  --treasury 1000000000

# 3) Key + identity bootstrap
cargo run -p aoxcmd -- key-bootstrap \
  --password "change-me" \
  --base-dir AOXC_DATA/keys \
  --name validator-1 \
  --chain AOXC-MAIN \
  --role validator \
  --zone core \
  --issuer AOXC-ROOT-CA \
  --validity-secs 31536000

# 4) Node bootstrap
cargo run -p aoxcmd -- node-bootstrap

# 5) Produce a deterministic single block
cargo run -p aoxcmd -- produce-once --tx "relay-coordination-demo"

# 6) Network smoke
cargo run -p aoxcmd -- network-smoke

# 7) Storage smoke
cargo run -p aoxcmd -- storage-smoke --index sqlite
cargo run -p aoxcmd -- storage-smoke --index redb

# 8) Economy bootstrap (treasury + staking)

# 8) Ekonomi bootstrap (hazine + stake)
cargo run -p aoxcmd -- economy-init --treasury-supply 1000000000000
cargo run -p aoxcmd -- treasury-transfer --to validator-1 --amount 500000000
cargo run -p aoxcmd -- stake-delegate --staker validator-1 --validator val-core-1 --amount 250000000
cargo run -p aoxcmd -- economy-status
```

## 7) Operator commands (`aoxc`)

```bash
./bin/aoxc vision
./bin/aoxc compat-matrix
./bin/aoxc genesis-init --path AOXC_DATA/identity/genesis.json --chain-num 1 --block-time 6 --treasury 1000000000
./bin/aoxc key-bootstrap --password "change-me" --base-dir AOXC_DATA/keys --name validator-1 --chain AOXC-MAIN --role validator --zone core --issuer AOXC-ROOT-CA --validity-secs 31536000
./bin/aoxc node-bootstrap
./bin/aoxc produce-once --tx "relay-coordination-demo"
./bin/aoxc network-smoke
./bin/aoxc storage-smoke --index sqlite
./bin/aoxc economy-init --treasury-supply 1000000000000
./bin/aoxc treasury-transfer --to validator-1 --amount 500000000
./bin/aoxc stake-delegate --staker validator-1 --validator val-core-1 --amount 250000000
./bin/aoxc economy-status
```

## 8) Repository map

| Path | Purpose |
|---|---|
| `crates/aoxcore` | Core domain primitives (identity, tx, genesis, mempool) |
| `crates/aoxcunity` | Consensus core (quorum, vote, proposer rotation, fork-choice, seal) |
| `crates/aoxcvm` | Multi-lane execution compatibility layer |
| `crates/aoxcnet` | Gossip/discovery/sync network shell |
| `crates/aoxcrpc` | HTTP / gRPC / WebSocket RPC entry layer |
| `crates/aoxcmd` | Node orchestration and deterministic operator commands |
| `crates/aoxckit` | Keyforge and operational crypto tooling |
| `crates/aoxcsdk` | SDK surface for integration developers |
| `docs/` | Architecture, audit readiness, operations, risk docs |

Detailed crate index: **[`crates/README.md`](crates/README.md)**

## 9) Dev/testnet references

- Local script: [`scripts/run-local.sh`](scripts/run-local.sh)
- Config profiles: [`configs/mainnet.toml`](configs/mainnet.toml), [`configs/testnet.toml`](configs/testnet.toml), [`configs/genesis.json`](configs/genesis.json)
- Container setup: [`Dockerfile`](Dockerfile), [`docker-compose.yaml`](docker-compose.yaml)

## 10) Documentation hub

### Operations + audit
- [`docs/AUDIT_READINESS_AND_OPERATIONS.md`](docs/AUDIT_READINESS_AND_OPERATIONS.md)
- [`docs/P2P_AUDIT_GUIDE_EN.md`](docs/P2P_AUDIT_GUIDE_EN.md)

### Architecture + roadmap
- [`docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md`](docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md)
- [`docs/TEKNIK_DERIN_ANALIZ_TR.md`](docs/TEKNIK_DERIN_ANALIZ_TR.md)
- [`docs/REPO_GAP_ANALIZI_TR.md`](docs/REPO_GAP_ANALIZI_TR.md)

### Responsible use + risk notice
- [`docs/SECURITY_AND_RISK_NOTICE_TR.md`](docs/SECURITY_AND_RISK_NOTICE_TR.md)

## 11) Contribution and security discipline

- Changes touching consensus/identity/networking must include tests.
- Keep linting clean (`clippy -D warnings`).
- For large changes, include design notes, threat model updates, and rollback plans.
- Keep key material, certificates, and sensitive artifacts under strict operational controls.

## 12) License

MIT (`LICENSE`).
