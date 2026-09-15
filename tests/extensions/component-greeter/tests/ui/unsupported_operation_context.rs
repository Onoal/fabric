use fabric_sdk::component;

struct Input;
struct Output;

component! {
    UnsupportedOperationContext {
        id: "fabric.test.unsupported-operation-context";
        config {}
        operations {
            run {
                id: "fabric.test.unsupported-operation-context.run";
                input: Input = "fabric.test.unsupported-operation-context.input";
                output: Output = "fabric.test.unsupported-operation-context.output";
                context: external;
                handler |context, input: Input| async move { let _ = context; Ok(Output) };
            }
        }
    }
}

fn main() {}
