use fabric::component;

component! {
    RemovedTeardown {
        id: "fabric.test.removed.teardown";
        teardown { Ok(()) }
    }
}

fn main() {}
