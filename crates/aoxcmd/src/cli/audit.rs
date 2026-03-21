use crate::build_info::BuildInfo;
use crate::cli_support::{arg_bool_value, arg_flag, arg_value};
use crate::data_home;
use crate::economy::ledger::EconomyState;
use crate::node::state;
use aoxcore::genesis::loader::GenesisLoader;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct InteropAssessment {
    pub(crate) estimated_readiness_percent: f64,
    pub(crate) status: &'static str,
    pub(crate) ready_for_all_chains: bool,
    pub(crate) can_claim_100_percent_security: bool,
    pub(crate) implemented_controls: Vec<&'static str>,
    pub(crate) missing_critical_controls: Vec<&'static str>,
    pub(crate) hard_blockers: Vec<&'static str>,
    pub(crate) next_priority_actions: Vec<&'static str>,
}

pub(crate) fn interop_assessment() -> InteropAssessment {
    InteropAssessment {
        estimated_readiness_percent: 38.0,
        status: "pre-mainnet-hardening",
        ready_for_all_chains: false,
        can_claim_100_percent_security: false,
        implemented_controls: vec![
            "relay-oriented multi-crate architecture",
            "multi-lane execution model (EVM/WASM/Sui Move/Cardano UTXO)",
            "runtime health/readiness and telemetry surfaces",
            "mainnet key generation explicit opt-in guard",
            "production audit CLI surface",
        ],
        missing_critical_controls: vec![
            "independent external security audit with remediation closure",
            "continuous fuzz/property testing for bridge and serialization paths",
            "deterministic replay suite across historical state transitions",
            "multi-node adversarial consensus and partition recovery tests",
            "chain-specific bridge adapter conformance vectors",
            "signed release artifacts, SBOM, and provenance attestation",
        ],
        hard_blockers: vec![
            "No proof that relay logic is safe against all target-chain finality differences",
            "No evidence of completed external audit closure for core/bridge/network paths",
            "No evidence of exhaustive cross-chain compatibility vectors per target family",
        ],
        next_priority_actions: vec![
            "Add 3+ node deterministic adversarial simulation suite",
            "Add replay fixtures and bridge proof failure-injection tests",
            "Add release signing, SBOM generation, and provenance verification",
            "Publish chain-family-specific compatibility matrices and acceptance criteria",
        ],
    }
}

pub(crate) fn cmd_interop_readiness() -> Result<(), String> {
    let assessment = interop_assessment();
    print_json(&serde_json::json!({
        "assessment": {"estimated_readiness_percent": assessment.estimated_readiness_percent, "status": assessment.status, "ready_for_all_chains": assessment.ready_for_all_chains, "can_claim_100_percent_security": assessment.can_claim_100_percent_security},
        "identity": {"key_algorithms": [
            {"name": "Dilithium3", "role": "post-quantum signing for actor identity", "status": "implemented in aoxcore::identity::pq_keys"},
            {"name": "Argon2id + AES-256-GCM keyfile", "role": "password-protected local key material at rest", "status": "implemented in aoxcore::identity::keyfile"}
        ]},
        "execution_lanes": [
            {"lane": "EVM", "priority": "high", "next_step": "RPC and receipt parity test vectors"},
            {"lane": "WASM", "priority": "high", "next_step": "host-call compatibility matrix"},
            {"lane": "Sui Move", "priority": "medium", "next_step": "object/state adapter validation"},
            {"lane": "Cardano UTXO", "priority": "medium", "next_step": "UTXO translator and witness mapping"}
        ],
        "production_checklist": ["cross-chain finality assumptions documented per target chain", "bridge adapter fuzz + property testing", "deterministic serialization and replay tests", "observability SLOs and alerting thresholds", "external security audit for bridge and key lifecycle"],
        "implemented_controls": assessment.implemented_controls,
        "missing_critical_controls": assessment.missing_critical_controls,
        "hard_blockers": assessment.hard_blockers,
        "next_priority_actions": assessment.next_priority_actions
    }))
}

pub(crate) fn cmd_interop_gate(args: &[String]) -> Result<(), String> {
    let audit_complete = arg_bool_value(args, "--audit-complete").unwrap_or(false);
    let fuzz_complete = arg_bool_value(args, "--fuzz-complete").unwrap_or(false);
    let replay_complete = arg_bool_value(args, "--replay-complete").unwrap_or(false);
    let finality_matrix_complete =
        arg_bool_value(args, "--finality-matrix-complete").unwrap_or(false);
    let slo_complete = arg_bool_value(args, "--slo-complete").unwrap_or(false);
    let checks = [
        ("external_security_audit", audit_complete),
        ("bridge_fuzz_property_testing", fuzz_complete),
        ("deterministic_replay_suite", replay_complete),
        ("finality_assumption_matrix", finality_matrix_complete),
        ("observability_slo_alerting", slo_complete),
    ];
    let passed = checks.iter().filter(|(_, ok)| *ok).count();
    let total = checks.len();
    let readiness_percent = ((passed as f64 / total as f64) * 100.0 * 100.0).round() / 100.0;
    let missing: Vec<&str> = checks
        .iter()
        .filter_map(|(name, ok)| if *ok { None } else { Some(*name) })
        .collect();
    let enforce = arg_flag(args, "--enforce");
    let output = serde_json::json!({"pass": missing.is_empty(), "readiness_percent": readiness_percent, "passed_checks": passed, "total_checks": total, "missing_controls": missing});
    print_json(&output)?;
    if enforce && !output["pass"].as_bool().unwrap_or(false) {
        return Err("interop gate failed: missing required controls".to_string());
    }
    Ok(())
}

