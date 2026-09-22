use fabric::component;

component! {
    MixedRuntime {
        id: "fabric.test.ui.mixed-runtime";
        runtime { prepare { Ok(()) } }
        operations {
            ping {
                id: "fabric.test.ui.mixed-runtime.ping";
                input: () = "fabric.test.ui.mixed-runtime.ping.input";
                output: () = "fabric.test.ui.mixed-runtime.ping.output";
                handler |_: ()| async move { Ok(()) };
            }
        }
    }
}

fn main() {}
