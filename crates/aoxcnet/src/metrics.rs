use serde::{Deserialize, Serialize};

/// Mutable network metrics for diagnostics, alerting, and audit reporting.
#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    pub accepted_peers: u64,
    pub rejected_peers: u64,
    pub active_sessions: u64,
    pub failed_handshakes: u64,
    pub replay_detections: u64,
    pub banned_peers: u64,
    pub frames_in: u64,
    pub frames_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub gossip_messages: u64,
    pub sync_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkMetricsSnapshot {
    pub accepted_peers: u64,
    pub rejected_peers: u64,
    pub active_sessions: u64,
    pub failed_handshakes: u64,
    pub replay_detections: u64,
    pub banned_peers: u64,
    pub frames_in: u64,
    pub frames_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub gossip_messages: u64,
    pub sync_requests: u64,
}

impl NetworkMetrics {
    #[must_use]
    pub fn snapshot(&self) -> NetworkMetricsSnapshot {
        NetworkMetricsSnapshot {
            accepted_peers: self.accepted_peers,
            rejected_peers: self.rejected_peers,
            active_sessions: self.active_sessions,
            failed_handshakes: self.failed_handshakes,
            replay_detections: self.replay_detections,
            banned_peers: self.banned_peers,
            frames_in: self.frames_in,
            frames_out: self.frames_out,
            bytes_in: self.bytes_in,
            bytes_out: self.bytes_out,
            gossip_messages: self.gossip_messages,
            sync_requests: self.sync_requests,
        }
    }
}
