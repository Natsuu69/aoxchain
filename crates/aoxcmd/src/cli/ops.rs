use crate::cli_support::arg_value;
use crate::data_home;
use crate::economy::ledger::EconomyState;
use crate::node::engine::produce_single_block;
use crate::node::state;
use crate::telemetry::prometheus::MetricsSnapshot;
use crate::telemetry::tracing::TraceProfile;
use aoxcdata::{BlockEnvelope, HybridDataStore, IndexBackend};
use aoxcnet::ports::LIVE_SMOKE_TEST_PORT;
use aoxcnet::transport::live_tcp::run_live_tcp_smoke_on;
use serde::Serialize;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MainnetReadinessControl {
    pub(crate) name: &'static str,
    pub(crate) area: &'static str,
    pub(crate) status: &'static str,
    pub(crate) weight: u8,
    pub(crate) rationale: &'static str,
}

pub(crate) fn cmd_load_benchmark(args: &[String]) -> Result<(), String> {
    let rounds: u64 = arg_value(args, "--rounds")
        .unwrap_or_else(|| "25".to_string())
        .parse()
        .map_err(|_| "--rounds must be a valid u64".to_string())?;
    let tx_per_block: usize = arg_value(args, "--tx-per-block")
        .unwrap_or_else(|| "50".to_string())
        .parse()
        .map_err(|_| "--tx-per-block must be a valid usize".to_string())?;
    let payload_bytes: usize = arg_value(args, "--payload-bytes")
        .unwrap_or_else(|| "256".to_string())
        .parse()
        .map_err(|_| "--payload-bytes must be a valid usize".to_string())?;
    let network_rounds: u64 = arg_value(args, "--network-rounds")
        .unwrap_or_else(|| "10".to_string())
        .parse()
        .map_err(|_| "--network-rounds must be a valid u64".to_string())?;
    let timeout_ms: u64 = arg_value(args, "--timeout-ms")
        .unwrap_or_else(|| "2000".to_string())
        .parse()
        .map_err(|_| "--timeout-ms must be a valid u64".to_string())?;
    if rounds == 0 || tx_per_block == 0 || payload_bytes == 0 {
        return Err("benchmark parameters must be greater than zero".to_string());
    }

    let home = data_home::resolve_data_home(args);
    let mut node = state::setup_with_home(&home).map_err(|error| error.to_string())?;
    let started = Instant::now();
    let mut produced_blocks = 0u64;
    let mut failed_rounds = Vec::new();
    let mut last_height = 0u64;

    for round in 0..rounds {
        let payloads = (0..tx_per_block)
            .map(|tx_index| synthetic_benchmark_payload(round, tx_index, payload_bytes))
            .collect::<Vec<_>>();
        match produce_single_block(&mut node, payloads) {
            Ok(outcome) => {
                produced_blocks += 1;
                last_height = outcome.block.header.height;
            }
            Err(error) => failed_rounds.push(format!("round {}: {}", round + 1, error)),
        }
    }

    let elapsed = started.elapsed();
    let total_txs_attempted = rounds as usize * tx_per_block;
    let total_txs_committed = produced_blocks as usize * tx_per_block;
    let tx_per_sec = if elapsed.as_secs_f64() == 0.0 {
        0.0
    } else {
        total_txs_committed as f64 / elapsed.as_secs_f64()
    };
    let blocks_per_sec = if elapsed.as_secs_f64() == 0.0 {
        0.0
    } else {
        produced_blocks as f64 / elapsed.as_secs_f64()
    };

    let mut network_rtts = Vec::new();
    let network_payload = synthetic_benchmark_payload(0, 0, payload_bytes.min(1024));
    for _ in 0..network_rounds {
        let report = run_live_tcp_smoke_on(
            "127.0.0.1:0",
            &network_payload,
            Duration::from_millis(timeout_ms),
        )
        .map_err(|error| format!("NETWORK_BENCHMARK_ERROR: {error}"))?;
        network_rtts.push(report.round_trip_ms);
    }
    let avg_network_rtt_ms = if network_rtts.is_empty() {
        None
    } else {
        Some((network_rtts.iter().sum::<u128>() / network_rtts.len() as u128) as u64)
    };
    print_json(&serde_json::json!({
        "command": "load-benchmark",
        "scope": "single-process local synthetic benchmark",
        "home": home,
        "configuration": {"rounds": rounds, "tx_per_block": tx_per_block, "payload_bytes": payload_bytes, "network_rounds": network_rounds, "network_timeout_ms": timeout_ms},
        "results": {"elapsed_ms": elapsed.as_millis() as u64, "blocks_requested": rounds, "blocks_produced": produced_blocks, "rounds_failed": failed_rounds.len(), "error_free": failed_rounds.is_empty(), "last_height": last_height, "tx_attempted": total_txs_attempted, "tx_committed": total_txs_committed, "blocks_per_sec": blocks_per_sec, "tx_per_sec": tx_per_sec},
        "network": {"loopback_round_trip_ms": {"min": network_rtts.iter().min().copied(), "max": network_rtts.iter().max().copied(), "avg": avg_network_rtt_ms}},
        "failures": failed_rounds,
        "note": "These numbers represent a local synthetic benchmark, not internet-scale mainnet throughput or adversarial-load certification.",
    }))
}

