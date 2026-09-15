use fabric::resource;

resource! {
    pub DuplicateRequiresField {
        id: "fabric.test.counter.duplicate-requires-field";
        schema: provisional;

        config {}

        requires {
            source: FirstCounter(provisional);
            source: SecondCounter(provisional);
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.duplicate-requires-field";
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
    pub FirstCounter {
        id: "fabric.test.counter.first";
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.first";
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

resource! {
    pub SecondCounter {
        id: "fabric.test.counter.second";
        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.second";
                version: provisional;

                fn current_value(&self) -> u64 {
                    2
                }
            }
        }

        runtime {
            fn current_value(&self) -> u64 {
                2
            }
        }
    }
}

fn main() {}
