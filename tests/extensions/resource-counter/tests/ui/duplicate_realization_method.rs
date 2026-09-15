use fabric::resource;

resource! {
    pub DuplicateRealizationMethod {
        id: "fabric.test.counter.duplicate-realization-method";
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.duplicate-realization-method";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        adapter Adapter {
            id: "fabric.test.resource.counter.duplicate-realization-method.realization";
            compatibility: provisional;

            fn current_value(&self) -> u64;
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
