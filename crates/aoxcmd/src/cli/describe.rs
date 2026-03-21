use crate::build_info::BuildInfo;
use crate::cli::AOXC_RELEASE_NAME;
use crate::cli_support::arg_flag;
use aoxcnet::ports::{PORT_BINDINGS, RPC_HTTP_PORT};
use aoxcore::protocol::{
    canonical_chain_families, canonical_message_envelope_fields, canonical_modules,
    canonical_sovereign_roots,
};

pub(crate) fn cmd_version() -> Result<(), String> {
    let build = BuildInfo::collect();
    let output = version_payload(&build);
    print_json(&output)
}

pub(crate) fn version_payload(build: &BuildInfo) -> serde_json::Value {
    serde_json::json!({
        "name": "aoxcmd",
        "release_name": AOXC_RELEASE_NAME,
        "version": build.semver,
        "git_commit": build.git_commit,
        "git_dirty": build.git_dirty,
        "source_date_epoch": build.source_date_epoch,
        "build_profile": build.build_profile,
        "release_channel": build.release_channel,
        "attestation_hash": build.attestation_hash,
        "embedded_cert": {
            "path": build.cert_path,
            "sha256": build.cert_sha256,
            "error": build.cert_error,
        }
    })
}

pub(crate) fn cmd_build_manifest() -> Result<(), String> {
    let build = BuildInfo::collect();
    print_json(&build_manifest_payload(&build))
}

pub(crate) fn cmd_node_connection_policy(args: &[String]) -> Result<(), String> {
    let build = BuildInfo::collect();
    let enforce = arg_flag(args, "--enforce-official");
    let official_release = is_official_release(&build);
    print_json(&node_connection_policy_payload(&build))?;
    if enforce && !official_release {
        return Err(
            "official node policy failed: build is not an official release artifact".to_string(),
        );
    }
    Ok(())
}

pub(crate) fn cmd_vision() -> Result<(), String> {
    let sovereign_roots: Vec<&str> = canonical_sovereign_roots()
        .iter()
        .map(|root| root.as_str())
        .collect();
    let attached_modules: Vec<&str> = canonical_modules()
        .iter()
        .map(|module| module.as_str())
        .collect();
    let output = serde_json::json!({
        "release_name": AOXC_RELEASE_NAME,
        "chain_positioning": "interop relay-oriented coordination chain",
        "primary_goal": "cross-chain compatibility and deterministic coordination over raw throughput",
        "execution_strategy": "sovereign constitutional local core + remote execution domains",
        "recommended_topology": "thin relay core + five attached functional modules",
        "constitutional_roots": sovereign_roots,
        "functional_modules": attached_modules,
        "identity_model": "post-quantum capable key/certificate/passport pipeline",
        "consensus_model": "quorum-based proposer/vote/finalization with explicit rotation",
        "status": "pre-mainnet; deterministic local smoke path available"
    });
    print_json(&output)
}

pub(crate) fn cmd_sovereign_core() -> Result<(), String> {
    let sovereign_roots: Vec<&str> = canonical_sovereign_roots()
        .iter()
        .map(|root| root.as_str())
        .collect();
    let output = serde_json::json!({
        "local_chain_role": "sovereign constitutional core",
        "remote_chain_role": "execution domains connected through contracts and settlement rules",
        "constitutional_roots": sovereign_roots,
        "local_must_keep": {
            "identity": ["root_account_registry","chain_mappings","signer_bindings","recovery_authority","key_rotation_rules","delegate_registry"],
            "supply": ["total_canonical_supply","mint_authority_root","burn_settlement_root","global_supply_accounting"],
            "governance": ["constitution","validator_admission_rules","parameter_change_authority","upgrade_authority","emergency_controls"],
            "relay": ["message_routing","checkpointing","light_client_commitments","cross_chain_ordering","fee_metering"],
            "security": ["slash_conditions","threat_levels","policy_roots","revocation_lists","security_audit_markers"],
            "settlement": ["bridge_settlement_root","dispute_intake","final_confirmation_state","cross_domain_settlement_journal"],
            "treasury": ["protocol_treasury","reserve_balances","insurance_reserve","strategic_liquidity_authority","module_funding_authority"]
        },
        "local_must_not_keep": ["heavy_application_logic","chain_specific_dapp_logic","remote_integration_implementation_details","large_data_payloads","ai_decision_engine","experimental_app_execution"]
    });
    print_json(&output)
}

pub(crate) fn is_official_release(build: &BuildInfo) -> bool {
    let channel_ok = matches!(build.release_channel, "stable" | "official" | "mainnet");
    let cert_ok = !matches!(build.cert_sha256, "not-configured" | "unavailable");
    channel_ok && build.git_dirty == "false" && cert_ok && build.attestation_hash.len() == 64
}

pub(crate) fn build_manifest_payload(build: &BuildInfo) -> serde_json::Value {
    let official_release = is_official_release(build);
    serde_json::json!({
        "artifact": {
            "name": "aoxcmd",
            "release_name": AOXC_RELEASE_NAME,
            "version": build.semver,
            "git_commit": build.git_commit,
            "git_dirty": build.git_dirty,
            "source_date_epoch": build.source_date_epoch,
            "build_profile": build.build_profile,
            "release_channel": build.release_channel,
            "attestation_hash": build.attestation_hash,
        },
        "certificate": {
            "path": build.cert_path,
            "sha256": build.cert_sha256,
            "error": build.cert_error,
        },
        "supply_chain_policy": {
            "official_release": official_release,
            "requires_embedded_certificate": true,
            "requires_attestation_hash": true,
            "accept_unofficial_node_builds": false,
        }
    })
}

