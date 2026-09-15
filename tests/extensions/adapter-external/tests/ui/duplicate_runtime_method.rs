fabric::adapter! {
    pub DuplicateRuntimeAdapter for resource fabric_test_resource_counter::AdaptedCounter implements fabric_test_resource_counter::AdaptedCounterRealization {
        schema: provisional;
        realization: "1.0.0";

        config {
            value: u64;
        }

        runtime {
            fn current_value(&self) -> fabric_test_resource_counter::CounterValue {
                fabric_test_resource_counter::CounterValue::new(self.config.value)
            }

            fn current_value(&self) -> fabric_test_resource_counter::CounterValue {
                fabric_test_resource_counter::CounterValue::new(self.config.value + 1)
            }
        }
    }
}

fn main() {}
