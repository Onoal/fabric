use fabric_sdk::resource;

resource! {
    pub MissingRuntimeMethod {
        id: "fabric.test.counter.missing-runtime";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.missing-runtime";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        runtime {}
    }
}

fn main() {}
