use fabric_test_system_operations::OperationMarker;

fabric_sdk::system! {
    pub DuplicateAdapterSection {
        id: "fabric.test.system.duplicate-adapter-section";

        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.system.duplicate-adapter-section.contract";
                version: provisional;

                fn current_marker(&self) -> OperationMarker;
            }
        }

        adapter PrimaryAdapter {
            id: "fabric.test.system.duplicate-adapter-section.realization.a";
            compatibility: "^1";

            fn current_marker(&self) -> OperationMarker;
        }

        adapter SecondaryAdapter {
            id: "fabric.test.system.duplicate-adapter-section.realization.b";
            compatibility: "^1";

            fn current_marker(&self) -> OperationMarker;
        }

        runtime {
            fn current_marker(&self) -> OperationMarker {
                self.primary_adapter.current_marker()
            }
        }
    }
}

fn main() {}
