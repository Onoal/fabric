use fabric_sdk::resource;

resource! {
    pub ValueSelfCounter {
        id: "fabric.test.counter.value-self";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.value-self";
                version: provisional;

                fn current_value(self) -> u64;
            }
        }

        runtime {
            fn current_value(self) -> u64 {
                self.config.value
            }
        }
    }
}

fn main() {}