pub(crate) fn node_connection_policy_payload(build: &BuildInfo) -> serde_json::Value {
    let official_release = is_official_release(build);
    serde_json::json!({
        "local_build": {
            "release_name": AOXC_RELEASE_NAME,
            "version": build.semver,
            "release_channel": build.release_channel,
            "git_dirty": build.git_dirty,
            "attestation_hash": build.attestation_hash,
            "embedded_cert_sha256": build.cert_sha256,
            "official_release": official_release,
        },
        "accepted_remote_policy": {
            "require_mtls": true,
            "require_certificate_fingerprint_match": true,
            "require_attestation_hash_exchange": true,
            "allow_unofficial_remote_builds": false,
            "accepted_release_channels": ["stable", "official", "mainnet"],
        },
        "operator_guidance": [
            "Embed a node certificate at build time with AOXC_EMBED_CERT_PATH",
            "Distribute attestation_hash and certificate fingerprint via a signed release manifest",
            "Reject ad-hoc local builds for production peering unless explicitly approved",
        ]
    })
}

pub(crate) fn cmd_module_architecture() -> Result<(), String> {
    let relay_module_names: Vec<&str> = canonical_modules()
        .iter()
        .map(|module| module.as_str())
        .collect();
    let sovereign_roots: Vec<&str> = canonical_sovereign_roots()
        .iter()
        .map(|root| root.as_str())
        .collect();
    let supported_chain_families: Vec<&str> = canonical_chain_families()
        .iter()
        .map(|family| family.as_str())
        .collect();
    let envelope_fields = canonical_message_envelope_fields();

    let output = serde_json::json!({
        "relay_core": {
            "principle": "keep the relay chain thin, neutral, and durable",
            "canonical_modules": relay_module_names,
            "sovereign_roots": sovereign_roots,
            "responsibilities": [
                "finality_ordering","shared_security","validator_set_management","cross_module_message_routing",
                "universal_identity_root","state_commitment_and_proof_root_anchoring","governance_and_upgrades",
                "fee_and_staking_settlement_root"
            ]
        },
        "attached_modules": [
            {"name": "AOXC-MODULE-IDENTITY","purpose": "universal identity, address binding, recovery, delegates, chain account mapping","must_depend_on_relay": ["identity_root", "governance", "state_commitment"]},
            {"name": "AOXC-MODULE-ASSET","purpose": "native asset, wrapped assets, treasury accounting, bridge escrow and settlement balances","must_depend_on_relay": ["settlement_root", "governance", "security_policy"]},
            {"name": "AOXC-MODULE-EXECUTION","purpose": "contracts, programmable actions, intents, and app-specific logic outside the relay core","must_depend_on_relay": ["checkpoint_acceptance", "message_bus", "governance"]},
            {"name": "AOXC-MODULE-INTEROP","purpose": "single bridge domain with adapter families for external chain connectivity","adapters": ["evm", "solana", "utxo", "ibc", "object"],"must_depend_on_relay": ["message_bus", "identity_root", "proof_anchoring", "security_policy"]},
            {"name": "AOXC-MODULE-PROOF","purpose": "data commitments, proof publication, light-client support data, batch/blob references","must_depend_on_relay": ["state_commitment", "finality", "governance"]}
        ],
        "message_envelope": {"fields": envelope_fields},
        "security_boundaries": {
            "relay_core": ["minimum_attack_surface","critical_state_only","no_heavy_app_logic","governance_controlled_upgrades"],
            "modules": ["separate_risk_domains","separate_rate_limits","separate_circuit_breakers","separate_fee_policies","separate_storage_proof_domains"]
        },
        "compatibility_strategy": {
            "model": "functional modules + adapter families",
            "supported_chain_families": supported_chain_families,
            "do_not_do": "do not turn the relay chain into a heavy application chain",
            "why": "chain families evolve, but identity, asset, execution, interop, and proof responsibilities remain stable"
        }
    });
    print_json(&output)
}

pub(crate) fn cmd_port_map() -> Result<(), String> {
    let ports: Vec<_> = PORT_BINDINGS
        .iter()
        .map(|binding| {
            serde_json::json!({
                "name": binding.name,
                "protocol": binding.protocol,
                "bind": binding.bind,
                "port": binding.port,
                "purpose": binding.purpose,
            })
        })
        .collect();
    print_json(&serde_json::json!({"primary_rpc_port": RPC_HTTP_PORT, "ports": ports}))
}

pub(crate) fn cmd_compat_matrix() -> Result<(), String> {
    let output = serde_json::json!({
        "execution_lanes": ["EVM", "WASM", "Sui Move", "Cardano UTXO"],
        "network_surface": ["Gossip", "Discovery", "Sync", "RPC"],
        "transport_profiles": ["TCP", "UDP", "QUIC"],
        "support_model": {"evm_family": "partial","wasm_family": "partial","move_family": "partial","utxo_family": "partial","all_chains_full_compatibility": false},
        "compatibility": {"evm_chains": "bridge-compatible via aoxcvm::lanes::evm","wasm_chains": "bridge-compatible via aoxcvm::lanes::wasm","move_ecosystem": "bridge-compatible via aoxcvm::lanes::sui_move","utxo_ecosystem": "bridge-compatible via aoxcvm::lanes::cardano"},
        "hard_limits": ["No relay chain can honestly guarantee 100% security","Full compatibility with every chain requires chain-specific adapters, test vectors, and finality proofs"],
        "note": "Deterministic coordination is implemented; production interoperability requires chain-specific bridge adapters, replay/finality validation, and audits."
    });
    print_json(&output)
}

fn print_json(value: &serde_json::Value) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );
    Ok(())
}
