use fabric::component;

struct Input;
struct Output;

component! {
    DuplicateOperationContext {
        id: "fabric.test.duplicate-operation-context";
        config {}
        operations {
            run {
                id: "fabric.test.duplicate-operation-context.run";
                input: Input = "fabric.test.duplicate-operation-context.input";
                output: Output = "fabric.test.duplicate-operation-context.output";
                context: invocation;
                context: invocation;
                handler |context, input: Input| async move { let _ = context; Ok(Output) };
            }
        }
    }
}

fn main() {}
