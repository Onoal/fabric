use fabric::resource;

resource! {
    pub MutSelfRealizationMethod {
        id: "fabric.test.counter.mut-self-realization-method";
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.mut-self-realization-method";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        adapter Adapter {
            id: "fabric.test.resource.counter.mut-self-realization-method.realization";
            compatibility: provisional;

            fn current_value(&mut self) -> u64;
        }

        runtime {
            fn current_value(&self) -> u64 {
                self.adapter.current_value()
            }
        }
    }
}

fn main() {}
