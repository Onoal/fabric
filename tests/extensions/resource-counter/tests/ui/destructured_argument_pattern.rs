use fabric_sdk::resource;

resource! {
    pub DestructuredCounter {
        id: "fabric.test.counter.destructured";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.destructured";
                version: provisional;

                fn current_value(&self, (value, extra): (u64, u64)) -> u64;
            }
        }

        runtime {
            fn current_value(&self, (value, extra): (u64, u64)) -> u64 {
                value + extra + self.config.value
            }
        }
    }
}

fn main() {}
