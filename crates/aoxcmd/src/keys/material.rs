use crate::error::{AppError, ErrorCode};
use aoxcore::identity::{
    key_bundle::{CryptoProfile, NodeKeyBundleV1},
    key_engine::KeyEngine,
    keyfile::{encrypt_key_to_envelope, KeyfileEnvelope},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub bundle: NodeKeyBundleV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterialSummary {
    pub bundle_fingerprint: String,
    pub crypto_profile: String,
    pub engine_fingerprint: String,
    pub consensus_public_key: String,
    pub transport_public_key: String,
    pub operator_public_key: String,
    pub role_count: usize,
}

impl KeyMaterial {
    pub fn generate(name: &str, profile: &str, password: &str) -> Result<Self, AppError> {
        let created_at = Utc::now().to_rfc3339();
        let engine = KeyEngine::new(None);
        let encrypted_root_seed =
            encrypt_key_to_envelope(engine.master_seed(), password).map_err(|error| {
                AppError::with_source(
                    ErrorCode::KeyMaterialInvalid,
                    "Failed to protect operator key material",
                    error,
                )
            })?;
        let bundle = NodeKeyBundleV1::generate(
            name,
            profile,
            created_at,
            infer_crypto_profile(profile),
            &engine,
            encrypted_root_seed,
        )
        .map_err(|error| {
            AppError::with_source(
                ErrorCode::KeyMaterialInvalid,
                "Failed to build canonical node key bundle",
                error,
            )
        })?;

        Ok(Self { bundle })
    }

    pub fn fingerprint(&self) -> &str {
        &self.bundle.bundle_fingerprint
    }

    pub fn encrypted_root_seed(&self) -> &KeyfileEnvelope {
        &self.bundle.encrypted_root_seed
    }

    pub fn consensus_public_key_hex(&self) -> Result<&str, AppError> {
        self.bundle
            .public_key_hex_for_role(aoxcore::identity::key_bundle::NodeKeyRole::Consensus)
            .map_err(|error| {
                AppError::with_source(
                    ErrorCode::KeyMaterialInvalid,
                    "Failed to read canonical consensus public key from key bundle",
                    error,
                )
            })
    }

    pub fn summary(&self) -> Result<KeyMaterialSummary, AppError> {
        let role_key = |role| {
            self.bundle
                .public_key_hex_for_role(role)
                .map(|value| value.to_string())
                .map_err(|error| {
                    AppError::with_source(
                        ErrorCode::KeyMaterialInvalid,
                        "Failed to build key material summary from bundle",
                        error,
                    )
                })
        };

        Ok(KeyMaterialSummary {
            bundle_fingerprint: self.bundle.bundle_fingerprint.clone(),
            crypto_profile: self.bundle.crypto_profile.to_string(),
            engine_fingerprint: self.bundle.engine_fingerprint.clone(),
            consensus_public_key: role_key(aoxcore::identity::key_bundle::NodeKeyRole::Consensus)?,
            transport_public_key: role_key(aoxcore::identity::key_bundle::NodeKeyRole::Transport)?,
            operator_public_key: role_key(aoxcore::identity::key_bundle::NodeKeyRole::Operator)?,
            role_count: self.bundle.keys.len(),
        })
    }
}

fn infer_crypto_profile(profile: &str) -> CryptoProfile {
    match profile.trim().to_ascii_lowercase().as_str() {
        "mainnet" => CryptoProfile::HybridEd25519Dilithium3,
        "testnet" | "validator" => CryptoProfile::ClassicEd25519,
        _ => CryptoProfile::HybridEd25519Dilithium3,
    }
}

pub fn validate_key_envelope(serialized: &str) -> Result<KeyfileEnvelope, AppError> {
    serde_json::from_str(serialized).map_err(|error| {
        AppError::with_source(
            ErrorCode::KeyMaterialInvalid,
            "Stored operator key envelope is malformed",
            error,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{validate_key_envelope, KeyMaterial};

    #[test]
    fn generated_material_uses_canonical_node_key_bundle() {
        let material = KeyMaterial::generate("validator-01", "testnet", "Test#2026!")
            .expect("key generation should succeed");

        assert_eq!(material.bundle.version, 1);
        assert_eq!(material.bundle.keys.len(), 6);
        assert_eq!(
            material.bundle.custody_model,
            "encrypted-root-seed-envelope"
        );
        assert!(material.bundle.keys[0].hd_path.starts_with("m/44/2626/"));

        let serialized = serde_json::to_string_pretty(material.encrypted_root_seed())
            .expect("envelope serialization should succeed");
        assert!(validate_key_envelope(&serialized).is_ok());
        assert_eq!(
            material.summary().expect("summary should build").role_count,
            6
        );
    }
}
