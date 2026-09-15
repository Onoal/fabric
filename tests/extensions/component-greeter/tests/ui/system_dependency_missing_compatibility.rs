use fabric_sdk::component;
use fabric_test_system_operations::TestOperations;

struct Input;
struct Output;

component! {
    MissingSystemCompatibility {
        id: "fabric.test.missing-system-compatibility";
        config {}
        system { operations: TestOperations; }
        operations {
            run {
                id: "fabric.test.missing-system-compatibility.run";
                input: Input = "fabric.test.missing-system-compatibility.input";
                output: Output = "fabric.test.missing-system-compatibility.output";
                handler |dependencies, input: Input| async move {
                    let _ = (dependencies, input);
                    Ok(Output)
                };
            }
        }
    }
}

fn main() {}
