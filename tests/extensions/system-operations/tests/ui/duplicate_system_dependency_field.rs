use fabric_test_system_operations::{OperationMarker, TestOperations};

fabric::system! {
    pub DuplicateSystemDependencyField {
        id: "fabric.test.system.duplicate-dependency-field";

        schema: provisional;

        config {}

        system {
            base: TestOperations(provisional);
            base: TestOperations(version = "^1");
        }

        contracts {
            primary Api {
                id: "fabric.test.system.duplicate-dependency-field.contract";
                version: provisional;

                fn current_marker(&self) -> OperationMarker;
            }
        }

        runtime {
            fn current_marker(&self) -> OperationMarker {
                self.base.current_marker()
            }
        }
    }
}

fn main() {}
