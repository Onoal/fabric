#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GreeterInput {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GreeterOutput {
    pub message: String,
}

fabric::component! {
    pub Greeter {
        id: "fabric.test.greeter";

        config {}

        api { fn greet(&self, input: GreeterInput) -> GreeterOutput; }

        runtime {
            fn greet(&self, input: GreeterInput) -> GreeterOutput {
                GreeterOutput { message: format!("hello, {}", input.name) }
            }
        }
    }
}
