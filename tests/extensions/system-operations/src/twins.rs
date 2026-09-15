fabric_sdk::system! {
    pub AlphaOperations {
        id: "fabric.test.system.alpha";

        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.system.alpha.contract";
                version: provisional;

                fn current_marker(&self) -> crate::OperationMarker;
            }
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                crate::OperationMarker::new(self.config.value)
            }
        }
    }
}

fabric_sdk::system! {
    pub BetaOperations {
        id: "fabric.test.system.beta";

        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.system.beta.contract";
                version: provisional;

                fn current_marker(&self) -> crate::OperationMarker;
            }
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                crate::OperationMarker::new(self.config.value + 1)
            }
        }
    }
}
