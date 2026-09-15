use fabric_sdk::component;

#[derive(Clone)]
pub struct Input;

#[derive(Clone)]
pub struct Output;

fn greet(_: Input) {}

component! {
    pub UnsupportedHandlerExpression {
        id: "fabric.test.component.unsupported-handler";

        config {}

        operations {
            greet {
                id: "fabric.test.component.unsupported-handler.greet";
                input: Input = "fabric.test.component.unsupported-handler.input";
                output: Output = "fabric.test.component.unsupported-handler.output";
                handler greet;
            }
        }
    }
}

fn main() {}
