#![allow(unused_imports)]

use crate::CounterValue;

fabric::resource! {
    pub DirectCounter {
        id: "fabric.test.counter";


        config {
            value: u64;
        }

        api {
            fn current_value(&self) -> CounterValue;
        }

        runtime {
            fn current_value(&self) -> CounterValue {
                CounterValue::new(self.config.value)
            }
        }
    }
}

fabric::resource! {
    pub DerivedCounter {
        id: "fabric.test.counter.derived";


        config {}

        relations {
            requires { source: DirectCounter(provisional); }
        }

        api {
            fn current_value(&self) -> CounterValue;
            fn source_provider(&self) -> String;
            fn source_identity(&self) -> String;
            fn source_contract_pointer(&self) -> usize;
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

fabric::resource! {
    pub AdaptedCounter {
        id: "fabric.test.counter.adapted";


        config {}

        api {
            fn current_value(&self) -> CounterValue;
        }
    }
}
