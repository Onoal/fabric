use fabric_test_system_operations::OperationMarker;

fabric::system! {
    pub MissingRuntimeMethod {
        id: "fabric.test.system.missing-runtime";

        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.system.missing-runtime.contract";
                version: provisional;

                fn current_marker(&self) -> OperationMarker;
            }
        }

        runtime {}
    }
}

fn main() {}
