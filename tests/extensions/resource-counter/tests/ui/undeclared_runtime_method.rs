use fabric::resource;

resource! {
    pub UndeclaredRuntimeMethod {
        id: "fabric.test.counter.undeclared-runtime";
        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.undeclared-runtime";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        runtime {
            fn current_value(&self) -> u64 {
                self.config.value
            }

            fn extra(&self) -> u64 {
                self.config.value + 1
            }
        }
    }
}

fn main() {}
