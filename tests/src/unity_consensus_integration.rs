use std::{
    env, fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use aoxcmd::{
    data_home::ensure_layout,
    keys::manager::bootstrap_operator_key,
    node::{engine::produce_once, lifecycle::bootstrap_state},
};
use aoxcnet::{
    config::{ExternalDomainKind, NetworkConfig},
    gossip::{
        consensus_gossip::GossipEngine,
        peer::{NodeCertificate, Peer, PeerRole},
    },
};
use aoxcunity::{
    AuthenticatedVote, Block, BlockBody, ConsensusEngine, ConsensusEvent, ConsensusJournal,
    ConsensusMessage, ConsensusState, ConstitutionalSeal, FinalityStore, KernelCertificate,
    KernelEffect, KernelSnapshot, LaneCommitment, LaneCommitmentSection, LaneType,
    LegitimacyCertificate, PersistedConsensusEvent, Proposer, QuorumThreshold, RecoveryState,
    SnapshotStore, Validator, ValidatorRole, ValidatorRotation, VerifiedAuthenticatedVote,
    VerifiedTimeoutVote, VerifiedVote, Vote, VoteAuthenticationContext, VoteKind,
    store::EvidenceStore,
};
use ed25519_dalek::{Signer, SigningKey};

#[test]
fn unity_consensus_flow_integrates_cmd_network_and_finality() {
    let _lock = env_lock().lock().expect("env mutex must not be poisoned");
    let home = unique_test_home("unity-consensus");
    let previous_home = env::var_os("AOXC_HOME");

    ensure_layout(&home).expect("test AOXC home layout should be created");
    unsafe {
        env::set_var("AOXC_HOME", &home);
    }

    let test_result = (|| {
        let key_material = bootstrap_operator_key("validator-01", "testnet", "Test#2026!")
            .expect("operator key bootstrap should succeed");
        let key_summary = key_material
            .summary()
            .expect("operator key summary should stay derivable");

        let bootstrapped = bootstrap_state().expect("node state bootstrap should succeed");
        assert_eq!(bootstrapped.consensus.network_id, 2626);

        let produced =
            produce_once("integration-lifecycle").expect("single block production should succeed");
        assert_eq!(produced.current_height, 1);
        assert_eq!(produced.produced_blocks, 1);
        assert_eq!(produced.consensus.last_message_kind, "block_proposal");
        assert_eq!(
            produced.key_material.consensus_public_key_hex,
            key_summary.consensus_public_key
        );

        let proposer_key =
            decode_hex32(&key_summary.consensus_public_key, "consensus proposer key");
        let auth_context = VoteAuthenticationContext {
            network_id: 2626,
            epoch: 1,
            validator_set_root: [9u8; 32],
            signature_scheme: 1,
        };

        let validator_keys = [[1u8; 32], [2u8; 32], [3u8; 32]]
            .into_iter()
            .map(|secret| SigningKey::from_bytes(&secret))
            .collect::<Vec<SigningKey>>();
        let validators = validator_keys
            .iter()
            .map(|key: &SigningKey| {
                Validator::new(key.verifying_key().to_bytes(), 1, ValidatorRole::Validator)
            })
            .collect::<Vec<_>>();

        let rotation =
            ValidatorRotation::new(validators).expect("validator rotation should be valid");
        let mut consensus = ConsensusState::new(rotation, QuorumThreshold::two_thirds());

        let proposal = Proposer::new(2626, proposer_key)
            .propose(
                [0u8; 32],
                1,
                0,
                1,
                1_800_000_000,
                BlockBody {
                    sections: vec![aoxcunity::BlockSection::LaneCommitment(
                        LaneCommitmentSection {
                            lanes: vec![LaneCommitment {
                                lane_id: 7,
                                lane_type: LaneType::Native,
                                tx_count: 1,
                                input_root: [1u8; 32],
                                output_root: [2u8; 32],
                                receipt_root: [3u8; 32],
                                state_commitment: [4u8; 32],
                                proof_commitment: [5u8; 32],
                            }],
                        },
                    )],
                },
            )
            .expect("canonical block proposal should be built");
        consensus
            .admit_block(proposal.clone())
            .expect("proposal should be admitted");

        let mut gossip = GossipEngine::new(NetworkConfig::default());
        gossip
            .register_peer(consensus_peer())
            .expect("peer should register");
        gossip
            .establish_session("validator-1")
            .expect("peer session should establish");

        let proposal_envelope = gossip
            .broadcast_from_peer(
                "validator-1",
                ConsensusMessage::BlockProposal {
                    block: proposal.clone(),
                },
            )
            .expect("proposal should broadcast");
        assert_eq!(
            proposal_envelope.payload.canonical_bytes().first(),
            Some(&0)
        );

        for signing_key in validator_keys.iter().take(2) {
            let vote = Vote {
                voter: signing_key.verifying_key().to_bytes(),
                block_hash: proposal.hash,
                height: proposal.header.height,
                round: proposal.header.round,
                kind: VoteKind::Commit,
            };
            let signature = signing_key
                .sign(
                    &AuthenticatedVote {
                        vote: vote.clone(),
                        context: auth_context,
                        signature: Vec::new(),
                    }
                    .signing_bytes(),
                )
                .to_bytes()
                .to_vec();
            let authenticated_vote = AuthenticatedVote {
                vote,
                context: auth_context,
                signature,
            };

            let verified = authenticated_vote
                .verify()
                .expect("authenticated vote should verify");
            consensus
                .add_authenticated_vote(verified, auth_context)
                .expect("verified vote should enter consensus state");
            gossip
                .broadcast_from_peer("validator-1", ConsensusMessage::Vote(authenticated_vote))
                .expect("vote should broadcast");
        }

        assert!(consensus.has_quorum(proposal.hash, VoteKind::Commit));
        let seal = consensus
            .try_finalize(proposal.hash, proposal.header.round)
            .expect("commit quorum should finalize block");
        let certificate = consensus
            .authenticated_quorum_certificate(proposal.hash, proposal.header.round, auth_context)
            .expect("finalized block should produce authenticated QC");

        let finalize_envelope = gossip
            .broadcast_from_peer(
                "validator-1",
                ConsensusMessage::Finalize {
                    seal: seal.clone(),
                    certificate: certificate.clone(),
                },
            )
            .expect("finalize should broadcast");

        assert_eq!(
            finalize_envelope.payload.canonical_bytes().first(),
            Some(&2)
        );
        assert!(gossip.receive().is_some());
        assert!(gossip.receive().is_some());
        assert!(gossip.receive().is_some());
        assert!(matches!(
            gossip.receive(),
            Some(ConsensusMessage::Finalize { .. })
        ));
        assert_eq!(
            consensus
                .fork_choice
                .finalized_head()
                .expect("finalized head should exist"),
            proposal.hash
        );

        Ok::<(), Box<dyn std::error::Error>>(())
    })();

    if let Some(previous) = previous_home {
        unsafe {
            env::set_var("AOXC_HOME", previous);
        }
    } else {
        unsafe {
            env::remove_var("AOXC_HOME");
        }
    }
    let _ = fs::remove_dir_all(&home);

    test_result.expect("integration flow should succeed");
}

#[test]
fn unity_kernel_records_equivocation_evidence_for_vote_and_timeout_attacks() {
    let signing_keys = validator_signing_keys();
    let mut engine = consensus_engine_from_signers(&signing_keys);

    let genesis = build_kernel_block([0u8; 32], 0, 0, [9u8; 32], 11);
    let fork_a = build_kernel_block(genesis.hash, 1, 1, [8u8; 32], 22);
    let fork_b = build_kernel_block(genesis.hash, 1, 1, [7u8; 32], 33);

    for block in [genesis.clone(), fork_a.clone()] {
        let admitted = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));
        assert!(
            admitted
                .accepted_effects
                .contains(&KernelEffect::BlockAccepted(block.hash))
        );
    }
    engine.state.blocks.insert(fork_b.hash, fork_b.clone());
    engine.state.fork_choice.insert_block(aoxcunity::BlockMeta {
        hash: fork_b.hash,
        parent: fork_b.header.parent_hash,
        height: fork_b.header.height,
        seal: None,
    });

    let honest_vote = engine.apply_event(verified_commit_event(&engine, &signing_keys[0], &fork_a));
    assert_eq!(honest_vote.rejected_reason, None);

    let equivocation =
        engine.apply_event(verified_commit_event(&engine, &signing_keys[0], &fork_b));
    assert_eq!(
        equivocation.rejected_reason,
        Some(aoxcunity::KernelRejection::InvariantViolation)
    );
    assert_eq!(engine.evidence_buffer.len(), 1);
    assert_eq!(engine.evidence_buffer[0].reason, "vote_equivocation");

    let timeout_a = engine.apply_event(timeout_vote_event(
        &signing_keys[1],
        &fork_a,
        fork_a.header.round,
        0,
        2,
    ));
    assert_eq!(timeout_a.rejected_reason, None);

    let timeout_conflict = engine.apply_event(timeout_vote_event(
        &signing_keys[1],
        &fork_b,
        fork_b.header.round,
        0,
        2,
    ));
    assert_eq!(
        timeout_conflict.rejected_reason,
        Some(aoxcunity::KernelRejection::InvariantViolation)
    );
    assert!(
        timeout_conflict
            .invariant_status
            .conflicting_finality_detected
    );
    assert_eq!(engine.evidence_buffer.len(), 2);
    assert_eq!(engine.evidence_buffer[1].reason, "timeout_equivocation");
}

