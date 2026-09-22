use fabric::component;

component! {
    DuplicateApiMethod {
        id: "fabric.test.component.duplicate-api-method";
        api {
            fn call(&self, input: String) -> String;
            fn call(&self, input: String) -> String;
        }
    }
}

fn main() {}
