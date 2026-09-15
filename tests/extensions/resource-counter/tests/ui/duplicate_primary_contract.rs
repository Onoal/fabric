use fabric::resource;

resource! {
    pub DuplicatePrimaryContract {
        id: "fabric.test.counter.duplicate-primary";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.duplicate-primary.a";
                version: provisional;

                fn current_value(&self) -> u64;
            }

            primary Secondary {
                id: "fabric.test.resource.counter.duplicate-primary.b";
                version: provisional;

                fn other_value(&self) -> u64;
            }
        }

        runtime {
            fn current_value(&self) -> u64 {
                self.config.value
            }

            fn other_value(&self) -> u64 {
                self.config.value
            }
        }
    }
}

fn main() {}
