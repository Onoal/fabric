use fabric::resource;

resource! {
    pub InvalidDependencyRequirement {
        id: "fabric.test.counter.invalid-dependency-requirement";
        schema: provisional;

        config {}

        requires {
            source: DirectCounter(version = "definitely-not-a-range");
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.invalid-dependency-requirement";
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
