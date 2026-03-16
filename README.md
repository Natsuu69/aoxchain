<div align="center">
  <a href="https://github.com/aoxc/aoxcore">
    <img src="logos/aoxc_transparent.png" alt="AOXCORE Logo" width="180" />
  </a>
</div>

# AOXChain

AOXChain is a **multi-crate Rust blockchain workspace** focused on deterministic behavior, auditability, and operator safety.

This repository contains:
- protocol primitives,
- consensus and networking layers,
- API ingress,
- execution compatibility surfaces,
- operational CLI tooling.

---

## 1) Architecture at a glance

| Domain | Crate(s) | Responsibility |
|---|---|---|
| Core protocol | `aoxcore` | Identity, genesis, transactions, mempool, shared primitives |
| Consensus | `aoxcunity` | Quorum, rounds, vote handling, finalization-related state |
| Networking | `aoxcnet` | Discovery/gossip/sync and transport utilities |
| API ingress | `aoxcrpc` | HTTP/gRPC/WebSocket service entry surfaces |
| Execution compatibility | `aoxcvm` | Lane-based compatibility (EVM/WASM/Move/UTXO-facing adapters) |
| Operator tooling | `aoxcmd`, `aoxckit` | Node bootstrap, runtime commands, key and ops workflows |

Detailed crate map: [`crates/README.md`](crates/README.md)

---

## 2) Quick start (local)

### Prerequisites
- Rust toolchain (stable)
- `cargo`

### Workspace validation

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

### Basic CLI sanity checks

```bash
cargo run -p aoxcmd -- version
cargo run -p aoxcmd -- vision
cargo run -p aoxcmd -- port-map
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- node-run --rounds 5 --sleep-ms 1000 --tx-prefix AOXC_RUN
cargo run -p aoxcmd -- real-network --rounds 5 --timeout-ms 3000 --pause-ms 250
cargo run -p aoxcmd -- interop-readiness
cargo run -p aoxcmd -- key-bootstrap --profile testnet --password "TEST#Secure2026!"
cargo run -p aoxcmd -- interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
```


## 3.1 CLI Security + Telemetry Baseline

`aoxcmd key-bootstrap` now enforces a strong password baseline (minimum 12 chars with upper/lower/digit/symbol classes) before key material is persisted. On Unix-like systems, key bundle, certificate, and passport artifacts are persisted with restrictive `0600` file permissions.

`key-bootstrap` also supports `--profile mainnet|testnet`. The `testnet` profile uses `TEST-` prefixed chain/issuer defaults (for example `TEST-XXX-XX-LOCAL`) so test keys are clearly separated from mainnet-oriented defaults.

For safety, `mainnet` profile key generation now requires explicit opt-in (`--allow-mainnet` or `AOXC_ALLOW_MAINNET_KEYS=true`) to reduce accidental production key creation during local/test runs.

`aoxcmd runtime-status` provides a production-friendly runtime snapshot for tracing profile + Prometheus-formatted telemetry payloads and can be wired into operator dashboards or external scrape bridges.

### Interop Release Gate

Use `interop-gate` for machine-readable release checks. It outputs pass/fail, readiness percentage, and missing controls, and can fail CI with `--enforce`.

Example:

```bash
cargo run -p aoxcmd -- interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
```

`aoxcmd runtime-status` provides a production-friendly runtime snapshot for tracing profile + Prometheus-formatted telemetry payloads and can be wired into operator dashboards or external scrape bridges.

## 3.2 Key Types (Production-Oriented Summary)

AOXChain currently uses a **post-quantum identity path** plus encrypted keyfile persistence:

- **Dilithium3** for identity signatures (`aoxcore::identity::pq_keys`)
- **Argon2id + AES-256-GCM** for secret-key encryption at rest (`aoxcore::identity::keyfile`)
- **Key bootstrap artifacts**: `<name>.key`, `<name>.cert.json`, `<name>.passport.json`

Example strong password for bootstrap:

```text
AOXc#Mainnet2026!
```

---

## 3) Most useful operator commands

> All commands below support `--home <dir>` globally (or `AOXC_HOME`) for data isolation.

### 3.1 Bootstrap and first block

```bash
cargo run -p aoxcmd -- key-bootstrap --profile testnet --password "TEST#Secure2026!"
cargo run -p aoxcmd -- genesis-init
cargo run -p aoxcmd -- node-bootstrap
cargo run -p aoxcmd -- produce-once --tx "hello-aox"
```

### 3.2 Runtime snapshot and release gate