pub(crate) fn cmd_production_audit(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let genesis_path = arg_value(args, "--genesis")
        .unwrap_or_else(|| data_home::join(&home, "identity/genesis.json"));
    let economy_state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
    let ai_model_signed = arg_bool_value(args, "--ai-model-signed").unwrap_or(false);
    let ai_prompt_guard = arg_bool_value(args, "--ai-prompt-guard").unwrap_or(false);
    let ai_anomaly_detection = arg_bool_value(args, "--ai-anomaly-detection").unwrap_or(false);
    let ai_human_override = arg_bool_value(args, "--ai-human-override").unwrap_or(false);
    let build = BuildInfo::collect();
    let genesis = GenesisLoader::load(&genesis_path).ok();
    let genesis_hash = genesis.as_ref().map(|g| g.config.state_hash());
    let genesis_chain_id = genesis.as_ref().map(|g| g.config.chain_id.clone());
    let genesis_valid = genesis.is_some();
    let economy = EconomyState::load_or_default(&economy_state_path)?;
    let mut stake_by_validator: BTreeMap<String, u128> = BTreeMap::new();
    for position in &economy.stakes {
        *stake_by_validator
            .entry(position.validator.clone())
            .or_insert(0) = stake_by_validator
            .get(&position.validator)
            .copied()
            .unwrap_or(0)
            .saturating_add(position.amount);
    }
    let node = state::setup_with_home(&home).map_err(|error| error.to_string())?;
    let ai_checks = [
        ("model_signature_verification", ai_model_signed),
        ("prompt_injection_guard", ai_prompt_guard),
        ("anomaly_detection_for_ai_paths", ai_anomaly_detection),
        ("human_override_for_high_risk_actions", ai_human_override),
    ];
    let mut recommendations = Vec::new();
    if !genesis_valid {
        recommendations.push(
            "Provide a valid genesis file and verify canonical state_hash before mainnet rollout",
        );
    }
    if build.cert_sha256 == "not-configured" {
        recommendations.push("Embed node certificate fingerprint into build pipeline and enforce startup verification");
    }
    for (name, ok) in ai_checks {
        if !ok {
            recommendations.push(match name {
                "model_signature_verification" => {
                    "Enable cryptographic AI model artifact signature verification"
                }
                "prompt_injection_guard" => "Enable AI prompt injection and jail-break guardrails",
                "anomaly_detection_for_ai_paths" => {
                    "Enable anomaly detection for AI-assisted decision paths"
                }
                "human_override_for_high_risk_actions" => {
                    "Require human override on high-risk AI decisions"
                }
                _ => "Enable missing AI security controls",
            });
        }
    }
    let ai_security_score = ai_control_score(&ai_checks);
    print_json(&serde_json::json!({
        "genesis": {"path": genesis_path, "valid": genesis_valid, "chain_id": genesis_chain_id, "state_hash": genesis_hash},
        "certificates": {"embedded_cert_path": build.cert_path, "embedded_cert_sha256": build.cert_sha256, "embedded_cert_error": build.cert_error},
        "key_security": {"mainnet_key_generation_requires_explicit_opt_in": true, "env_override": "AOXC_ALLOW_MAINNET_KEYS"},
        "ai_security": {"controls": ai_checks, "score": ai_security_score},
        "validator_network": {"configured_validators": node.rotation.validators().len(), "quorum": {"numerator": node.consensus.quorum.numerator, "denominator": node.consensus.quorum.denominator}},
        "treasury_and_staking": {"state_path": economy_state_path, "treasury_account": economy.treasury_account, "treasury_balance": economy.treasury_balance(), "total_staked": economy.total_staked(), "stake_by_validator": stake_by_validator, "positions": economy.stakes},
        "recommendations": recommendations,
    }))
}

pub(crate) fn ai_control_score(controls: &[(&str, bool)]) -> u8 {
    if controls.is_empty() {
        return 0;
    }
    ((controls.iter().filter(|(_, ok)| *ok).count() as f64 / controls.len() as f64) * 100.0).round()
        as u8
}

fn print_json(value: &serde_json::Value) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );
    Ok(())
}
