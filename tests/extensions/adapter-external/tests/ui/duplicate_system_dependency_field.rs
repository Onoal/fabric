fabric::adapter! {
    pub DuplicateSystemDependencyAdapter for system fabric_test_system_operations::AdaptedOperations implements fabric_test_system_operations::AdaptedOperationsRealization {
        schema: "^2";
        realization: "1.0.0";

        config {
            value: u64;
        }

        system {
            operations: fabric_test_system_operations::TestOperations(version = "^1.2");
            operations: fabric_test_system_operations::TestOperations(version = "^1");
        }

        runtime {
            fn current_marker(&self) -> fabric_test_system_operations::OperationMarker {
                fabric_test_system_operations::OperationMarker::new(
                    self.operations.current_marker().value() + self.config.value,
                )
            }
        }
    }
}

fn main() {}
