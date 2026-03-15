use crate::config::RpcConfig;
use crate::types::HealthResponse;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Returns the current health status exposed by the HTTP RPC interface.
#[must_use]
pub fn health() -> HealthResponse {
    health_with_context(&RpcConfig::default(), 0)
}

/// Returns a detailed health payload suitable for production-grade health probes.
#[must_use]
pub fn health_with_context(config: &RpcConfig, uptime_secs: u64) -> HealthResponse {
    let mut warnings = Vec::new();

    let tls_cert_exists = Path::new(&config.tls_cert_path).exists();
    let tls_key_exists = Path::new(&config.tls_key_path).exists();
    let mtls_enabled = config.mtls_ca_cert_path.is_some();
    let mtls_ca_exists = config
        .mtls_ca_cert_path
        .as_ref()
        .is_some_and(|path| Path::new(path).exists());

    if config.genesis_hash.is_none() {
        warnings.push("genesis_hash is not configured".to_string());
    }

    if !tls_cert_exists {
        warnings.push("tls certificate file is missing".to_string());
    }

    if !tls_key_exists {
        warnings.push("tls private key file is missing".to_string());
    }

    if mtls_enabled && !mtls_ca_exists {
        warnings.push("mTLS CA certificate file is missing".to_string());
    }

    if !mtls_enabled {
        warnings.push("mTLS is disabled".to_string());
    }

    let readiness_score = readiness_score(
        config.genesis_hash.is_some(),
        tls_cert_exists,
        tls_key_exists,
        mtls_enabled && mtls_ca_exists,
    );

    let status = if warnings.is_empty() {
        "ok".to_string()
    } else {
        "degraded".to_string()
    };

    HealthResponse {
        status,
        chain_id: config.chain_id.clone(),
        genesis_hash: config.genesis_hash.clone(),
        tls_enabled: tls_cert_exists && tls_key_exists,
        mtls_enabled,
        tls_cert_sha256: certificate_fingerprint_sha256_from_path(&config.tls_cert_path),
        readiness_score,
        warnings,
        uptime_secs,
    }
}

fn readiness_score(
    has_genesis_hash: bool,
    tls_cert_exists: bool,
    tls_key_exists: bool,
    mtls_ready: bool,
) -> u8 {
    let mut score = 0_u8;

    if has_genesis_hash {
        score += 25;
    }
    if tls_cert_exists {
        score += 25;
    }
    if tls_key_exists {
        score += 25;
    }
    if mtls_ready {
        score += 25;
    }

    score
}

fn certificate_fingerprint_sha256_from_path(path: &str) -> Option<String> {
    let cert_bytes = fs::read(path).ok()?;
    Some(certificate_fingerprint_sha256(&cert_bytes))
}

fn certificate_fingerprint_sha256(cert_bytes: &[u8]) -> String {
    let digest = Sha256::digest(cert_bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_health_is_degraded_without_production_inputs() {
        let health = health();

        assert_eq!(health.status, "degraded");
        assert!(health.readiness_score < 100);
        assert!(health
            .warnings
            .iter()
            .any(|warning| warning.contains("genesis_hash")));
    }

    #[test]
    fn certificate_fingerprint_is_deterministic() {
        let fingerprint_a = certificate_fingerprint_sha256(b"dummy-cert");
        let fingerprint_b = certificate_fingerprint_sha256(b"dummy-cert");
        let fingerprint_c = certificate_fingerprint_sha256(b"dummy-cert-2");

        assert_eq!(fingerprint_a, fingerprint_b);
        assert_ne!(fingerprint_a, fingerprint_c);
        assert_eq!(fingerprint_a.len(), 64);
    }

    #[test]
    fn health_is_ok_when_all_critical_controls_are_ready() {
        let mut config = RpcConfig::default();
        config.genesis_hash = Some("0xabc123".to_string());
        config.tls_cert_path = "Cargo.toml".to_string();
        config.tls_key_path = "Cargo.toml".to_string();
        config.mtls_ca_cert_path = Some("Cargo.toml".to_string());

        let health = health_with_context(&config, 42);

        assert_eq!(health.status, "ok");
        assert_eq!(health.readiness_score, 100);
        assert!(health.warnings.is_empty());
        assert_eq!(health.uptime_secs, 42);
        assert!(health.tls_cert_sha256.is_some());
    }
}
