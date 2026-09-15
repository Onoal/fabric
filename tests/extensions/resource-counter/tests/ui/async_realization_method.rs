use fabric::resource;

resource! {
    pub AsyncRealizationMethod {
        id: "fabric.test.counter.async-realization-method";
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.async-realization-method";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        adapter Adapter {
            id: "fabric.test.resource.counter.async-realization-method.realization";
            compatibility: provisional;

            async fn current_value(&self) -> u64;
        }

        runtime {
            fn current_value(&self) -> u64 {
                self.adapter.current_value()
            }
        }
    }
}

fn main() {}
