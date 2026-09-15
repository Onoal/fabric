use fabric_sdk::resource;

resource! {
    pub DuplicateContractMethodCounter {
        id: "fabric.test.counter.duplicate-contract-method";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.duplicate-contract-method";
                version: provisional;

                fn current_value(&self) -> u64;
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
