/// Static metrics snapshot used by `/metrics` style exporters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetricsSnapshot {
    pub tps: f64,
    pub peer_count: usize,
    pub error_rate: f64,
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self {
            tps: 0.0,
            peer_count: 0,
            error_rate: 0.0,
        }
    }
}

impl MetricsSnapshot {
    pub fn to_prometheus(self) -> String {
        format!(
            "aox_tps {}\naox_peer_count {}\naox_error_rate {}\n",
            self.tps, self.peer_count, self.error_rate
        )
    }
}