#[test]
fn unity_kernel_recovery_replays_persisted_journal_and_preserves_finality() {
    let signing_keys = validator_signing_keys();
    let mut source_engine = consensus_engine_from_signers(&signing_keys);
    let mut store = MemoryRecoveryStore::default();

    let genesis = build_kernel_block([0u8; 32], 0, 0, [6u8; 32], 40);
    let block = build_kernel_block(genesis.hash, 1, 1, [5u8; 32], 41);
    let legitimacy = LegitimacyCertificate::new(
        block.hash,
        0,
        [1u8; 32],
        [2u8; 32],
        [3u8; 32],
        signing_keys
            .iter()
            .map(|key| key.verifying_key().to_bytes())
            .collect(),
    );

    let events = vec![
        ConsensusEvent::AdmitBlock(genesis.clone()),
        ConsensusEvent::AdmitBlock(block.clone()),
        ConsensusEvent::ObserveLegitimacy(legitimacy),
        timeout_vote_event(&signing_keys[0], &block, block.header.round, 0, 1),
        timeout_vote_event(&signing_keys[1], &block, block.header.round, 0, 1),
        timeout_vote_event(&signing_keys[2], &block, block.header.round, 0, 1),
        verified_commit_event(&source_engine, &signing_keys[0], &block),
        verified_commit_event(&source_engine, &signing_keys[1], &block),
        verified_commit_event(&source_engine, &signing_keys[2], &block),
        ConsensusEvent::EvaluateFinality {
            block_hash: block.hash,
        },
    ];

    let mut constitutional = None;
    for (index, event) in events.iter().cloned().enumerate() {
        let result = source_engine.apply_event(event.clone());
        store
            .append(PersistedConsensusEvent {
                sequence: index as u64,
                event_hash: [(index as u8).wrapping_add(1); 32],
                event: event.clone(),
            })
            .expect("journal append should succeed");

        if let Some(seal) =
            result
                .emitted_certificates
                .iter()
                .find_map(|certificate| match certificate {
                    KernelCertificate::Constitutional(seal) => Some(seal.clone()),
                    _ => None,
                })
        {
            constitutional = Some(seal);
        }
    }

    let finality = constitutional.expect("constitutional seal should be emitted");
    store
        .store_snapshot(KernelSnapshot {
            snapshot_height: source_engine.current_height,
            snapshot_round: source_engine.state.round.round,
            lock_state: source_engine.lock_state.clone(),
            finalized_seal: Some(finality.clone()),
        })
        .expect("snapshot store should succeed");
    store
        .store_finalized_seal(finality.clone())
        .expect("finality store should succeed");

    let recovered =
        aoxcunity::recover_state(&store, &store, &store, &store).expect("recovery should succeed");
    assert_eq!(recovered.journal.len(), 10);
    assert_eq!(recovered.finalized_seal, Some(finality.clone()));

    let mut replay_engine = consensus_engine_from_signers(&signing_keys);
    replay_recovery_state(&recovered, &mut replay_engine);

    assert_eq!(
        replay_engine.state.fork_choice.finalized_head(),
        source_engine.state.fork_choice.finalized_head()
    );
    assert_eq!(replay_engine.current_height, source_engine.current_height);
    assert_eq!(replay_engine.lock_state, source_engine.lock_state);

    let duplicate = replay_engine.apply_event(ConsensusEvent::RecoverPersistedEvent {
        event_hash: recovered.journal[0].event_hash,
    });
    assert!(duplicate.invariant_status.replay_diverged);
    assert_eq!(
        duplicate.rejected_reason,
        Some(aoxcunity::KernelRejection::DuplicateArtifact)
    );
}

