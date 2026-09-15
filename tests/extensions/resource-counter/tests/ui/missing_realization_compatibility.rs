use fabric_sdk::resource;

resource! {
    pub MissingRealizationCompatibility {
        id: "fabric.test.counter.missing-realization-compatibility";
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.missing-realization-compatibility";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        adapter Adapter {
            id: "fabric.test.resource.counter.missing-realization-compatibility.realization";

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