pub(crate) fn cmd_mainnet_readiness() -> Result<(), String> {
    let controls = mainnet_readiness_controls();
    let total_weight: u32 = controls
        .iter()
        .map(|control| u32::from(control.weight))
        .sum();
    let achieved_weight: u32 = controls
        .iter()
        .filter(|control| control.status == "ready")
        .map(|control| u32::from(control.weight))
        .sum();
    let readiness_percent = if total_weight == 0 {
        0.0
    } else {
        (achieved_weight as f64 / total_weight as f64) * 100.0
    };
    let blockers = controls
        .iter()
        .filter(|control| control.status == "missing")
        .map(|control| format!("{} ({})", control.name, control.area))
        .collect::<Vec<_>>();
    let partials = controls
        .iter()
        .filter(|control| control.status == "partial")
        .map(|control| format!("{} ({})", control.name, control.area))
        .collect::<Vec<_>>();
    print_json(&serde_json::json!({
        "command": "mainnet-readiness",
        "readiness_percent": readiness_percent,
        "grade": readiness_grade(readiness_percent),
        "summary": readiness_summary(readiness_percent),
        "controls": controls,
        "hard_blockers": blockers,
        "partial_gaps": partials,
        "recommendations": [
            "Complete multi-host p2p tests and sustained peer churn recovery.",
            "Add adversarial partition/byzantine/fault-injection suites.",
            "Implement state sync, replay recovery, and snapshot restore validation.",
            "Add long-duration soak tests and public testnet telemetry/SLO dashboards.",
            "Validate real-world latency and throughput on multiple machines before any mainnet claim."
        ],
        "note": "This is an engineering readiness estimate, not a security audit or a guarantee of production safety."
    }))
}

