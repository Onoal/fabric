use fabric_sdk::resource;

resource! {
    pub MissingRealizationContractId {
        id: "fabric.test.counter.missing-realization-contract-id";
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.missing-realization-contract-id";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        adapter Adapter {
            compatibility: provisional;

            fn current_value(&self) -> u64;
        }

        runtime {
            fn current_value(&self) -> u64 {
                self.adapter.current_value()
            }
        }
    }
}

fn main() {}
