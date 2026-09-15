use fabric_test_resource_clock::{Clock, ClockConfig};
use fabric_test_system_operations::{FixedOperationsAdapter, FixedOperationsAdapterConfig};

fn main() {
    let _ = Clock::select("primary", ClockConfig::default())
        .expect("selection")
        .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
            value: 7,
        }));
}