#[test]
fn unity_kernel_seed_matrix_keeps_finality_deterministic_under_vote_permutations() {
    let seeds = [0_u64, 1, 7, 19, 42, 99];
    let signing_keys = validator_signing_keys();

    for seed in seeds {
        let mut engine = consensus_engine_from_signers(&signing_keys);
        let genesis = build_kernel_block([0u8; 32], 0, 0, [4u8; 32], seed as u8);
        let block = build_kernel_block(genesis.hash, 1, 1, [3u8; 32], seed.wrapping_mul(3) as u8);

        for event in [
            ConsensusEvent::AdmitBlock(genesis.clone()),
            ConsensusEvent::AdmitBlock(block.clone()),
        ] {
            let result = engine.apply_event(event);
            assert!(result.rejected_reason.is_none());
        }

        let mut order = [0usize, 1, 2];
        let rotation = (seed as usize) % order.len();
        order.rotate_left(rotation);

        for signer_index in order {
            let result = engine.apply_event(verified_commit_event(
                &engine,
                &signing_keys[signer_index],
                &block,
            ));
            assert_eq!(result.rejected_reason, None);
        }

        let finalized = engine.apply_event(ConsensusEvent::EvaluateFinality {
            block_hash: block.hash,
        });
        assert!(
            finalized
                .accepted_effects
                .contains(&KernelEffect::BlockFinalized(block.hash)),
            "seed {seed} should still finalize deterministically"
        );
        assert_eq!(engine.state.fork_choice.finalized_head(), Some(block.hash));
    }
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn unique_test_home(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be monotonic")
        .as_nanos();
    env::temp_dir().join(format!("aoxchain-{prefix}-{nanos}"))
}

fn decode_hex32(value: &str, label: &str) -> [u8; 32] {
    let bytes = hex::decode(value).unwrap_or_else(|_| panic!("{label} must decode from hex"));
    assert_eq!(bytes.len(), 32, "{label} must be exactly 32 bytes");
    let mut output = [0u8; 32];
    output.copy_from_slice(&bytes);
    output
}

fn consensus_peer() -> Peer {
    Peer::new(
        "validator-1",
        "10.0.0.1:2727",
        "AOXC-MAINNET",
        ExternalDomainKind::Native,
        PeerRole::Validator,
        3,
        true,
        NodeCertificate {
            subject: "validator-1".to_string(),
            issuer: "AOXC-ROOT".to_string(),
            valid_from_unix: 1,
            valid_until_unix: u64::MAX,
            serial: "serial-1".to_string(),
            domain_attestation_hash: "attestation-1".to_string(),
        },
    )
}

fn validator_signing_keys() -> Vec<SigningKey> {
    [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]]
        .into_iter()
        .map(|secret| SigningKey::from_bytes(&secret))
        .collect()
}

