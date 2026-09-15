use fabric_test_resource_clock::ClockRealization;

use crate::MemoryClock;

#[test]
fn memory_clock_adapter_ticks_monotonically() {
    let adapter = MemoryClock::new(3);

    assert_eq!(adapter.current_tick().expect("first tick").value(), 3);
    assert_eq!(adapter.current_tick().expect("second tick").value(), 4);
    adapter.clear().expect("clear");
    assert_eq!(adapter.current_tick().expect("reset tick").value(), 3);
}
