use dioxus::prelude::*;
use crate::types::*;

pub struct GlobalChainState {
    pub height: ReadOnlySignal<u64>,
    pub lanes: ReadOnlySignal<Vec<LaneStatus>>,
    pub total_staked: ReadOnlySignal<u128>,
    pub network_health: ReadOnlySignal<f32>,
}

pub fn provide_chain_context() {
    // Bu sinyaller gerçekte src/services/rpc_client.rs üzerinden güncellenir
    provide_context(Signal::new(GlobalChainState {
        height: Signal::new(1_450_982).into(),
        lanes: Signal::new(vec![
            LaneStatus { kind: LaneKind::EVM, tps: 850.5, load_percent: 34, is_active: true, last_checkpoint: "0x...a1".into() },
            LaneStatus { kind: LaneKind::Move, tps: 1240.2, load_percent: 67, is_active: true, last_checkpoint: "0x...f4".into() },
            LaneStatus { kind: LaneKind::WASM, tps: 0.0, load_percent: 0, is_active: false, last_checkpoint: "N/A".into() },
        ]).into(),
        total_staked: Signal::new(12_500_000_000).into(),
        network_health: Signal::new(99.98).into(),
    }));
}