```bash
cargo run -p aoxcmd -- runtime-status --trace standard --tps 12.4 --peers 7 --error-rate 0.001
cargo run -p aoxcmd -- interop-readiness
cargo run -p aoxcmd -- interop-gate --audit-complete true --fuzz-complete true --replay-complete true --finality-matrix-complete true --slo-complete true --enforce
./scripts/quality_gate.sh quick
./scripts/quality_gate.sh full
./scripts/quality_gate.sh release
```


GitHub Actions CI runs:
- quick gate on all PRs
- full gate on pushes to protected branches
- weekly scheduled `cargo audit` security scan (`Security Audit` workflow)

## 4. Production Readiness Note

This repository is under active development. Before production deployment, at minimum complete:

For Turkish go/no-go criteria focused on real-chain operations, see [`docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md`](docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md).

1. Independent security audits (consensus, identity, networking, RPC)
2. Threat modeling and adversarial scenario validation
3. Performance and resilience testing (stress/chaos/partition)
4. Operational runbooks, SLO/SLA targets, and observability policies
5. Release, rollback, and artifact provenance controls

## 5. Repository Map

Detailed crate index: [`crates/README.md`](crates/README.md)

| Path | Responsibility |
|---|---|
| `crates/aoxcore` | Core protocol domain primitives |
| `crates/aoxcunity` | Consensus engine |
| `crates/aoxcnet` | P2P networking layer |
| `crates/aoxcrpc` | API ingress layer |
| `crates/aoxcvm` | Execution compatibility layer |
| `crates/aoxcmd` | Node and operations command surface |
| `crates/aoxckit` | Keyforge/certificate tooling |

## 6. Documentation Policy

README files must remain synchronized with code changes. Any critical behavior update should include a README revision in the same PR.

## 7. License

MIT (`LICENSE`).

## 8. Mainnet/Testnet Operational Playbook (Detailed)

> ⚠️ Important: This repository provides a strong engineering baseline, but **no blockchain deployment can be guaranteed “100% error-free”**. Use staged rollout, audits, and monitored canary deployments.

### 8.1 Build + Binary Packaging

```bash
make quality-quick
make package-bin
```

### 3.3 Continuous local node flow (`node-run`)

```bash
cargo run -p aoxcmd -- node-run --rounds 20 --sleep-ms 1000 --tx-prefix AOXC_RUN
```

What it does:
- produces multiple blocks in sequence,
- sleeps between rounds,
- returns machine-readable JSON summary (`rounds_produced`, `rounds_failed`, `final_height`).

### 3.4 Repeated network probe (`real-network`)

```bash
cargo run -p aoxcmd -- real-network --rounds 10 --timeout-ms 3000 --pause-ms 200 --bind-host 127.0.0.1 --port 0
```

What it does:
- runs repeated live TCP probe rounds,
- reports pass/fail counts,
- reports RTT min/max/avg metrics.

> Important: this is a **probe utility**, not proof of full internet-grade production P2P readiness.

---

## 4) Command reference (aoxcmd)

```text
vision
compat-matrix
port-map
version
key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>]
node-bootstrap
produce-once [--tx <payload>]
node-run [--home <dir>] [--rounds <u64>] [--sleep-ms <u64>] [--tx-prefix <text>]
network-smoke [--timeout-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
real-network [--rounds <u64>] [--timeout-ms <u64>] [--pause-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]
storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
economy-status [--home <dir>] [--state <file>]
runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
interop-readiness
interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
help
```

Language support:
- `--lang <en|tr|es|de>`
- `AOXC_LANG=<code>`

---

## 5) Security notes

- `key-bootstrap` enforces strong password baseline (length + complexity).
- `mainnet` key bootstrap is intentionally guarded and requires explicit opt-in:
  - `--allow-mainnet`, or
  - `AOXC_ALLOW_MAINNET_KEYS=true`
- Key/cert/passport outputs are written with restrictive file permissions on Unix-like systems.

---

## 6) Real-network readiness guidance

For Turkish go/no-go criteria that separate demo-level validation from operational real-chain readiness, see:

- [`docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md`](docs/GERCEK_AG_HAZIRLIK_KRITERLERI_TR.md)

Additional references:
- [`docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md`](docs/RELAY_CHAIN_MAINNET_BLUEPRINT.md)
- [`docs/AUDIT_READINESS_AND_OPERATIONS.md`](docs/AUDIT_READINESS_AND_OPERATIONS.md)

---

## 7) Quality gates and CI helpers

```bash
make quality-quick
make quality
make quality-release
./scripts/quality_gate.sh quick
./scripts/quality_gate.sh full
./scripts/quality_gate.sh release
```

---

## 8) License

MIT (`LICENSE`)
