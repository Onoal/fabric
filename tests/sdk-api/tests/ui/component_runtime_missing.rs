use fabric::component;

component! {
    MissingRuntimeMethod {
        id: "fabric.test.ui.missing-runtime-method";
        api { fn ping(&self) -> usize; }
        runtime { prepare { Ok(()) } }
    }
}

fn main() {}
