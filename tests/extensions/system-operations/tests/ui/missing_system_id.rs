use fabric_test_system_operations::OperationMarker;

fabric_sdk::system! {
    pub MissingSystemId {
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.system.missing-id";
                version: provisional;

                fn current_marker(&self) -> OperationMarker;
            }
        }

        runtime {
            fn current_marker(&self) -> OperationMarker {
                OperationMarker::new(1)
            }
        }
    }
}

fn main() {}
