use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::block::Block;
use crate::constitutional::{ConstitutionalSeal, ContinuityCertificate, LegitimacyCertificate};
use crate::seal::QuorumCertificate;
use crate::validator::ValidatorId;
use crate::vote::VerifiedVote;

const TIMEOUT_VOTE_DOMAIN_V1: &[u8] = b"AOXC_TIMEOUT_VOTE_V1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutVote {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub timeout_round: u64,
    pub voter: ValidatorId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedTimeoutVote {
    pub timeout_vote: TimeoutVote,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedTimeoutVote {
    pub timeout_vote: TimeoutVote,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TimeoutAuthenticationError {
    #[error("timeout vote public key is malformed")]
    MalformedPublicKey,

    #[error("timeout vote signature is invalid")]
    InvalidSignature,
}

impl TimeoutVote {
    #[must_use]
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(TIMEOUT_VOTE_DOMAIN_V1.len() + 32 + 8 * 4 + 32);
        bytes.extend_from_slice(TIMEOUT_VOTE_DOMAIN_V1);
        bytes.extend_from_slice(&self.block_hash);
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.round.to_le_bytes());
        bytes.extend_from_slice(&self.epoch.to_le_bytes());
        bytes.extend_from_slice(&self.timeout_round.to_le_bytes());
        bytes.extend_from_slice(&self.voter);
        bytes
    }
}

