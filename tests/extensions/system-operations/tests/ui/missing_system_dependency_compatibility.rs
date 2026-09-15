use fabric_test_system_operations::{OperationMarker, TestOperations};

fabric::system! {
    pub MissingSystemDependencyCompatibility {
        id: "fabric.test.system.missing-dependency-compatibility";

        schema: provisional;

        config {}

        system {
            base: TestOperations();
        }

        contracts {
            primary Api {
                id: "fabric.test.system.missing-dependency-compatibility.contract";
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
