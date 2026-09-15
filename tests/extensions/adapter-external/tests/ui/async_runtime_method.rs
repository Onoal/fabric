fabric_sdk::adapter! {
    pub AsyncRuntimeAdapter for resource fabric_test_resource_counter::AdaptedCounter implements fabric_test_resource_counter::AdaptedCounterRealization {
        schema: provisional;
        realization: "1.0.0";

        config {
            value: u64;
        }

        runtime {
            async fn current_value(&self) -> fabric_test_resource_counter::CounterValue {
                fabric_test_resource_counter::CounterValue::new(self.config.value)
            }
        }
    }
}

fn main() {}
