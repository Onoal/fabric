fabric_sdk::adapter! {
    pub MutSelfRuntimeAdapter for resource fabric_test_resource_counter::AdaptedCounter implements fabric_test_resource_counter::AdaptedCounterRealization {
        schema: provisional;
        realization: "1.0.0";

        config {
            value: u64;
        }

        runtime {
            fn current_value(&mut self) -> fabric_test_resource_counter::CounterValue {
                fabric_test_resource_counter::CounterValue::new(self.config.value)
            }
        }
    }
}

fn main() {}
