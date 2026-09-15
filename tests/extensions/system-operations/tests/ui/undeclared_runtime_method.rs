use fabric_test_system_operations::OperationMarker;

fabric_sdk::system! {
    pub UndeclaredRuntimeMethod {
        id: "fabric.test.system.undeclared-runtime";

        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.system.undeclared-runtime.contract";
                version: provisional;

                fn current_marker(&self) -> OperationMarker;
            }
        }

        runtime {
            fn current_marker(&self) -> OperationMarker {
                OperationMarker::new(1)
            }

            fn extra(&self) -> OperationMarker {
                OperationMarker::new(2)
            }
        }
    }
}

fn main() {}
