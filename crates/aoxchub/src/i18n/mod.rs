// src/state.rs
use dioxus::prelude::*;
use crate::i18n::Language;

#[derive(Clone, Copy)]
pub struct ChainState {
    pub block_height: ReadOnlySignal<u64>,
    pub sync_status: ReadOnlySignal<f32>,
    pub is_connected: ReadOnlySignal<bool>,
}

pub fn provide_global_state() {
    provide_context(Signal::new(Language::TR));
    // Gelecekte buraya RPC üzerinden gelen canlı veriler bağlanacak
    provide_context(ChainState {
        block_height: Signal::new(1_245_789).into(),
        sync_status: Signal::new(99.8).into(),
        is_connected: Signal::new(true).into(),
    });
}
