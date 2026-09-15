use fabric_test_adapter_clock_memory::MemoryClock;
use fabric_test_resource_counter::{DirectCounter, DirectCounterConfig};

fn main() {
    let _ = DirectCounter::select("primary", DirectCounterConfig { value: 7 })
        .expect("selection")
        .using(MemoryClock::default());
}
