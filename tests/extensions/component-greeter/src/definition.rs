#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GreeterInput {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GreeterOutput {
    pub message: String,
}

fabric_sdk::component! {
    pub Greeter {
        id: "fabric.test.greeter";

        config {}

        operations {
            greet {
                id: "fabric.test.greeter.greet";
                input: GreeterInput = "fabric.test.greeter.input";
                output: GreeterOutput = "fabric.test.greeter.output";
                handler |input: GreeterInput| async move {
                    Ok(GreeterOutput {
                        message: format!("hello, {}", input.name),
                    })
                };
            }
        }
    }
}
