use fabric::resource;

resource! {
    pub MissingResourceId {
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.missing-id";
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
