use crate::cli::TESTNET_FIXTURE_MEMBERS;
use crate::cli_support::{arg_flag, arg_value};
use crate::data_home;
use crate::keys::{KeyBootstrapRequest, KeyManager, KeyPaths};
use aoxcore::genesis::config::{GenesisConfig, SettlementLink, TREASURY_ACCOUNT};
use aoxcore::genesis::loader::GenesisLoader;
use aoxcore::identity::ca::CertificateAuthority;
use aoxcore::identity::hd_path::HdPath;
use aoxcore::identity::key_engine::{KeyEngine, MASTER_SEED_LEN};
use serde::Serialize;
use sha3::{Digest, Sha3_256};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, Clone)]
pub(crate) struct BootstrapDefaults {
    pub(crate) profile: &'static str,
    pub(crate) name: String,
    pub(crate) chain: String,
    pub(crate) issuer: String,
}

pub(crate) fn bootstrap_defaults(args: &[String]) -> Result<BootstrapDefaults, String> {
    let profile = arg_value(args, "--profile").unwrap_or_else(|| "mainnet".to_string());
    match profile.as_str() {
        "mainnet" => Ok(BootstrapDefaults {
            profile: "mainnet",
            name: "node".to_string(),
            chain: "AOXC-MAIN".to_string(),
            issuer: "AOXC-ROOT-CA".to_string(),
        }),
        "testnet" | "test" => Ok(BootstrapDefaults {
            profile: "testnet",
            name: "TEST-VALIDATOR-01".to_string(),
            chain: "TEST-XXX-XX-LOCAL".to_string(),
            issuer: "TEST-XXX-ROOT-CA".to_string(),
        }),
        other => Err(format!(
            "unsupported --profile value: {other}, expected mainnet|testnet"
        )),
    }
}

pub(crate) fn assert_mainnet_key_policy(args: &[String], profile: &str) -> Result<(), String> {
    if profile != "mainnet" {
        return Ok(());
    }
    let allow_flag = arg_flag(args, "--allow-mainnet");
    let allow_env = env::var("AOXC_ALLOW_MAINNET_KEYS")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);
    if allow_flag || allow_env {
        Ok(())
    } else {
        Err("mainnet key bootstrap blocked: pass --allow-mainnet or set AOXC_ALLOW_MAINNET_KEYS=true".to_string())
    }
}

pub(crate) fn cmd_key_bootstrap(args: &[String]) -> Result<(), String> {
    let defaults = bootstrap_defaults(args)?;
    assert_mainnet_key_policy(args, defaults.profile)?;
    let home = data_home::resolve_data_home(args);
    let base_dir = arg_value(args, "--base-dir").unwrap_or_else(|| data_home::join(&home, "keys"));
    let name = arg_value(args, "--name").unwrap_or(defaults.name);
    let chain = arg_value(args, "--chain").unwrap_or(defaults.chain);
    let role = arg_value(args, "--role").unwrap_or_else(|| "validator".to_string());
    let zone = arg_value(args, "--zone").unwrap_or_else(|| "core".to_string());
    let issuer = arg_value(args, "--issuer").unwrap_or(defaults.issuer);
    let password = arg_value(args, "--password")
        .ok_or_else(|| "--password is required for key-bootstrap".to_string())?;
    let validity_secs: u64 = arg_value(args, "--validity-secs")
        .unwrap_or_else(|| "31536000".to_string())
        .parse()
        .map_err(|_| "--validity-secs must be a valid u64".to_string())?;

    let paths = KeyPaths::new(base_dir, &name);
    let request = KeyBootstrapRequest::new(chain, role, zone, password, validity_secs);
    let manager = KeyManager::new(paths, request);
    let ca = CertificateAuthority::new(issuer);
    let material = manager
        .load_or_create(&ca)
        .map_err(|error| format!("key bootstrap failed [{}]: {}", error.code(), error))?;
    print_json(&serde_json::json!({"profile": defaults.profile, "summary": material.summary()}))
}

