#![allow(unused_imports)]

use crate::CounterValue;

fabric_sdk::resource! {
    pub DirectCounter {
        id: "fabric.test.counter";

        schema: provisional;

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter";
                version: provisional;

                fn current_value(&self) -> CounterValue;
            }
        }

        runtime {
            fn current_value(&self) -> CounterValue {
                CounterValue::new(self.config.value)
            }
        }
    }
}

fabric_sdk::resource! {
    pub DerivedCounter {
        id: "fabric.test.counter.derived";

        schema: provisional;

        config {}

        requires {
            source: DirectCounter(provisional);
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.derived";
                version: provisional;

                fn current_value(&self) -> CounterValue;
                fn source_provider(&self) -> String;
                fn source_identity(&self) -> String;
                fn source_contract_pointer(&self) -> usize;
            }
        }

        runtime {
            fn current_value(&self) -> CounterValue {
                CounterValue::new(self.source.current_value().value() * 2)
            }

            fn source_provider(&self) -> String {
                self.source.provider().as_str().to_owned()
            }

            fn source_identity(&self) -> String {
                self.source.identity().to_string()
            }

            fn source_contract_pointer(&self) -> usize {
                ::std::sync::Arc::as_ptr(&self.source.value()) as usize
            }
        }
    }
}

fabric_sdk::resource! {
    pub AdaptedCounter {
        id: "fabric.test.counter.adapted";

        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.adapted";
                version: provisional;

                fn current_value(&self) -> CounterValue;
                fn adapter_provider(&self) -> String;
                fn adapter_identity(&self) -> String;
            }
        }

        adapter Adapter {
            id: "fabric.test.resource.counter.adapted.realization";
            compatibility: "^1";

            fn current_value(&self) -> CounterValue;
        }

        runtime {
            fn current_value(&self) -> CounterValue {
                self.adapter.current_value()
            }

            fn adapter_provider(&self) -> String {
                self.adapter.provider().as_str().to_owned()
            }

            fn adapter_identity(&self) -> String {
                self.adapter.identity().to_string()
            }
        }
    }
}
