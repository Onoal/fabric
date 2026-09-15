use std::sync::{Arc, Mutex};

use fabric_core::{ContractKey, ContractRequirement, ModuleDeclaration};
use fabric_resource::{ResourceError, ResourceId, ResourceSchemaDescriptor, ResourceSchemaVersion};
use fabric_sdk::prelude::{
    AdaptableResourceDefinition, IntoResourceName, PrimaryResourceContract, ResourceDefinition,
    ResourceSelection,
};

use crate::{BoundClockRealization, ClockContract, ClockRealizationContract, NativeClock};

pub type ClockLifecycleCapture = Arc<Mutex<Vec<String>>>;
pub type ClockRealizationCapture = Arc<Mutex<Option<BoundClockRealization>>>;

#[derive(Clone, Default)]
pub struct ClockConfig {
    lifecycle_capture: Option<ClockLifecycleCapture>,
    realization_capture: Option<ClockRealizationCapture>,
}

impl ClockConfig {
    pub fn with_lifecycle_capture(mut self, lifecycle_capture: ClockLifecycleCapture) -> Self {
        self.lifecycle_capture = Some(lifecycle_capture);
        self
    }

    pub fn with_realization_capture(
        mut self,
        realization_capture: ClockRealizationCapture,
    ) -> Self {
        self.realization_capture = Some(realization_capture);
        self
    }

    pub(crate) fn lifecycle_capture(&self) -> Option<&ClockLifecycleCapture> {
        self.lifecycle_capture.as_ref()
    }

    pub(crate) fn realization_capture(&self) -> Option<&ClockRealizationCapture> {
        self.realization_capture.as_ref()
    }
}

pub struct Clock;

impl Clock {
    pub fn select(
        name: impl IntoResourceName,
        config: ClockConfig,
    ) -> Result<ResourceSelection<Self>, ResourceError> {
        <Self as ResourceDefinition>::select(name, config)
    }
}

impl PrimaryResourceContract for Clock {
    type Contract = ClockContract;

    fn primary_contract_key() -> ContractKey<Self::Contract> {
        crate::clock_contract_key()
    }
}

impl ResourceDefinition for Clock {
    type Config = ClockConfig;

    fn resource_id() -> ResourceId {
        crate::clock_resource_id()
    }

    fn schema() -> ResourceSchemaDescriptor {
        ResourceSchemaDescriptor::versioned(Self::resource_id(), clock_schema_version())
    }

    fn declaration(selection: &ResourceSelection<Self>) -> ModuleDeclaration {
        ModuleDeclaration::new(selection.module_id().clone())
            .with_provided_contracts(vec![crate::clock_contract_key().declaration()])
            .with_required_contracts(vec![Self::realization_requirement().declaration().clone()])
    }

    fn materialize(
        selection: &ResourceSelection<Self>,
    ) -> Option<Box<dyn fabric_core::ModuleRuntime>> {
        Some(Box::new(NativeClock::new(
            selection.module_id().as_str(),
            selection.config().clone(),
        )))
    }
}

impl AdaptableResourceDefinition for Clock {
    type RealizationContract = ClockRealizationContract;

    fn realization_requirement() -> ContractRequirement<Self::RealizationContract> {
        ContractRequirement::versioned(
            crate::clock_realization_contract_id(),
            crate::clock_realization_contract_version_requirement(),
        )
    }
}

pub fn clock_schema_version() -> ResourceSchemaVersion {
    ResourceSchemaVersion::parse("1.0.0").expect("static clock schema version")
}

pub fn clock_realization_contract_version_requirement() -> fabric_core::ContractVersionRequirement {
    fabric_core::ContractVersionRequirement::parse("^1")
        .expect("static clock realization contract requirement")
}
