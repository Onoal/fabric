fabric::system! {
    pub AlphaOperations {
        id: "fabric.test.system.alpha";


        config {
            value: u64;
        }

        api {
            fn current_marker(&self) -> crate::OperationMarker;
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                crate::OperationMarker::new(self.config.value)
            }
        }
    }
}

fabric::system! {
    pub BetaOperations {
        id: "fabric.test.system.beta";


        config {
            value: u64;
        }

        api {
            fn current_marker(&self) -> crate::OperationMarker;
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                crate::OperationMarker::new(self.config.value + 1)
            }
        }
    }
}
