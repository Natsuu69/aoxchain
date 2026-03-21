use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use aoxcunity::messages::ConsensusMessage;
use sha2::{Digest, Sha256};

use crate::config::{NetworkConfig, SecurityMode};
use crate::error::NetworkError;
use crate::gossip::peer::Peer;

/// Represents an authenticated peer session established by the in-memory P2P
/// runtime.
///
/// The session ticket binds a peer identity to a certificate fingerprint,
/// deterministic session identifier, replay nonce stream, and finite trust
/// window. This structure is intentionally lightweight and does not require
/// additional serialization dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTicket {
    /// Canonical peer identifier recognized by the runtime.
    pub peer_id: String,

    /// Fingerprint of the certificate accepted during session establishment.
    pub cert_fingerprint: String,

    /// UNIX timestamp at which the session was established.
    pub established_at_unix: u64,

    /// Session-scoped replay-protection nonce.
    pub replay_window_nonce: u64,

    /// Deterministically derived session identifier.
    pub session_id: String,

    /// UNIX timestamp after which the session must no longer be trusted.
    pub expires_at_unix: u64,
}

/// Canonical in-memory protocol frame used by the AOXC network shell.
///
/// The envelope binds the payload to a chain-domain label, peer identity,
/// session identity, nonce, and issuance metadata. It is intentionally
/// minimal so it can remain stable during transport-layer evolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolEnvelope {
    /// Protocol framing version for forward compatibility.
    pub protocol_version: u16,

    /// Canonical AOXC chain identifier bound to the frame.
    pub chain_id: String,

    /// Peer identifier from which the frame originated.
    pub peer_id: String,

    /// Session identifier under which the frame was emitted.
    pub session_id: String,

    /// Session-scoped replay nonce used for duplicate rejection.
    pub nonce: u64,

    /// UNIX timestamp at which the frame was issued.
    pub issued_at_unix: u64,

    /// UNIX timestamp after which the frame must no longer be trusted.
    pub expires_at_unix: u64,

    /// Deterministic frame integrity hash.
    pub frame_hash_hex: String,

    /// Consensus payload transported by this envelope.
    pub payload: ConsensusMessage,
}

/// In-memory secure transport shell for deterministic tests, smoke validation,
/// and future transport-adapter integration.
///
/// This implementation prioritizes predictable security behavior over transport
/// sophistication. It enforces peer admission checks, authenticated session
/// establishment, replay detection, bounded session lifetime, and explicit
/// ban-state handling.
#[derive(Debug, Clone)]
pub struct P2PNetwork {
    config: NetworkConfig,
    peers: HashMap<String, Peer>,
    sessions: HashMap<String, SessionTicket>,
    replay_cache: HashSet<String>,
    replay_order: VecDeque<String>,
    inbound: VecDeque<ProtocolEnvelope>,
    banned_until_unix: HashMap<String, u64>,
}

