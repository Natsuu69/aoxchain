use thiserror::Error;

/// Network subsystem errors.
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("peer disconnected")]
    PeerDisconnected,
    #[error("peer already registered: {0}")]
    PeerAlreadyRegistered(String),
    #[error("unknown peer: {0}")]
    UnknownPeer(String),
    #[error("certificate validation failed: {0}")]
    CertificateValidationFailed(String),
    #[error("invalid security mode transition")]
    InvalidSecurityMode,
}
