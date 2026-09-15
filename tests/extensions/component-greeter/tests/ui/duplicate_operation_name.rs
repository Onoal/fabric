use fabric_sdk::component;

#[derive(Clone)]
pub struct Input;

#[derive(Clone)]
pub struct Output;

component! {
    pub DuplicateOperationName {
        id: "fabric.test.component.duplicate-operation-name";

        config {}

        operations {
            greet {
                id: "fabric.test.component.duplicate-operation-name.first";
                input: Input = "fabric.test.component.duplicate-operation-name.first.input";
                output: Output = "fabric.test.component.duplicate-operation-name.first.output";
                handler |input: Input| async move {
                    let _ = input;
                    Ok(Output)
                };
            }

            greet {
                id: "fabric.test.component.duplicate-operation-name.second";
                input: Input = "fabric.test.component.duplicate-operation-name.second.input";
                output: Output = "fabric.test.component.duplicate-operation-name.second.output";
                handler |input: Input| async move {
                    let _ = input;
                    Ok(Output)
                };
            }
        }
    }
}

fn main() {}
