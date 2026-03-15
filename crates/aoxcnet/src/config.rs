/// Security policy profile for p2p transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    /// No encryption or certificate checks.
    Insecure,
    /// Mutual certificate verification and replay protection.
    MutualAuth,
    /// Mutual auth + strict policy checks for production environments.
    AuditStrict,
}

/// Network configuration parameters.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addr: String,
    pub public_advertise_addr: String,
    pub max_peers: usize,
    pub heartbeat_ms: u64,
    pub security_mode: SecurityMode,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:26656".to_string(),
            public_advertise_addr: "127.0.0.1:26656".to_string(),
            max_peers: 128,
            heartbeat_ms: 1_000,
            security_mode: SecurityMode::MutualAuth,
        }
    }
}
