use aoxcunity::messages::ConsensusMessage;

use crate::config::SecurityMode;
use crate::error::NetworkError;
use crate::gossip::peer::Peer;
use crate::p2p::P2PNetwork;

/// Gossip engine responsible for propagating and receiving consensus messages.
#[derive(Debug, Clone)]
pub struct GossipEngine {
    network: P2PNetwork,
}

impl Default for GossipEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GossipEngine {
    /// Creates a new gossip engine instance with secure defaults.
    pub fn new() -> Self {
        Self {
            network: P2PNetwork::new(SecurityMode::MutualAuth, 128),
        }
    }

    /// Registers peer identity and certificate chain metadata.
    pub fn register_peer(&mut self, peer: Peer) -> Result<(), NetworkError> {
        self.network.register_peer(peer)
    }

    /// Establishes session tickets for a peer.
    pub fn establish_session(&mut self, peer_id: &str) -> Result<(), NetworkError> {
        self.network.establish_session(peer_id).map(|_| ())
    }

    /// Broadcasts a consensus message to connected peers.
    ///
    /// Backward-compatible path used by existing CLI smoke flows.
    pub fn broadcast(&mut self, msg: ConsensusMessage) {
        self.network.broadcast_compat(msg);
    }

    /// Secure broadcast from a known peer id.
    pub fn broadcast_from_peer(
        &mut self,
        peer_id: &str,
        msg: ConsensusMessage,
    ) -> Result<(), NetworkError> {
        self.network.broadcast_secure(peer_id, msg)
    }

    /// Receives the next available consensus message from the gossip layer.
    pub fn receive(&mut self) -> Option<ConsensusMessage> {
        self.network.receive()
    }

    pub fn stats(&self) -> (usize, usize) {
        (
            self.network.registered_peers(),
            self.network.active_sessions(),
        )
    }
}
