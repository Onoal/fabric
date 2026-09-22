use fabric::component;

component! {
    HandlerInApi {
        id: "fabric.test.component.handler-in-api";
        api {
            fn call(&self, input: String) -> String {
                input
            }
        }
    }
}

fn main() {}
