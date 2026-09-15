use fabric::component;

#[derive(Clone)]
pub struct Input;

#[derive(Clone)]
pub struct Output;

component! {
    pub OperationMissingOutput {
        id: "fabric.test.component.operation-missing-output";

        config {}

        operations {
            greet {
                id: "fabric.test.component.operation-missing-output.greet";
                input: Input = "fabric.test.component.operation-missing-output.input";
                handler |_: Input| async move { Ok(Output) };
            }
        }
    }
}

fn main() {}
