use fabric_test_system_operations::OperationMarker;

fabric::system! {
    pub AsyncContractMethod {
        id: "fabric.test.system.async-contract";

        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.system.async-contract.contract";
                version: provisional;

                async fn current_marker(&self) -> OperationMarker;
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
