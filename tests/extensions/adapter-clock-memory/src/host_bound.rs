use fabric::authoring::{AdapterDefinition, ResourceDefinition};
use fabric::prelude::{HostFacilityId, HostRequirement};
use fabric_core::{ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric_resource::{AdapterResourceSchemaSupport, ResourceSchemaRequirement};
use fabric_test_resource_clock::{Clock, clock_realization_contract_key};

use crate::memory::{MemoryClock, MemoryClockProvider};

#[derive(Clone, Default)]
pub struct HostBoundClockAdapter {
    realization: MemoryClock,
}

impl HostBoundClockAdapter {
    pub fn new(start: u64) -> Self {
        Self {
            realization: MemoryClock::new(start),
        }
    }
}

impl AdapterDefinition for HostBoundClockAdapter {
    fn adapter_definition_id(&self) -> fabric::authoring::AdapterDefinitionId {
        fabric::authoring::AdapterDefinitionId::new("test.manual.host-bound-clock-adapter")
            .expect("static test Adapter definition ID is valid")
    }

    type Target = Clock;
    type Compatibility = AdapterResourceSchemaSupport;

    fn compatibility(&self) -> Self::Compatibility {
        AdapterResourceSchemaSupport::versioned(
            Clock::resource_id(),
            ResourceSchemaRequirement::parse("^1").expect("schema requirement"),
        )
    }

    fn host_requirement(&self) -> HostRequirement {
        HostRequirement::new().require_facility(host_bound_clock_facility())
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(provider_module_id)
            .with_provided_contracts(vec![clock_realization_contract_key().declaration()])
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(MemoryClockProvider::new(
            provider_module_id,
            self.realization.clone(),
        )))
    }
}

pub fn host_bound_clock_facility() -> HostFacilityId {
    HostFacilityId::new("fabric.test.host.clock").expect("static host-bound clock facility")
}
