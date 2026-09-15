use fabric_test_adapter_clock_memory::MemoryClock;
use fabric_test_system_operations::{AdaptedOperations, AdaptedOperationsConfig};

fn main() {
    let _ = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("system selection")
        .using(MemoryClock::default());
}
