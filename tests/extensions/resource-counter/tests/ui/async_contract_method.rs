use fabric_sdk::resource;

resource! {
    pub AsyncContractMethod {
        id: "fabric.test.counter.async-contract";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.async-contract";
                version: provisional;

                async fn current_value(&self) -> u64;
            }
        }

        runtime {
            async fn current_value(&self) -> u64 {
                self.config.value
            }
        }
    }
}

fn main() {}
