use fabric::component;
use fabric_test_resource_counter::DirectCounter;

struct Input;
struct Output;

component! {
    DuplicateDependencyField {
        id: "fabric.test.duplicate-dependency-field";
        config {}
        requires {
            first: DirectCounter(provisional);
            first: DirectCounter(provisional);
        }
        operations {
            run {
                id: "fabric.test.duplicate-dependency-field.run";
                input: Input = "fabric.test.duplicate-dependency-field.input";
                output: Output = "fabric.test.duplicate-dependency-field.output";
                handler |dependencies, input: Input| async move {
                    let _ = (dependencies, input);
                    Ok(Output)
                };
            }
        }
    }
}

fn main() {}