fn consensus_engine_from_signers(signers: &[SigningKey]) -> ConsensusEngine {
    let validators = signers
        .iter()
        .map(|key| Validator::new(key.verifying_key().to_bytes(), 1, ValidatorRole::Validator))
        .collect::<Vec<_>>();

    ConsensusEngine::new(ConsensusState::new(
        ValidatorRotation::new(validators).expect("validator set should be valid"),
        QuorumThreshold::two_thirds(),
    ))
}

fn build_kernel_block(
    parent_hash: [u8; 32],
    height: u64,
    round: u64,
    proposer: [u8; 32],
    lane_seed: u8,
) -> Block {
    Proposer::new(2626, proposer)
        .propose(
            parent_hash,
            height,
            0,
            round,
            1_800_000_000 + height + round,
            BlockBody {
                sections: vec![aoxcunity::BlockSection::LaneCommitment(
                    LaneCommitmentSection {
                        lanes: vec![LaneCommitment {
                            lane_id: height as u32 + 1,
                            lane_type: LaneType::Native,
                            tx_count: 1,
                            input_root: [lane_seed; 32],
                            output_root: [lane_seed.wrapping_add(1); 32],
                            receipt_root: [lane_seed.wrapping_add(2); 32],
                            state_commitment: [lane_seed.wrapping_add(3); 32],
                            proof_commitment: [lane_seed.wrapping_add(4); 32],
                        }],
                    },
                )],
            },
        )
        .expect("kernel test block should build")
}

