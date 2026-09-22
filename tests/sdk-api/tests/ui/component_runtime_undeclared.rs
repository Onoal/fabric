use fabric::component;

component! {
    UndeclaredRuntimeMethod {
        id: "fabric.test.ui.undeclared-runtime-method";
        api { fn ping(&self); }
        runtime { fn pong(&self) {} }
    }
}

fn main() {}
