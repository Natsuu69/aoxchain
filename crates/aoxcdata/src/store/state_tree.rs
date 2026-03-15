use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Deterministic state tree surface with a Merkle-like root hash.
#[derive(Debug, Default, Clone)]
pub struct StateTree {
    nodes: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl StateTree {
    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.nodes.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.nodes.get(key).map(Vec::as_slice)
    }

    pub fn root_hash_hex(&self) -> String {
        let mut h = Sha256::new();
        for (k, v) in &self.nodes {
            h.update((k.len() as u64).to_le_bytes());
            h.update(k);
            h.update((v.len() as u64).to_le_bytes());
            h.update(v);
        }
        hex::encode(h.finalize())
    }
}
