use fabric::component;

#[derive(Clone)]
pub struct Input;

#[derive(Clone)]
pub struct Output;

component! {
    pub MissingComponentId {
        config {}

        operations {
            greet {
                id: "fabric.test.component.missing-id.greet";
                input: Input = "fabric.test.component.missing-id.input";
                output: Output = "fabric.test.component.missing-id.output";
                handler |_: Input| async move { Ok(Output) };
            }
        }
    }
}

fn main() {}
