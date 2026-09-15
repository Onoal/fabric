use fabric::component;
use fabric_test_resource_counter::DirectCounter;

struct Input;
struct Output;

component! {
    MissingResourceCompatibility {
        id: "fabric.test.missing-resource-compatibility";
        config {}
        requires { counter: DirectCounter; }
        operations {
            run {
                id: "fabric.test.missing-resource-compatibility.run";
                input: Input = "fabric.test.missing-resource-compatibility.input";
                output: Output = "fabric.test.missing-resource-compatibility.output";
                handler |dependencies, input: Input| async move {
                    let _ = (dependencies, input);
                    Ok(Output)
                };
            }
        }
    }
}

fn main() {}
