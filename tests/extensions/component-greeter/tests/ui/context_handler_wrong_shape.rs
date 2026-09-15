use fabric_sdk::component;

struct Input;
struct Output;

component! {
    BadContextHandler {
        id: "fabric.test.bad-context-handler";
        config {}
        operations {
            run {
                id: "fabric.test.bad-context-handler.run";
                input: Input = "fabric.test.bad-context-handler.input";
                output: Output = "fabric.test.bad-context-handler.output";
                context: invocation;
                handler |input: Input| async move { Ok(Output) };
            }
        }
    }
}

fn main() {}
