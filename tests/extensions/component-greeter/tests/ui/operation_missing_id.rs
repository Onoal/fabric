use fabric::component;

#[derive(Clone)]
pub struct Input;

#[derive(Clone)]
pub struct Output;

component! {
    pub OperationMissingId {
        id: "fabric.test.component.operation-missing-id";

        config {}

        operations {
            greet {
                input: Input = "fabric.test.component.operation-missing-id.input";
                output: Output = "fabric.test.component.operation-missing-id.output";
                handler |_: Input| async move { Ok(Output) };
            }
        }
    }
}

fn main() {}
