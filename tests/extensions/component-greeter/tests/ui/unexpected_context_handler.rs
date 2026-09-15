use fabric::component;

struct Input;
struct Output;

component! {
    UnexpectedContextHandler {
        id: "fabric.test.unexpected-context-handler";
        config {}
        operations {
            run {
                id: "fabric.test.unexpected-context-handler.run";
                input: Input = "fabric.test.unexpected-context-handler.input";
                output: Output = "fabric.test.unexpected-context-handler.output";
                handler |context, input: Input| async move { let _ = context; Ok(Output) };
            }
        }
    }
}

fn main() {}
