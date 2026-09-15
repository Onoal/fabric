use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperationMarker(u64);

impl OperationMarker {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

pub struct OperationSequence {
    seed: u64,
    step: u64,
    next: Arc<AtomicU64>,
}

impl OperationSequence {
    pub fn new(seed: u64, step: u64) -> Self {
        Self {
            seed,
            step,
            next: Arc::new(AtomicU64::new(seed)),
        }
    }

    pub fn current_marker(&self) -> OperationMarker {
        OperationMarker::new(self.next.fetch_add(self.step, Ordering::Relaxed))
    }
}

impl Clone for OperationSequence {
    fn clone(&self) -> Self {
        Self::new(self.seed, self.step)
    }
}
