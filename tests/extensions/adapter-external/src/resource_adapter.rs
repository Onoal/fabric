use fabric::prelude::{HostFacilityId, HostRequirement};
use fabric_test_resource_counter::{AdaptedCounter as ExternalCounter, CounterValue};

pub fn external_counter_host_facility() -> HostFacilityId {
    HostFacilityId::new("fabric.test.host.external-counter")
        .expect("static external counter host facility")
}

fabric::adapter! {
    pub ExternalCounterAdapter for ExternalCounter {

        config {
            value: u64;
        }

        runtime {
            fn current_value(&self) -> CounterValue {
                CounterValue::new(self.config.value)
            }
        }
    }
}

fabric::adapter! {
    pub HostBoundExternalCounterAdapter for ExternalCounter {

        config {
            value: u64;
        }

        host: HostRequirement::new().require_facility(external_counter_host_facility());

        runtime {
            fn current_value(&self) -> CounterValue {
                CounterValue::new(self.config.value)
            }
        }
    }
}
