use std::sync::Arc;

/// Minimal memory manager facade for future mmap-backed block/state regions.
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    data: Arc<[u8]>,
}

impl MemoryRegion {
    pub fn new_zeroed(size: usize) -> Self {
        let data = vec![0_u8; size].into();
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
