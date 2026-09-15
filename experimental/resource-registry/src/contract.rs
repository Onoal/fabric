use std::sync::Arc;

use fabric_core::{ContractId, ContractKey, ModuleId, ModuleRuntime};
use fabric_resource::ResourceId;

use crate::{ResourceConfiguration, ResourceDescriptor, ResourceInspection, ResourceRegistryError};

const RESOURCE_REGISTRY_CONTRACT_ID: &str = "fabric.resource.registry";

pub fn resource_registry_contract_id() -> ContractId {
    ContractId::new(RESOURCE_REGISTRY_CONTRACT_ID).expect("static resource registry contract id")
}

pub fn resource_registry_contract_key() -> ContractKey<ResourceRegistry> {
    ContractKey::provisional(resource_registry_contract_id())
}

pub trait ResourceRegistryService: Send + Sync {
    fn register(
        &self,
        module: &dyn ModuleRuntime,
        capability: ResourceDescriptor,
    ) -> Result<ResourceDescriptor, ResourceRegistryError>;

    fn unregister(&self, module_id: &ModuleId)
    -> Result<ResourceDescriptor, ResourceRegistryError>;

    fn resource(
        &self,
        resource_id: &ResourceId,
    ) -> Result<ResourceDescriptor, ResourceRegistryError>;

    fn resources(&self) -> Vec<ResourceDescriptor>;

    fn consume_configuration(
        &self,
        resource_id: &ResourceId,
        configuration: &ResourceConfiguration,
    ) -> Result<(), ResourceRegistryError>;

    fn inspect(
        &self,
        resource_id: &ResourceId,
    ) -> Result<ResourceInspection, ResourceRegistryError>;
}

#[derive(Clone)]
pub struct ResourceRegistry {
    inner: Arc<dyn ResourceRegistryService>,
}

impl ResourceRegistry {
    pub fn new(inner: Arc<dyn ResourceRegistryService>) -> Self {
        Self { inner }
    }

    pub fn register(
        &self,
        module: &dyn ModuleRuntime,
        capability: ResourceDescriptor,
    ) -> Result<ResourceDescriptor, ResourceRegistryError> {
        self.inner.register(module, capability)
    }

    pub fn unregister(
        &self,
        module_id: &ModuleId,
    ) -> Result<ResourceDescriptor, ResourceRegistryError> {
        self.inner.unregister(module_id)
    }

    pub fn resource(
        &self,
        resource_id: &ResourceId,
    ) -> Result<ResourceDescriptor, ResourceRegistryError> {
        self.inner.resource(resource_id)
    }

    pub fn resources(&self) -> Vec<ResourceDescriptor> {
        self.inner.resources()
    }

    pub fn consume_configuration(
        &self,
        resource_id: &ResourceId,
        configuration: &ResourceConfiguration,
    ) -> Result<(), ResourceRegistryError> {
        self.inner.consume_configuration(resource_id, configuration)
    }

    pub fn inspect(
        &self,
        resource_id: &ResourceId,
    ) -> Result<ResourceInspection, ResourceRegistryError> {
        self.inner.inspect(resource_id)
    }
}
