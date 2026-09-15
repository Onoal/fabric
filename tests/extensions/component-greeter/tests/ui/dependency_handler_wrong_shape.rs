use fabric_sdk::component;
use fabric_test_resource_counter::DirectCounter;

struct Input;
struct Output;

component! {
    BadHandler {
        id: "fabric.test.bad-handler";
        config {}
        requires { counter: DirectCounter(provisional); }
        operations {
            run {
                id: "fabric.test.bad-handler.run";
                input: Input = "fabric.test.bad-handler.input";
                output: Output = "fabric.test.bad-handler.output";
                handler |input: Input| async move { Ok(Output) };
            }
        }
    }
}

fn main() {}
