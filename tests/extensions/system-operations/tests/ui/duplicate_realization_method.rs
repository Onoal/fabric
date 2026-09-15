use fabric_test_system_operations::OperationMarker;

fabric_sdk::system! {
    pub DuplicateRealizationMethod {
        id: "fabric.test.system.duplicate-realization-method";

        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.system.duplicate-realization-method.contract";
                version: provisional;

                fn current_marker(&self) -> OperationMarker;
            }
        }

        adapter Adapter {
            id: "fabric.test.system.duplicate-realization-method.realization";
            compatibility: "^1";

            fn current_marker(&self) -> OperationMarker;
            fn current_marker(&self) -> OperationMarker;
        }

        runtime {
            fn current_marker(&self) -> OperationMarker {
                self.adapter.current_marker()
            }
        }
    }
}

fn main() {}