pub(crate) fn mainnet_readiness_controls() -> Vec<MainnetReadinessControl> {
    vec![
        MainnetReadinessControl {
            name: "Deterministic genesis and test fixture",
            area: "bootstrap",
            status: "ready",
            weight: 10,
            rationale: "Deterministic local fixture, funded genesis, and reproducible node homes exist.",
        },
        MainnetReadinessControl {
            name: "Single-node block production path",
            area: "consensus",
            status: "ready",
            weight: 10,
            rationale: "Local block production/finalization path is implemented and covered by tests.",
        },
        MainnetReadinessControl {
            name: "Loopback transport smoke tests",
            area: "network",
            status: "ready",
            weight: 8,
            rationale: "TCP loopback path and repeated local network probes are available.",
        },
        MainnetReadinessControl {
            name: "Storage smoke path",
            area: "data",
            status: "ready",
            weight: 8,
            rationale: "Hybrid block storage smoke flow exists for local verification.",
        },
        MainnetReadinessControl {
            name: "Multi-host peer network validation",
            area: "network",
            status: "missing",
            weight: 15,
            rationale: "No evidence yet of sustained cross-host production-grade p2p validation.",
        },
        MainnetReadinessControl {
            name: "Partition, byzantine, and fault-injection tests",
            area: "resilience",
            status: "missing",
            weight: 15,
            rationale: "Adversarial recovery evidence is not present in the current repo.",
        },
        MainnetReadinessControl {
            name: "State sync and snapshot recovery",
            area: "operations",
            status: "missing",
            weight: 12,
            rationale: "State sync/replay/snapshot recovery needs explicit validation before mainnet.",
        },
        MainnetReadinessControl {
            name: "Long-duration soak and SLO telemetry",
            area: "operations",
            status: "partial",
            weight: 12,
            rationale: "There are runtime/health probes, but no evidence of long-duration audited soak benchmarks.",
        },
        MainnetReadinessControl {
            name: "Official release / attestation controls",
            area: "supply-chain",
            status: "partial",
            weight: 10,
            rationale: "Build attestation surfaces exist, but deployment discipline still depends on release process.",
        },
    ]
}

pub(crate) fn readiness_grade(percent: f64) -> &'static str {
    if percent >= 85.0 {
        "A"
    } else if percent >= 70.0 {
        "B"
    } else if percent >= 55.0 {
        "C"
    } else if percent >= 40.0 {
        "D"
    } else {
        "E"
    }
}
fn readiness_summary(percent: f64) -> &'static str {
    if percent >= 85.0 {
        "Close to production candidate, but still requires external validation."
    } else if percent >= 70.0 {
        "Strong pre-mainnet engineering base with several critical gaps still open."
    } else if percent >= 55.0 {
        "Mid-stage readiness: useful local/system validation exists, but mainnet blockers remain."
    } else {
        "Early-stage readiness: architecture exists, but operational and adversarial evidence is insufficient."
    }
}

pub(crate) fn synthetic_benchmark_payload(
    round: u64,
    tx_index: usize,
    payload_bytes: usize,
) -> Vec<u8> {
    let prefix = format!("AOXC_BENCH_{round}_{tx_index}_");
    let mut payload = prefix.into_bytes();
    while payload.len() < payload_bytes {
        payload.extend_from_slice(b"X");
    }
    payload.truncate(payload_bytes);
    payload
}

pub(crate) fn cmd_node_bootstrap(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let node = state::setup_with_home(&home).map_err(|error| error.to_string())?;
    print_json(&serde_json::json!({
        "mempool_max_txs": node.mempool.config().max_txs,
        "mempool_max_tx_size": node.mempool.config().max_tx_size,
        "validator_count": node.rotation.validators().len(),
        "quorum": {"numerator": node.consensus.quorum.numerator, "denominator": node.consensus.quorum.denominator}
    }))
}

pub(crate) fn cmd_produce_once(args: &[String]) -> Result<(), String> {
    let tx = arg_value(args, "--tx").unwrap_or_else(|| "AOXC_RELAY_DEMO_TX".to_string());
    let home = data_home::resolve_data_home(args);
    let mut node = state::setup_with_home(&home).map_err(|error| error.to_string())?;
    let outcome = produce_single_block(&mut node, vec![tx.into_bytes()])?;
    print_json(
        &serde_json::json!({"height": outcome.block.header.height, "hash": hex::encode(outcome.block.hash), "parent": hex::encode(outcome.block.header.parent_hash), "finalized": outcome.seal.is_some(), "seal": outcome.seal}),
    )
}

