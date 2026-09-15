fabric::system! {
    pub DerivedOperations {
        id: "fabric.test.operations.derived";

        schema: provisional;

        config {
            offset: u64;
        }

        system {
            base: crate::TestOperations(version = "^1.2");
        }

        contracts {
            primary Api {
                id: "fabric.test.system.operations.derived";
                version: provisional;

                fn current_marker(&self) -> crate::OperationMarker;
                fn source_provider(&self) -> String;
                fn source_identity(&self) -> String;
            }
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                crate::OperationMarker::new(self.base.current_marker().value() + self.config.offset)
            }

            fn source_provider(&self) -> String {
                self.base.provider().as_str().to_owned()
            }

            fn source_identity(&self) -> String {
                self.base.identity().to_string()
            }
        }
    }
}
