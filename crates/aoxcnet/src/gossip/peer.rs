use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

/// Certificate metadata used for secure peer admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeCertificate {
    pub subject: String,
    pub issuer: String,
    pub valid_from_unix: u64,
    pub valid_until_unix: u64,
    pub serial: String,
}

impl NodeCertificate {
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.subject.as_bytes());
        hasher.update(self.issuer.as_bytes());
        hasher.update(self.serial.as_bytes());
        hasher.update(self.valid_from_unix.to_le_bytes());
        hasher.update(self.valid_until_unix.to_le_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn is_valid_at(&self, unix_ts: u64) -> bool {
        unix_ts >= self.valid_from_unix && unix_ts <= self.valid_until_unix
    }
}

/// Representation of a gossip peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    pub id: String,
    pub address: String,
    pub certificate: NodeCertificate,
    pub cert_fingerprint: String,
}

impl Peer {
    pub fn new(
        id: impl Into<String>,
        address: impl Into<String>,
        certificate: NodeCertificate,
    ) -> Self {
        let cert_fingerprint = certificate.fingerprint();
        Self {
            id: id.into(),
            address: address.into(),
            certificate,
            cert_fingerprint,
        }
    }

    pub fn validate_certificate(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.certificate.is_valid_at(now)
    }
}
