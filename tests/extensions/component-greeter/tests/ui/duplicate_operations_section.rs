use fabric::component;

#[derive(Clone)]
pub struct Input;

#[derive(Clone)]
pub struct Output;

component! {
    pub DuplicateOperationsSection {
        id: "fabric.test.component.duplicate-operations";

        config {}

        operations {
            greet {
                id: "fabric.test.component.duplicate-operations.greet";
                input: Input = "fabric.test.component.duplicate-operations.input";
                output: Output = "fabric.test.component.duplicate-operations.output";
                handler |_: Input| async move { Ok(Output) };
            }
        }

        operations {}
    }
}

fn main() {}
