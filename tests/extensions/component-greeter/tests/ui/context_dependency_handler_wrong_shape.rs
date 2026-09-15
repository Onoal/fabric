use fabric::component;
use fabric_test_resource_counter::DirectCounter;

struct Input;
struct Output;

component! {
    BadContextDependencyHandler {
        id: "fabric.test.bad-context-dependency-handler";
        config {}
        requires { counter: DirectCounter(provisional); }
        operations {
            run {
                id: "fabric.test.bad-context-dependency-handler.run";
                input: Input = "fabric.test.bad-context-dependency-handler.input";
                output: Output = "fabric.test.bad-context-dependency-handler.output";
                context: invocation;
                handler |context, input: Input| async move { let _ = context; Ok(Output) };
            }
        }
    }
}

fn main() {}
