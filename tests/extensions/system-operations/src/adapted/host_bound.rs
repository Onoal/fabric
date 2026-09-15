use fabric::prelude::{HostFacilityId, HostRequirement};

pub fn host_bound_operations_facility() -> HostFacilityId {
    HostFacilityId::new("fabric.test.host.operations")
        .expect("static host-bound operations facility")
}

fabric::adapter! {
    pub HostBoundOperationsAdapter for system crate::adapted::definition::AdaptedOperations implements crate::AdaptedOperationsRealization {
        schema: "^2";
        realization: "1.0.0";

        config {
            value: u64;
        }

        host: HostRequirement::new().require_facility(host_bound_operations_facility());

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                crate::OperationMarker::new(self.config.value)
            }
        }
    }
}
