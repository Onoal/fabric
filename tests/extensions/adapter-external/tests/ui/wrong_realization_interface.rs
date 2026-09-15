fabric::adapter! {
    pub WrongRealizationInterfaceAdapter
        for resource fabric_test_resource_counter::AdaptedCounter
        implements fabric_test_system_operations::AdaptedOperationsRealization
    {
        schema: provisional;
        realization: "1.0.0";

        config {
            value: u64;
        }

        runtime {
            fn current_marker(&self) -> fabric_test_system_operations::OperationMarker {
                fabric_test_system_operations::OperationMarker::new(self.config.value)
            }
        }
    }
}

fn main() {}
