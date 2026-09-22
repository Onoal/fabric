use fabric::component;

component! {
    MismatchedRuntimeMethod {
        id: "fabric.test.ui.mismatched-runtime-method";
        api { fn ping(&self, value: usize) -> usize; }
        runtime { fn ping(&self, value: String) -> usize { value.len() } }
    }
}

fn main() {}
