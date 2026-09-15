use fabric_sdk::resource;

resource! {
    pub GenericMethodCounter {
        id: "fabric.test.counter.generic-method";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.generic-method";
                version: provisional;

                fn current_value<T>(&self, value: T) -> u64;
            }
        }

        runtime {
            fn current_value<T>(&self, _value: T) -> u64 {
                self.config.value
            }
        }
    }
}

fn main() {}
