use fabric::{adapter, component};

component! {
    Target {
        id: "fabric.test.ui.component-adapter-missing-runtime-method";
        api { fn ping(&self) -> usize; }
    }
}

adapter! {
    Incomplete for Target {
        runtime { prepare { Ok(()) } }
    }
}

fn main() {}
