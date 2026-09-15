use fabric::resource;

resource! {
    pub DuplicateAdapterSection {
        id: "fabric.test.counter.duplicate-adapter-section";
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.duplicate-adapter-section";
                version: provisional;

                fn current_value(&self) -> u64;
            }
        }

        adapter PrimaryAdapter {
            id: "fabric.test.resource.counter.duplicate-adapter-section.primary";
            compatibility: provisional;

            fn current_value(&self) -> u64;
        }

        adapter SecondaryAdapter {
            id: "fabric.test.resource.counter.duplicate-adapter-section.secondary";
            compatibility: provisional;

            fn current_value(&self) -> u64;
        }

        runtime {
            fn current_value(&self) -> u64 {
                self.primary_adapter.current_value()
            }
        }
    }
}

fn main() {}