impl SignedTimeoutVote {
    pub fn verify(&self) -> Result<VerifiedTimeoutVote, TimeoutAuthenticationError> {
        let key = VerifyingKey::from_bytes(&self.timeout_vote.voter)
            .map_err(|_| TimeoutAuthenticationError::MalformedPublicKey)?;
        let signature = Signature::from_slice(&self.signature)
            .map_err(|_| TimeoutAuthenticationError::InvalidSignature)?;
        key.verify(&self.timeout_vote.signing_bytes(), &signature)
            .map_err(|_| TimeoutAuthenticationError::InvalidSignature)?;
        Ok(VerifiedTimeoutVote {
            timeout_vote: self.timeout_vote.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusEvent {
    AdmitBlock(Block),
    AdmitVerifiedVote(VerifiedVote),
    AdmitTimeoutVote(VerifiedTimeoutVote),
    ObserveLegitimacy(LegitimacyCertificate),
    AdvanceRound { height: u64, round: u64 },
    EvaluateFinality { block_hash: [u8; 32] },
    PruneFinalizedState { finalized_height: u64 },
    RecoverPersistedEvent { event_hash: [u8; 32] },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelCertificate {
    Execution(QuorumCertificate),
    Legitimacy(LegitimacyCertificate),
    Continuity(ContinuityCertificate),
    Constitutional(ConstitutionalSeal),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelEffect {
    BlockAccepted([u8; 32]),
    VoteAccepted([u8; 32]),
    TimeoutAccepted([u8; 32]),
    RoundAdvanced { height: u64, round: u64 },
    StateRecovered([u8; 32]),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelRejection {
    UnknownParent,
    DuplicateArtifact,
    InvalidSignature,
    StaleArtifact,
    FinalityConflict,
    InvariantViolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruningAction {
    pub pruned_blocks: usize,
    pub pruned_votes: usize,
    pub pruned_timeouts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantStatus {
    pub conflicting_finality_detected: bool,
    pub stale_branch_reactivated: bool,
    pub replay_diverged: bool,
}

impl InvariantStatus {
    #[must_use]
    pub const fn healthy() -> Self {
        Self {
            conflicting_finality_detected: false,
            stale_branch_reactivated: false,
            replay_diverged: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionResult {
    pub accepted_effects: Vec<KernelEffect>,
    pub rejected_reason: Option<KernelRejection>,
    pub emitted_certificates: Vec<KernelCertificate>,
    pub pruning_actions: Vec<PruningAction>,
    pub invariant_status: InvariantStatus,
}

impl TransitionResult {
    #[must_use]
    pub fn accepted(effect: KernelEffect) -> Self {
        Self {
            accepted_effects: vec![effect],
            rejected_reason: None,
            emitted_certificates: Vec::new(),
            pruning_actions: Vec::new(),
            invariant_status: InvariantStatus::healthy(),
        }
    }

    #[must_use]
    pub fn rejected(reason: KernelRejection) -> Self {
        Self {
            accepted_effects: Vec::new(),
            rejected_reason: Some(reason),
            emitted_certificates: Vec::new(),
            pruning_actions: Vec::new(),
            invariant_status: InvariantStatus::healthy(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};

    use crate::constitutional::{
        ConstitutionalSeal, ContinuityCertificate, ExecutionCertificate, LegitimacyCertificate,
    };
    use crate::seal::QuorumCertificate;
    use crate::vote::{SignedVote, Vote, VoteKind};

    use super::{
        InvariantStatus, KernelCertificate, KernelEffect, KernelRejection, SignedTimeoutVote,
        TimeoutAuthenticationError, TimeoutVote, TransitionResult,
    };

    #[test]
    fn accepted_transition_result_is_explicit_and_healthy() {
        let result = TransitionResult::accepted(KernelEffect::BlockAccepted([1u8; 32]));

        assert_eq!(result.accepted_effects.len(), 1);
        assert!(result.rejected_reason.is_none());
        assert_eq!(result.invariant_status, InvariantStatus::healthy());
    }

    #[test]
    fn rejected_transition_result_carries_explicit_reason() {
        let result = TransitionResult::rejected(KernelRejection::StaleArtifact);

        assert!(result.accepted_effects.is_empty());
        assert_eq!(result.rejected_reason, Some(KernelRejection::StaleArtifact));
    }

    #[test]
    fn signed_timeout_vote_verifies() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let timeout_vote = TimeoutVote {
            block_hash: [2u8; 32],
            height: 3,
            round: 4,
            epoch: 5,
            timeout_round: 6,
            voter: signing_key.verifying_key().to_bytes(),
        };
        let signature = signing_key
            .sign(&timeout_vote.signing_bytes())
            .to_bytes()
            .to_vec();

        let verified = SignedTimeoutVote {
            timeout_vote,
            signature,
        }
        .verify();
        assert!(verified.is_ok());
    }

    #[test]
    fn modified_timeout_vote_breaks_signature() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let mut timeout_vote = TimeoutVote {
            block_hash: [2u8; 32],
            height: 3,
            round: 4,
            epoch: 5,
            timeout_round: 6,
            voter: signing_key.verifying_key().to_bytes(),
        };
        let signature = signing_key
            .sign(&timeout_vote.signing_bytes())
            .to_bytes()
            .to_vec();
        timeout_vote.timeout_round = 7;

        let verified = SignedTimeoutVote {
            timeout_vote,
            signature,
        }
        .verify();
        assert_eq!(verified, Err(TimeoutAuthenticationError::InvalidSignature));
    }

    #[test]
    fn constitutional_certificate_variant_can_be_emitted() {
        let qc = QuorumCertificate::new([1u8; 32], 2, 3, vec![[9u8; 32]], 10, 10, 2, 3);
        let execution = ExecutionCertificate::new(4, [2u8; 32], qc);
        let legitimacy = LegitimacyCertificate::new(
            [1u8; 32],
            4,
            [3u8; 32],
            [4u8; 32],
            [5u8; 32],
            vec![[9u8; 32]],
        );
        let continuity = ContinuityCertificate::new([1u8; 32], 2, 3, 4, 4, 10, vec![[9u8; 32]]);
        let seal = ConstitutionalSeal::compose(&execution, &legitimacy, &continuity).unwrap();

        let result = TransitionResult {
            accepted_effects: vec![KernelEffect::BlockAccepted([1u8; 32])],
            rejected_reason: None,
            emitted_certificates: vec![KernelCertificate::Constitutional(seal)],
            pruning_actions: Vec::new(),
            invariant_status: InvariantStatus::healthy(),
        };

        assert_eq!(result.emitted_certificates.len(), 1);
    }

    #[test]
    fn verified_vote_is_produced_from_real_signature() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let vote = Vote {
            voter: signing_key.verifying_key().to_bytes(),
            block_hash: [4u8; 32],
            height: 9,
            round: 2,
            kind: VoteKind::Commit,
        };
        let signature = signing_key.sign(&vote.signing_bytes()).to_bytes();
        let verified = SignedVote {
            vote,
            signature: signature.to_vec(),
        }
        .verify();
        assert!(verified.is_ok());
    }
}