pub(crate) fn cmd_node_run(args: &[String]) -> Result<(), String> {
    let rounds: u64 = arg_value(args, "--rounds")
        .unwrap_or_else(|| "10".to_string())
        .parse()
        .map_err(|_| "--rounds must be a valid u64".to_string())?;
    let sleep_ms: u64 = arg_value(args, "--sleep-ms")
        .unwrap_or_else(|| "2000".to_string())
        .parse()
        .map_err(|_| "--sleep-ms must be a valid u64".to_string())?;
    let tx_prefix =
        arg_value(args, "--tx-prefix").unwrap_or_else(|| "AOXC_NODE_RUN_TX".to_string());
    let home = data_home::resolve_data_home(args);
    let mut node = state::setup_with_home(&home).map_err(|error| error.to_string())?;
    let mut produced = 0u64;
    let mut last_height = 0u64;
    let mut failures = Vec::new();
    for round in 0..rounds {
        let tx = format!("{}-{}", tx_prefix, round + 1);
        match produce_single_block(&mut node, vec![tx.into_bytes()]) {
            Ok(outcome) => {
                produced += 1;
                last_height = outcome.block.header.height;
            }
            Err(error) => failures.push(format!("round {}: {}", round + 1, error)),
        }
        if round + 1 < rounds {
            thread::sleep(Duration::from_millis(sleep_ms));
        }
    }
    print_json(
        &serde_json::json!({"mode": "continuous-local-node-run", "rounds_requested": rounds, "rounds_produced": produced, "rounds_failed": failures.len(), "sleep_ms": sleep_ms, "final_height": last_height, "errors": failures}),
    )
}

pub(crate) fn cmd_network_smoke(args: &[String]) -> Result<(), String> {
    let timeout_ms: u64 = arg_value(args, "--timeout-ms")
        .unwrap_or_else(|| "3000".to_string())
        .parse()
        .map_err(|_| "--timeout-ms must be a valid u64".to_string())?;
    let payload = arg_value(args, "--payload")
        .unwrap_or_else(|| "AOXC_LIVE_TCP_PING".to_string())
        .into_bytes();
    let bind_host = arg_value(args, "--bind-host").unwrap_or_else(|| "127.0.0.1".to_string());
    let bind_port: u16 = arg_value(args, "--port")
        .unwrap_or_else(|| LIVE_SMOKE_TEST_PORT.to_string())
        .parse()
        .map_err(|_| "--port must be a valid u16".to_string())?;
    let bind_addr = format!("{bind_host}:{bind_port}");
    let report = run_live_tcp_smoke_on(&bind_addr, &payload, Duration::from_millis(timeout_ms))
        .map_err(|error| format!("NETWORK_LIVE_SMOKE_ERROR: {error}"))?;
    print_json(
        &serde_json::json!({"transport": "tcp", "mode": "live-loopback-socket", "listener": report.listener_addr.to_string(), "bytes_sent": report.bytes_sent, "bytes_received": report.bytes_received, "payload_echoed": report.payload_echoed, "round_trip_ms": report.round_trip_ms}),
    )
}

