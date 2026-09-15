use fabric_test_system_operations::{
    AdaptedOperations as ExternalOperations,
    AdaptedOperationsRealization as ExternalOperationsRealization, OperationMarker,
};

fabric::adapter! {
    pub ExternalOperationsAdapter for system ExternalOperations implements ExternalOperationsRealization {
        schema: "^2";
        realization: "1.0.0";

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
    pub ExternalOperationsWithSystemAdapter for system ExternalOperations implements ExternalOperationsRealization {
        schema: "^2";
        realization: "1.0.0";

        config {
            offset: u64;
        }

        system {
            operations: fabric_test_system_operations::TestOperations(version = "^1.2");
        }

        runtime {
            fn current_marker(&self) -> OperationMarker {
                OperationMarker::new(self.operations.current_marker().value() + self.config.offset)
            }
        }
    }
}
