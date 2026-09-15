use fabric::resource;

resource! {
    pub GenericRealizationMethod {
        id: "fabric.test.counter.generic-realization-method";
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.generic-realization-method";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        adapter Adapter {
            id: "fabric.test.resource.counter.generic-realization-method.realization";
            compatibility: provisional;

            fn current_value<T>(&self, value: T) -> T;
        }

        runtime {
            fn current_value(&self) -> u64 {
                self.adapter.current_value(1)
            }
        }
    }
}

fn main() {}
