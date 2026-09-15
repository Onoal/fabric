use fabric_sdk::resource;

resource! {
    pub RequiresMissingCompatibility {
        id: "fabric.test.counter.requires-missing-compatibility";
        schema: provisional;

        config {}

        requires {
            source: DirectCounter();
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.requires-missing-compatibility";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        runtime {
            fn current_value(&self) -> u64 {
                self.source.current_value()
            }
        }
    }
}

resource! {
    pub DirectCounter {
        id: "fabric.test.counter.direct";
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.direct";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        runtime {
            fn current_value(&self) -> u64 {
                1
            }
        }
    }
}

fn main() {}