impl P2PNetwork {
    /// Creates a new in-memory AOXC network runtime bound to the supplied
    /// network configuration.
    #[must_use]
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            peers: HashMap::new(),
            sessions: HashMap::new(),
            replay_cache: HashSet::new(),
            replay_order: VecDeque::new(),
            inbound: VecDeque::new(),
            banned_until_unix: HashMap::new(),
        }
    }

    /// Returns the active security mode.
    #[must_use]
    pub fn security_mode(&self) -> SecurityMode {
        self.config.security_mode
    }

    /// Returns the number of currently registered peers.
    #[must_use]
    pub fn registered_peers(&self) -> usize {
        self.peers.len()
    }

    /// Returns the number of currently active authenticated sessions.
    #[must_use]
    pub fn active_sessions(&self) -> usize {
        self.sessions.len()
    }

    /// Registers a peer after capacity, certificate, and ban-state checks.
    pub fn register_peer(&mut self, peer: Peer) -> Result<(), NetworkError> {
        if self.peers.contains_key(&peer.id) {
            return Err(NetworkError::PeerAlreadyRegistered(peer.id));
        }

        if self.is_banned(&peer.id) {
            return Err(NetworkError::PeerDisconnected);
        }

        if self.peers.len() >= self.config.max_peers_total() {
            return Err(NetworkError::PeerDisconnected);
        }

        peer.validate_certificate_for_mode(self.config.security_mode)?;
        self.peers.insert(peer.id.clone(), peer);

        Ok(())
    }

    /// Establishes an authenticated session for a previously registered peer.
    pub fn establish_session(&mut self, peer_id: &str) -> Result<SessionTicket, NetworkError> {
        if self.is_banned(peer_id) {
            return Err(NetworkError::PeerDisconnected);
        }

        let peer = self
            .peers
            .get(peer_id)
            .ok_or_else(|| NetworkError::UnknownPeer(peer_id.to_string()))?;

        let now = unix_now();
        let ticket = SessionTicket {
            peer_id: peer.id.clone(),
            cert_fingerprint: peer.cert_fingerprint.clone(),
            established_at_unix: now,
            replay_window_nonce: initial_nonce(now),
            session_id: derive_session_id(peer_id, &peer.cert_fingerprint, now),
            expires_at_unix: now.saturating_add(session_lifetime_secs(&self.config)),
        };

        self.sessions.insert(peer.id.clone(), ticket.clone());
        Ok(ticket)
    }

    /// Securely broadcasts a payload from an authenticated peer session.
    ///
    /// The method rejects unknown peers, banned peers, expired sessions, and
    /// replayed session-nonce pairs. It returns the canonical protocol envelope
    /// pushed into the inbound queue.
    pub fn broadcast_secure(
        &mut self,
        from_peer_id: &str,
        payload: ConsensusMessage,
    ) -> Result<ProtocolEnvelope, NetworkError> {
        if self.is_banned(from_peer_id) {
            return Err(NetworkError::PeerDisconnected);
        }

        let now = unix_now();
        let security_mode = self.config.security_mode;
        let chain_id = self.config.interop.canonical_chain_id().to_string();

        let envelope = {
            let ticket = self
                .sessions
                .get_mut(from_peer_id)
                .ok_or_else(|| NetworkError::UnknownPeer(from_peer_id.to_string()))?;

            if security_mode != SecurityMode::Insecure && now > ticket.expires_at_unix {
                return Err(NetworkError::PeerDisconnected);
            }

            let envelope = ProtocolEnvelope::new(&chain_id, ticket, payload, now);
            ticket.replay_window_nonce = ticket.replay_window_nonce.saturating_add(1);
            envelope
        };

        let replay_key = replay_key(&envelope.session_id, envelope.nonce);
        if self.replay_cache.contains(&replay_key) {
            return Err(NetworkError::PeerDisconnected);
        }

        self.replay_cache.insert(replay_key.clone());
        self.replay_order.push_back(replay_key);
        self.trim_replay_cache();

        self.inbound.push_back(envelope.clone());
        Ok(envelope)
    }

    /// Receives the next consensus payload from the inbound queue, if any.
    pub fn receive(&mut self) -> Option<ConsensusMessage> {
        self.inbound.pop_front().map(|envelope| envelope.payload)
    }

    /// Bans the peer for the configured ban window and tears down any active
    /// session associated with it.
    pub fn ban_peer(&mut self, peer_id: &str) {
        let until = unix_now().saturating_add(self.config.peer_ban_secs);
        self.banned_until_unix.insert(peer_id.to_string(), until);
        self.sessions.remove(peer_id);
    }

    /// Returns `true` when the peer is presently inside an active ban window.
    #[must_use]
    fn is_banned(&self, peer_id: &str) -> bool {
        self.banned_until_unix
            .get(peer_id)
            .map(|until| unix_now() <= *until)
            .unwrap_or(false)
    }

    /// Trims the replay cache deterministically by evicting the oldest replay
    /// keys first.
    fn trim_replay_cache(&mut self) {
        while self.replay_cache.len() > self.config.replay_window_size {
            let Some(oldest_key) = self.replay_order.pop_front() else {
                break;
            };
            self.replay_cache.remove(&oldest_key);
        }
    }
}

impl ProtocolEnvelope {
    /// Creates a new canonical protocol envelope from a trusted session ticket.
    #[must_use]
    pub fn new(
        chain_id: &str,
        ticket: &SessionTicket,
        payload: ConsensusMessage,
        issued_at_unix: u64,
    ) -> Self {
        let frame_hash_hex = derive_frame_hash(
            chain_id,
            &ticket.peer_id,
            &ticket.session_id,
            ticket.replay_window_nonce,
            issued_at_unix,
            ticket.expires_at_unix,
        );

        Self {
            protocol_version: 1,
            chain_id: chain_id.to_string(),
            peer_id: ticket.peer_id.clone(),
            session_id: ticket.session_id.clone(),
            nonce: ticket.replay_window_nonce,
            issued_at_unix,
            expires_at_unix: ticket.expires_at_unix,
            frame_hash_hex,
            payload,
        }
    }
}

