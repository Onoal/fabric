use fabric::resource;

resource! {
    pub InvalidResourceSchemaVersion {
        id: "fabric.test.counter.invalid-schema-version";
        schema: "not-a-version";

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.invalid-schema-version";
                version: provisional;

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