pub(crate) fn cmd_genesis_init(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let path = arg_value(args, "--path")
        .unwrap_or_else(|| data_home::join(&home, "identity/genesis.json"));
    let chain_num: u32 = arg_value(args, "--chain-num")
        .unwrap_or_else(|| "1".to_string())
        .parse()
        .map_err(|_| "--chain-num must be a valid u32".to_string())?;
    let block_time: u64 = arg_value(args, "--block-time")
        .unwrap_or_else(|| "6".to_string())
        .parse()
        .map_err(|_| "--block-time must be a valid u64".to_string())?;
    let treasury: u128 = arg_value(args, "--treasury")
        .unwrap_or_else(|| "1000000000".to_string())
        .parse()
        .map_err(|_| "--treasury must be a valid u128".to_string())?;
    let native_symbol = arg_value(args, "--native-symbol").unwrap_or_else(|| "AOXC".to_string());
    let native_decimals: u8 = arg_value(args, "--native-decimals")
        .unwrap_or_else(|| "18".to_string())
        .parse()
        .map_err(|_| "--native-decimals must be a valid u8".to_string())?;
    let settlement_network =
        arg_value(args, "--settlement-network").unwrap_or_else(|| "xlayer".to_string());
    let settlement_token_address = arg_value(args, "--xlayer-token")
        .unwrap_or_else(|| "0xeb9580c3946bb47d73aae1d4f7a94148b554b2f4".to_string());
    let settlement_main_contract = arg_value(args, "--xlayer-main-contract")
        .unwrap_or_else(|| "0x97bdd1fd1caf756e00efd42eba9406821465b365".to_string());
    let settlement_multisig_contract = arg_value(args, "--xlayer-multisig")
        .unwrap_or_else(|| "0x20c0dd8b6559912acfac2ce061b8d5b19db8ca84".to_string());
    let equivalence_mode =
        arg_value(args, "--equivalence-mode").unwrap_or_else(|| "1:1".to_string());

    let mut config = GenesisConfig::new();
    config.chain_num = chain_num;
    config.chain_id = GenesisConfig::generate_chain_id(chain_num);
    config.block_time = block_time;
    config.treasury = treasury;
    config.settlement_link = SettlementLink {
        native_symbol,
        native_decimals,
        settlement_network,
        settlement_token_address,
        settlement_main_contract,
        settlement_multisig_contract,
        equivalence_mode,
    };
    config.add_account(TREASURY_ACCOUNT.to_string(), treasury);
    config.validate()?;
    GenesisLoader::save(&config, &path).map_err(|error| error.to_string())?;
    let loaded = GenesisLoader::load(&path).map_err(|error| error.to_string())?;
    print_json(&serde_json::json!({
        "saved_path": path,
        "chain_num": loaded.config.chain_num,
        "chain_id": loaded.config.chain_id,
        "block_time": loaded.config.block_time,
        "treasury": loaded.config.treasury,
        "total_supply": loaded.config.total_supply(),
        "state_hash": loaded.config.state_hash(),
        "settlement_link": {
            "native_symbol": loaded.config.settlement_link.native_symbol,
            "native_decimals": loaded.config.settlement_link.native_decimals,
            "settlement_network": loaded.config.settlement_link.settlement_network,
            "settlement_token_address": loaded.config.settlement_link.settlement_token_address,
            "settlement_main_contract": loaded.config.settlement_link.settlement_main_contract,
            "settlement_multisig_contract": loaded.config.settlement_link.settlement_multisig_contract,
            "equivalence_mode": loaded.config.settlement_link.equivalence_mode
        }
    }))
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TestnetFixtureAccount {
    pub(crate) slug: String,
    pub(crate) display_name: String,
    pub(crate) chain_num: u32,
    pub(crate) hd_path: String,
    pub(crate) master_seed_hex: String,
    pub(crate) node_seed_path: String,
    pub(crate) account_address: String,
    pub(crate) validator_id_hex: String,
    pub(crate) account_funding: String,
    pub(crate) p2p_listen_addr: String,
    pub(crate) rpc_addr: String,
    pub(crate) peers: Vec<String>,
    pub(crate) key_engine_fingerprint: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TestnetFixtureManifest {
    pub(crate) profile: String,
    pub(crate) chain_num: u32,
    pub(crate) chain_id: String,
    pub(crate) block_time_secs: u64,
    pub(crate) security_mode: String,
    pub(crate) fund_amount_per_account: String,
    pub(crate) warning: String,
    pub(crate) accounts: Vec<TestnetFixtureAccount>,
}

pub(crate) fn cmd_testnet_fixture_init(args: &[String]) -> Result<(), String> {
    let output_dir = arg_value(args, "--output-dir")
        .unwrap_or_else(|| "configs/deterministic-testnet".to_string());
    let chain_num: u32 = arg_value(args, "--chain-num")
        .unwrap_or_else(|| "77".to_string())
        .parse()
        .map_err(|_| "--chain-num must be a valid u32".to_string())?;
    let fund_amount: u128 = arg_value(args, "--fund-amount")
        .unwrap_or_else(|| "2500000000000000000000".to_string())
        .parse()
        .map_err(|_| "--fund-amount must be a valid u128".to_string())?;
    let manifest = build_testnet_fixture_manifest(chain_num, fund_amount)?;
    write_testnet_fixture(&output_dir, &manifest)?;
    print_json(&serde_json::json!({
        "output_dir": output_dir,
        "chain_id": manifest.chain_id,
        "account_count": manifest.accounts.len(),
        "accounts_file": format!("{}/accounts.json", output_dir),
        "genesis_file": format!("{}/genesis.json", output_dir),
        "launch_script": format!("{}/launch-testnet.sh", output_dir),
        "warning": manifest.warning,
    }))
}

pub(crate) fn build_testnet_fixture_manifest(
    chain_num: u32,
    fund_amount: u128,
) -> Result<TestnetFixtureManifest, String> {
    let mut accounts = Vec::with_capacity(TESTNET_FIXTURE_MEMBERS.len());
    for (slug, display_name, p2p_port, rpc_port, zone, master_seed_hex) in TESTNET_FIXTURE_MEMBERS {
        let seed = decode_master_seed_hex(master_seed_hex)?;
        let key_engine = KeyEngine::from_seed(seed);
        let hd_path = HdPath::new(chain_num, 1, u32::from(zone), 0)
            .map_err(|error| format!("invalid deterministic hd path: {error}"))?;
        let entropy = key_engine.derive_entropy(&hd_path);
        let account_address = deterministic_address(slug, &entropy);
        let validator_id_hex = hex::encode_upper(&entropy[..32]);
        let peers = TESTNET_FIXTURE_MEMBERS
            .iter()
            .filter(|(peer_slug, ..)| peer_slug != &slug)
            .map(|(_, _, peer_p2p_port, ..)| format!("127.0.0.1:{peer_p2p_port}"))
            .collect();
        accounts.push(TestnetFixtureAccount {
            slug: slug.to_string(),
            display_name: display_name.to_string(),
            chain_num,
            hd_path: hd_path.to_string(),
            master_seed_hex: master_seed_hex.to_string(),
            node_seed_path: "identity/test-node-seed.hex".to_string(),
            account_address,
            validator_id_hex,
            account_funding: fund_amount.to_string(),
            p2p_listen_addr: format!("127.0.0.1:{p2p_port}"),
            rpc_addr: format!("127.0.0.1:{rpc_port}"),
            peers,
            key_engine_fingerprint: key_engine.fingerprint(),
        });
    }
    Ok(TestnetFixtureManifest {
        profile: "deterministic-testnet".to_string(),
        chain_num,
        chain_id: GenesisConfig::generate_chain_id(chain_num),
        block_time_secs: 4,
        security_mode: "mutual_auth_test_fixture".to_string(),
        fund_amount_per_account: fund_amount.to_string(),
        warning:
            "TEST ONLY: all seeds in this fixture are public and must never be used in production."
                .to_string(),
        accounts,
    })
}

fn write_testnet_fixture(
    output_dir: &str,
    manifest: &TestnetFixtureManifest,
) -> Result<(), String> {
    fs::create_dir_all(output_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(format!("{output_dir}/nodes")).map_err(|error| error.to_string())?;
    fs::create_dir_all(format!("{output_dir}/homes")).map_err(|error| error.to_string())?;
    let accounts_json = serde_json::to_vec_pretty(manifest)
        .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?;
    fs::write(format!("{output_dir}/accounts.json"), accounts_json)
        .map_err(|error| error.to_string())?;

    let mut genesis = GenesisConfig::new();
    genesis.chain_num = manifest.chain_num;
    genesis.chain_id = manifest.chain_id.clone();
    genesis.block_time = manifest.block_time_secs;
    genesis.treasury = 5_000_000_000_000;
    genesis.add_account(TREASURY_ACCOUNT.to_string(), genesis.treasury);
    for account in &manifest.accounts {
        genesis.add_account(
            account.account_address.clone(),
            account
                .account_funding
                .parse()
                .map_err(|_| "invalid account_funding in manifest".to_string())?,
        );
    }
    GenesisLoader::save(&genesis, format!("{output_dir}/genesis.json"))
        .map_err(|error| error.to_string())?;

    for account in &manifest.accounts {
        fs::write(
            format!("{output_dir}/nodes/{}.toml", account.slug),
            render_node_toml(account, manifest),
        )
        .map_err(|error| error.to_string())?;
        let home_identity_dir = format!("{output_dir}/homes/{}/identity", account.slug);
        fs::create_dir_all(&home_identity_dir).map_err(|error| error.to_string())?;
        fs::write(
            format!("{home_identity_dir}/test-node-seed.hex"),
            format!("{}\n", account.master_seed_hex),
        )
        .map_err(|error| error.to_string())?;
        fs::copy(
            format!("{output_dir}/genesis.json"),
            format!("{home_identity_dir}/genesis.json"),
        )
        .map_err(|error| error.to_string())?;
    }

    let launch_script_path = format!("{output_dir}/launch-testnet.sh");
    fs::write(&launch_script_path, render_launch_script(manifest))
        .map_err(|error| error.to_string())?;
    fs::set_permissions(&launch_script_path, fs::Permissions::from_mode(0o755))
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn render_node_toml(account: &TestnetFixtureAccount, manifest: &TestnetFixtureManifest) -> String {
    let peers = account
        .peers
        .iter()
        .map(|peer| format!("  \"{peer}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "chain_id = \"{}\"\nnode_name = \"{}\"\nlisten_addr = \"{}\"\nrpc_addr = \"{}\"\npeers = [\n{}\n]\nsecurity_mode = \"{}\"\nhd_path = \"{}\"\nvalidator_id_hex = \"{}\"\naccount_address = \"{}\"\nwarning = \"TEST ONLY - public fixture seed\"\n",
        manifest.chain_id,
        account.slug,
        account.p2p_listen_addr,
        account.rpc_addr,
        peers,
        manifest.security_mode,
        account.hd_path,
        account.validator_id_hex,
        account.account_address
    )
}

pub(crate) fn render_launch_script(manifest: &TestnetFixtureManifest) -> String {
    let mut script = String::from(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nROOT_DIR=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd)\"\nAOXC_BIN=\"${AOXC_BIN:-cargo run -q -p aoxcmd --}\"\nROUNDS=\"${ROUNDS:-2}\"\nSLEEP_MS=\"${SLEEP_MS:-250}\"\n\necho \"[fixture] chain_id=",
    );
    script.push_str(&manifest.chain_id);
    script.push_str("\"\n");
    script.push_str("echo \"[fixture] TEST ONLY seeds are public; do not reuse outside local/dev environments.\"\n\n");
    for account in &manifest.accounts {
        let tx_prefix = format!("{}-TX", account.slug.to_uppercase());
        script.push_str(&format!("echo \"[fixture] bootstrapping {slug}\" \n$AOXC_BIN node-bootstrap --home \"$ROOT_DIR/homes/{slug}\" >/tmp/aoxc-{slug}-bootstrap.json\n$AOXC_BIN node-run --home \"$ROOT_DIR/homes/{slug}\" --rounds \"$ROUNDS\" --sleep-ms \"$SLEEP_MS\" --tx-prefix \"{tx_prefix}\" >/tmp/aoxc-{slug}-run.json\n", slug = account.slug, tx_prefix = tx_prefix));
    }
    script
}

fn decode_master_seed_hex(value: &str) -> Result<[u8; MASTER_SEED_LEN], String> {
    let raw = hex::decode(value).map_err(|_| "fixture seed must be valid hex".to_string())?;
    if raw.len() != MASTER_SEED_LEN {
        return Err(format!(
            "fixture seed must be {MASTER_SEED_LEN} bytes, got {}",
            raw.len()
        ));
    }
    let mut seed = [0u8; MASTER_SEED_LEN];
    seed.copy_from_slice(&raw);
    Ok(seed)
}

fn deterministic_address(slug: &str, entropy: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(b"AOXC-TESTNET-FIXTURE-ADDRESS-V1");
    hasher.update([0x00]);
    hasher.update(slug.as_bytes());
    hasher.update([0x00]);
    hasher.update(entropy);
    let digest = hasher.finalize();
    format!("AOXC_TEST_{}", hex::encode_upper(&digest[..20]))
}

fn print_json(value: &serde_json::Value) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );
    Ok(())
}