/// Returns the session lifetime in seconds derived from the configured idle
/// timeout. A minimum lifetime of one second is enforced.
#[must_use]
fn session_lifetime_secs(config: &NetworkConfig) -> u64 {
    (config.idle_timeout_ms / 1_000).max(1)
}

/// Returns the initial replay nonce derived from current time.
///
/// The nonce derivation intentionally avoids fixed zero initialization so that
/// independent sessions started at different times do not share the same
/// opening nonce.
#[must_use]
fn initial_nonce(now: u64) -> u64 {
    now ^ 0xA0C0_A0C0_A0C0_A0C0_u64
}

/// Derives a deterministic session identifier from peer identity,
/// certificate fingerprint, and establishment timestamp.
#[must_use]
fn derive_session_id(peer_id: &str, cert_fingerprint: &str, unix_ts: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC-NET-SESSION-V1");
    hasher.update(peer_id.as_bytes());
    hasher.update(cert_fingerprint.as_bytes());
    hasher.update(unix_ts.to_be_bytes());
    hex::encode(hasher.finalize())
}

/// Derives a deterministic frame integrity hash from envelope metadata.
///
/// This helper intentionally excludes payload hashing so that the file remains
/// dependency-light and aligned with the current crate graph.
#[must_use]
fn derive_frame_hash(
    chain_id: &str,
    peer_id: &str,
    session_id: &str,
    nonce: u64,
    issued_at_unix: u64,
    expires_at_unix: u64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC-NET-FRAME-V1");
    hasher.update(chain_id.as_bytes());
    hasher.update(peer_id.as_bytes());
    hasher.update(session_id.as_bytes());
    hasher.update(nonce.to_be_bytes());
    hasher.update(issued_at_unix.to_be_bytes());
    hasher.update(expires_at_unix.to_be_bytes());
    hex::encode(hasher.finalize())
}

/// Returns a canonical replay-cache key for a session and nonce pair.
#[must_use]
fn replay_key(session_id: &str, nonce: u64) -> String {
    format!("{session_id}:{nonce}")
}

/// Returns the current UNIX timestamp in seconds.
///
/// In the unlikely event that system time is observed before the UNIX epoch,
/// the function returns zero instead of panicking.
#[must_use]
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::P2PNetwork;
    use crate::config::NetworkConfig;
    use crate::gossip::peer::{NodeCertificate, Peer};
    use aoxcunity::messages::ConsensusMessage;
    use aoxcunity::vote::{Vote, VoteKind};

    fn test_peer() -> Peer {
        let cert = NodeCertificate {
            subject: "node-1".to_string(),
            issuer: "AOXC-ROOT".to_string(),
            valid_from_unix: 1,
            valid_until_unix: u64::MAX,
            serial: "serial-1".to_string(),
        };

        Peer::new("node-1", "10.0.0.1:2727", cert)
    }

    fn test_vote() -> ConsensusMessage {
        ConsensusMessage::Vote(Vote {
            voter: [1u8; 32],
            block_hash: [2u8; 32],
            height: 1,
            round: 0,
            kind: VoteKind::Prepare,
        })
    }

    #[test]
    fn secure_broadcast_requires_active_session() {
        let mut net = P2PNetwork::new(NetworkConfig::default());
        let peer = test_peer();

        net.register_peer(peer).expect("peer should register");

        let err = net
            .broadcast_secure("node-1", test_vote())
            .expect_err("broadcast without session must fail");

        assert!(matches!(err, crate::error::NetworkError::UnknownPeer(_)));
    }

    #[test]
    fn session_based_broadcast_is_accepted() {
        let mut net = P2PNetwork::new(NetworkConfig::default());
        let peer = test_peer();

        net.register_peer(peer).expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");

        net.broadcast_secure("node-1", test_vote())
            .expect("broadcast should be accepted");

        assert!(net.receive().is_some());
    }

    #[test]
    fn banned_peer_cannot_broadcast() {
        let mut net = P2PNetwork::new(NetworkConfig::default());
        let peer = test_peer();

        net.register_peer(peer).expect("peer should register");
        net.establish_session("node-1")
            .expect("session should be established");
        net.ban_peer("node-1");

        assert!(net.broadcast_secure("node-1", test_vote()).is_err());
    }
}
