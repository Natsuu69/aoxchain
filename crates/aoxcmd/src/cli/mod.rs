use crate::cli_support::{arg_value, detect_language, localized_unknown_command, print_usage};
use std::env;

pub(crate) mod audit;
pub(crate) mod bootstrap;
pub(crate) mod describe;
pub(crate) mod ops;

pub(crate) const AOXC_RELEASE_NAME: &str = "AOXC Alpha: Genesis V1";
pub(crate) const TESTNET_FIXTURE_MEMBERS: [(&str, &str, u16, u16, u16, &str); 5] = [
    (
        "atlas",
        "Atlas Validator",
        39001,
        19101,
        1,
        "11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",
    ),
    (
        "boreal",
        "Boreal Validator",
        39002,
        19102,
        2,
        "22222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222",
    ),
    (
        "cypher",
        "Cypher Validator",
        39003,
        19103,
        3,
        "33333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333",
    ),
    (
        "delta",
        "Delta Validator",
        39004,
        19104,
        4,
        "44444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444",
    ),
    (
        "ember",
        "Ember Validator",
        39005,
        19105,
        5,
        "55555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555",
    ),
];

pub fn run_cli() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let lang = detect_language(&args[1..]);
    apply_home_override(&args[1..]);

    if args.len() < 2 {
        print_usage(lang);
        return Ok(());
    }

    match args[1].as_str() {
        "version" | "--version" | "-V" => describe::cmd_version(),
        "help" | "--help" | "-h" => {
            print_usage(lang);
            Ok(())
        }
        "vision" => describe::cmd_vision(),
        "build-manifest" => describe::cmd_build_manifest(),
        "node-connection-policy" => describe::cmd_node_connection_policy(&args[2..]),
        "sovereign-core" => describe::cmd_sovereign_core(),
        "module-architecture" => describe::cmd_module_architecture(),
        "compat-matrix" => describe::cmd_compat_matrix(),
        "port-map" => describe::cmd_port_map(),
        "testnet-fixture-init" => bootstrap::cmd_testnet_fixture_init(&args[2..]),
        "load-benchmark" => ops::cmd_load_benchmark(&args[2..]),
        "mainnet-readiness" => ops::cmd_mainnet_readiness(),
        "key-bootstrap" => bootstrap::cmd_key_bootstrap(&args[2..]),
        "genesis-init" => bootstrap::cmd_genesis_init(&args[2..]),
        "node-bootstrap" => ops::cmd_node_bootstrap(&args[2..]),
        "produce-once" => ops::cmd_produce_once(&args[2..]),
        "node-run" => ops::cmd_node_run(&args[2..]),
        "network-smoke" => ops::cmd_network_smoke(&args[2..]),
        "real-network" => ops::cmd_real_network(&args[2..]),
        "storage-smoke" => ops::cmd_storage_smoke(&args[2..]),
        "economy-init" => ops::cmd_economy_init(&args[2..]),
        "treasury-transfer" => ops::cmd_treasury_transfer(&args[2..]),
        "stake-delegate" => ops::cmd_stake_delegate(&args[2..]),
        "stake-undelegate" => ops::cmd_stake_undelegate(&args[2..]),
        "economy-status" => ops::cmd_economy_status(&args[2..]),
        "runtime-status" => ops::cmd_runtime_status(&args[2..]),
        "interop-readiness" => audit::cmd_interop_readiness(),
        "interop-gate" => audit::cmd_interop_gate(&args[2..]),
        "production-audit" => audit::cmd_production_audit(&args[2..]),
        other => Err(localized_unknown_command(lang, other)),
    }
}

fn apply_home_override(args: &[String]) {
    if let Some(home) = arg_value(args, "--home") {
        unsafe { env::set_var("AOXC_HOME", home) };
    }
}