pub(crate) fn cmd_real_network(args: &[String]) -> Result<(), String> {
    let rounds: u64 = arg_value(args, "--rounds")
        .unwrap_or_else(|| "5".to_string())
        .parse()
        .map_err(|_| "--rounds must be a valid u64".to_string())?;
    let timeout_ms: u64 = arg_value(args, "--timeout-ms")
        .unwrap_or_else(|| "3000".to_string())
        .parse()
        .map_err(|_| "--timeout-ms must be a valid u64".to_string())?;
    let pause_ms: u64 = arg_value(args, "--pause-ms")
        .unwrap_or_else(|| "250".to_string())
        .parse()
        .map_err(|_| "--pause-ms must be a valid u64".to_string())?;
    let payload = arg_value(args, "--payload")
        .unwrap_or_else(|| "AOXC_REAL_NETWORK_PROBE".to_string())
        .into_bytes();
    let bind_host = arg_value(args, "--bind-host").unwrap_or_else(|| "127.0.0.1".to_string());
    let bind_port: u16 = arg_value(args, "--port")
        .unwrap_or_else(|| "0".to_string())
        .parse()
        .map_err(|_| "--port must be a valid u16".to_string())?;
    let bind_addr = format!("{bind_host}:{bind_port}");
    let mut passes = 0u64;
    let mut failures = Vec::new();
    let mut rtts: Vec<u128> = Vec::new();
    for round in 0..rounds {
        match run_live_tcp_smoke_on(&bind_addr, &payload, Duration::from_millis(timeout_ms)) {
            Ok(report) => {
                if report.payload_echoed {
                    passes += 1;
                    rtts.push(report.round_trip_ms);
                } else {
                    failures.push(format!("round {}: payload mismatch", round + 1));
                }
            }
            Err(error) => failures.push(format!("round {}: {}", round + 1, error)),
        }
        if round + 1 < rounds {
            thread::sleep(Duration::from_millis(pause_ms));
        }
    }
    let avg_rtt = if rtts.is_empty() {
        None
    } else {
        Some((rtts.iter().sum::<u128>() / rtts.len() as u128) as u64)
    };
    print_json(
        &serde_json::json!({"command": "real-network", "mode": "multi-round-live-tcp-probe", "rounds_requested": rounds, "rounds_passed": passes, "rounds_failed": failures.len(), "success_ratio": if rounds == 0 { 0.0 } else { passes as f64 / rounds as f64 }, "bind_addr": bind_addr, "timeout_ms": timeout_ms, "pause_ms": pause_ms, "rtt_ms": {"min": rtts.iter().min().copied(), "max": rtts.iter().max().copied(), "avg": avg_rtt}, "failures": failures, "note": "This command validates repeated live TCP behavior. For internet-grade production readiness, run multi-host peer tests with partition/recovery scenarios."}),
    )
}

pub(crate) fn cmd_storage_smoke(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let base_dir =
        arg_value(args, "--base-dir").unwrap_or_else(|| data_home::join(&home, "storage"));
    let backend = arg_value(args, "--index").unwrap_or_else(|| "sqlite".to_string());
    let index_backend = match backend.as_str() {
        "sqlite" => IndexBackend::Sqlite,
        "redb" => IndexBackend::Redb,
        other => {
            return Err(format!(
                "unsupported --index backend: {other}, expected sqlite|redb"
            ));
        }
    };
    let store = HybridDataStore::new(&base_dir, index_backend).map_err(|e| e.to_string())?;
    let block = BlockEnvelope {
        height: 1,
        block_hash_hex: "aa".repeat(32),
        parent_hash_hex: "00".repeat(32),
        payload: b"aoxc-relay-ipfs-block".to_vec(),
    };
    let meta = store.put_block(&block).map_err(|e| e.to_string())?;
    let loaded = store.get_block_by_height(1).map_err(|e| e.to_string())?;
    print_json(
        &serde_json::json!({"base_dir": base_dir, "index_backend": backend, "cid": meta.cid, "height": loaded.height, "payload_len": loaded.payload.len(), "storage_policy": {"block_body": "ipfs(ipld-compatible content addressing)", "state_index": "sqlite_or_redb"}}),
    )
}

pub(crate) fn cmd_economy_init(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
    let treasury_supply: u128 = arg_value(args, "--treasury-supply")
        .unwrap_or_else(|| "1000000000000".to_string())
        .parse()
        .map_err(|_| "--treasury-supply must be a valid u128".to_string())?;
    let mut state = EconomyState::default();
    state.mint_to_treasury(treasury_supply);
    state.save(&state_path)?;
    print_json(
        &serde_json::json!({"state_path": state_path, "treasury_account": state.treasury_account, "treasury_balance": state.treasury_balance(), "total_staked": state.total_staked()}),
    )
}

pub(crate) fn cmd_treasury_transfer(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
    let to = arg_value(args, "--to").ok_or_else(|| "--to is required".to_string())?;
    let amount: u128 = arg_value(args, "--amount")
        .ok_or_else(|| "--amount is required".to_string())?
        .parse()
        .map_err(|_| "--amount must be a valid u128".to_string())?;
    let mut state = EconomyState::load_or_default(&state_path)?;
    let treasury = state.treasury_account.clone();
    state.transfer(&treasury, &to, amount)?;
    state.save(&state_path)?;
    print_json(
        &serde_json::json!({"state_path": state_path, "to": to, "amount": amount, "treasury_balance": state.treasury_balance(), "recipient_balance": state.balances.get(&to).copied().unwrap_or_default()}),
    )
}

