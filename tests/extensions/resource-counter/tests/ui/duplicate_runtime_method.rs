use fabric::resource;

resource! {
    pub DuplicateRuntimeMethodCounter {
        id: "fabric.test.counter.duplicate-runtime-method";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.duplicate-runtime-method";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        runtime {
            fn current_value(&self) -> u64 {
                self.config.value
            }

            fn current_value(&self) -> u64 {
                self.config.value + 1
            }
        }
    }
}

fn main() {}
