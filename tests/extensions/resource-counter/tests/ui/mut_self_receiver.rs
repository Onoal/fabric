use fabric_sdk::resource;

resource! {
    pub MutSelfCounter {
        id: "fabric.test.counter.mut-self";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.mut-self";
                version: provisional;

                fn current_value(&mut self) -> u64;
            }
        }

        runtime {
            fn current_value(&mut self) -> u64 {
                self.config.value
            }
        }
    }
}

fn main() {}
