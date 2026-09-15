mod host_bound;
mod memory;

#[cfg(test)]
mod tests;

pub use host_bound::{HostBoundClockAdapter, host_bound_clock_facility};
pub use memory::MemoryClock;