pub(crate) fn cmd_stake_delegate(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
    let staker = arg_value(args, "--staker").ok_or_else(|| "--staker is required".to_string())?;
    let validator =
        arg_value(args, "--validator").ok_or_else(|| "--validator is required".to_string())?;
    let amount: u128 = arg_value(args, "--amount")
        .ok_or_else(|| "--amount is required".to_string())?
        .parse()
        .map_err(|_| "--amount must be a valid u128".to_string())?;
    let mut state = EconomyState::load_or_default(&state_path)?;
    state.delegate(&staker, &validator, amount)?;
    state.save(&state_path)?;
    print_json(
        &serde_json::json!({"state_path": state_path, "staker": staker, "validator": validator, "delegated_amount": amount, "total_staked": state.total_staked()}),
    )
}

pub(crate) fn cmd_stake_undelegate(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
    let staker = arg_value(args, "--staker").ok_or_else(|| "--staker is required".to_string())?;
    let validator =
        arg_value(args, "--validator").ok_or_else(|| "--validator is required".to_string())?;
    let amount: u128 = arg_value(args, "--amount")
        .ok_or_else(|| "--amount is required".to_string())?
        .parse()
        .map_err(|_| "--amount must be a valid u128".to_string())?;
    let mut state = EconomyState::load_or_default(&state_path)?;
    state.undelegate(&staker, &validator, amount)?;
    state.save(&state_path)?;
    print_json(
        &serde_json::json!({"state_path": state_path, "staker": staker, "validator": validator, "undelegated_amount": amount, "total_staked": state.total_staked()}),
    )
}

pub(crate) fn cmd_economy_status(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
    let state = EconomyState::load_or_default(&state_path)?;
    print_json(
        &serde_json::json!({"state_path": state_path, "treasury_account": state.treasury_account, "treasury_balance": state.treasury_balance(), "total_accounts": state.balances.len(), "total_staked": state.total_staked(), "positions": state.stakes, "balances": state.balances}),
    )
}

pub(crate) fn cmd_runtime_status(args: &[String]) -> Result<(), String> {
    let profile_arg = arg_value(args, "--trace").unwrap_or_else(|| "standard".to_string());
    let trace_profile = match profile_arg.as_str() {
        "minimal" => TraceProfile::Minimal,
        "standard" => TraceProfile::Standard,
        "verbose" => TraceProfile::Verbose,
        other => {
            return Err(format!(
                "unsupported --trace profile: {other}, expected minimal|standard|verbose"
            ));
        }
    };
    let tps: f64 = arg_value(args, "--tps")
        .unwrap_or_else(|| "0.0".to_string())
        .parse()
        .map_err(|_| "--tps must be a valid f64".to_string())?;
    let peer_count: usize = arg_value(args, "--peers")
        .unwrap_or_else(|| "0".to_string())
        .parse()
        .map_err(|_| "--peers must be a valid usize".to_string())?;
    let error_rate: f64 = arg_value(args, "--error-rate")
        .unwrap_or_else(|| "0.0".to_string())
        .parse()
        .map_err(|_| "--error-rate must be a valid f64".to_string())?;
    let metrics = MetricsSnapshot {
        tps,
        peer_count,
        error_rate,
    };
    print_json(
        &serde_json::json!({"tracing": {"profile": profile_arg, "filter": trace_profile.as_filter()}, "telemetry": {"snapshot": {"tps": metrics.tps, "peer_count": metrics.peer_count, "error_rate": metrics.error_rate}, "prometheus": metrics.to_prometheus()}}),
    )
}

fn print_json(value: &serde_json::Value) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );
    Ok(())
}
