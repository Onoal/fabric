fabric::adapter! {
    pub DestructuredArgumentAdapter for system fabric_test_system_operations::AdaptedOperations implements fabric_test_system_operations::AdaptedOperationsRealization {
        schema: "^2";
        realization: "1.0.0";

        config {
            value: u64;
        }

        runtime {
            fn current_marker(
                &self,
                (value,): (u64,),
            ) -> fabric_test_system_operations::OperationMarker {
                fabric_test_system_operations::OperationMarker::new(value + self.config.value)
            }
        }
    }
}

fn main() {}
