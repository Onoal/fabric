use fabric_test_system_operations::{AdaptedOperations as ExternalOperations, OperationMarker};

fabric::adapter! {
    pub ExternalOperationsAdapter for ExternalOperations {

        config {
            value: u64;
        }

        runtime {
            fn current_marker(&self) -> OperationMarker {
                OperationMarker::new(self.config.value)
            }
        }
    }
}

fabric::adapter! {
    pub ExternalOperationsWithSystemAdapter for ExternalOperations {

        config {
            offset: u64;
        }

        relations { requires { operations: fabric_test_system_operations::TestOperations(version = "^1"); } }

        runtime {
            fn current_marker(&self) -> OperationMarker {
                OperationMarker::new(self.operations.current_marker().value() + self.config.offset)
            }
        }
    }
}
