use fabric::component;

#[derive(Clone)]
pub struct Input;

#[derive(Clone)]
pub struct Output;

component! {
    pub OperationMissingInput {
        id: "fabric.test.component.operation-missing-input";

        config {}

        operations {
            greet {
                id: "fabric.test.component.operation-missing-input.greet";
                output: Output = "fabric.test.component.operation-missing-input.output";
                handler |_: Input| async move { Ok(Output) };
            }
        }
    }
}

fn main() {}
