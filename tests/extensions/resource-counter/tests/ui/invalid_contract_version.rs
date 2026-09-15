use fabric::resource;

resource! {
    pub InvalidContractVersion {
        id: "fabric.test.counter.invalid-contract-version";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.invalid-contract-version";
                version: "not-a-version";

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
