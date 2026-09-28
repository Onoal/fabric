#![allow(unused_imports)]

use crate::CounterValue;

fabric::resource! {
    pub DirectCounter {
        id: "fabric.test.counter";


        config {
            value: u64;
        }

        api {
            async fn current_value(&self) -> CounterValue;
        }

        runtime {
            async fn current_value(&self) -> CounterValue {
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
            async fn current_value(&self) -> CounterValue;
            async fn source_provider(&self) -> String;
            async fn source_identity(&self) -> String;
            async fn source_contract_pointer(&self) -> usize;
        }

        runtime {
            async fn current_value(&self) -> CounterValue {
                CounterValue::new(self.source.current_value().await.value() * 2)
            }

            async fn source_provider(&self) -> String {
                self.source.provider().as_str().to_owned()
            }

            async fn source_identity(&self) -> String {
                self.source.identity().to_string()
            }

            async fn source_contract_pointer(&self) -> usize {
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
            async fn current_value(&self) -> CounterValue;
        }
    }
}
