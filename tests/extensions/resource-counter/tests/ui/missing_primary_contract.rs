use fabric::resource;

resource! {
    pub MissingPrimaryContract {
        id: "fabric.test.counter.missing-primary";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            Api {
                id: "fabric.test.resource.counter.missing-primary";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        runtime {
            fn current_value(&self) -> u64 {
                self.config.value
            }
        }
    }
}

fn main() {}
