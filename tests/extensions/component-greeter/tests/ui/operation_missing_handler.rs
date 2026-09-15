use fabric::component;

#[derive(Clone)]
pub struct Input;

#[derive(Clone)]
pub struct Output;

component! {
    pub OperationMissingHandler {
        id: "fabric.test.component.operation-missing-handler";

        config {}

        operations {
            greet {
                id: "fabric.test.component.operation-missing-handler.greet";
                input: Input = "fabric.test.component.operation-missing-handler.input";
                output: Output = "fabric.test.component.operation-missing-handler.output";
            }
        }
    }
}

fn main() {}