fn verified_commit_event(
    engine: &ConsensusEngine,
    signing_key: &SigningKey,
    block: &Block,
) -> ConsensusEvent {
    ConsensusEvent::AdmitVerifiedVote(VerifiedVote {
        authenticated_vote: VerifiedAuthenticatedVote {
            vote: Vote {
                voter: signing_key.verifying_key().to_bytes(),
                block_hash: block.hash,
                height: block.header.height,
                round: block.header.round,
                kind: VoteKind::Commit,
            },
            context: VoteAuthenticationContext {
                network_id: 2626,
                epoch: engine.current_epoch,
                validator_set_root: engine.state.rotation.validator_set_hash(),
                signature_scheme: 1,
            },
        },
        verification_tag: signing_key.verifying_key().to_bytes(),
    })
}

fn timeout_vote_event(
    signing_key: &SigningKey,
    block: &Block,
    round: u64,
    epoch: u64,
    timeout_round: u64,
) -> ConsensusEvent {
    ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
        timeout_vote: aoxcunity::TimeoutVote {
            block_hash: block.hash,
            height: block.header.height,
            round,
            epoch,
            timeout_round,
            voter: signing_key.verifying_key().to_bytes(),
        },
        verification_tag: signing_key.verifying_key().to_bytes(),
    })
}

fn replay_recovery_state(recovered: &RecoveryState, engine: &mut ConsensusEngine) {
    for event in &recovered.journal {
        let _ = engine.apply_event(ConsensusEvent::RecoverPersistedEvent {
            event_hash: event.event_hash,
        });
        let _ = engine.apply_event(event.event.clone());
    }
}

#[derive(Default, Clone)]
struct MemoryRecoveryStore {
    journal: Vec<PersistedConsensusEvent>,
    snapshot: Option<KernelSnapshot>,
    finality: Option<ConstitutionalSeal>,
}

impl aoxcunity::ConsensusJournal for MemoryRecoveryStore {
    fn append(&mut self, event: PersistedConsensusEvent) -> Result<(), String> {
        self.journal.push(event);
        Ok(())
    }

    fn load_all(&self) -> Result<Vec<PersistedConsensusEvent>, String> {
        Ok(self.journal.clone())
    }
}

impl SnapshotStore for MemoryRecoveryStore {
    fn store_snapshot(&mut self, snapshot: KernelSnapshot) -> Result<(), String> {
        self.snapshot = Some(snapshot);
        Ok(())
    }

    fn load_snapshot(&self) -> Result<Option<KernelSnapshot>, String> {
        Ok(self.snapshot.clone())
    }
}

impl EvidenceStore for MemoryRecoveryStore {
    fn append_evidence(&mut self, _evidence: aoxcunity::ConsensusEvidence) -> Result<(), String> {
        Ok(())
    }

    fn load_evidence(&self) -> Result<Vec<aoxcunity::ConsensusEvidence>, String> {
        Ok(Vec::new())
    }
}

impl FinalityStore for MemoryRecoveryStore {
    fn store_finalized_seal(&mut self, seal: ConstitutionalSeal) -> Result<(), String> {
        self.finality = Some(seal);
        Ok(())
    }

    fn load_finalized_seal(&self) -> Result<Option<ConstitutionalSeal>, String> {
        Ok(self.finality.clone())
    }
}
