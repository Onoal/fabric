use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use fabric_core::{
    Health, Module, ModuleBindings, ModuleContract, ModuleDeclaration, ModuleError, ModuleId,
    ModuleRuntime,
};
use fabric_resource::ResourceId;

use crate::contract::{ResourceRegistry, ResourceRegistryService, resource_registry_contract_key};
use crate::model::resource_registry_module_id;
use crate::{
    ResourceConfiguration, ResourceDescriptor, ResourceInspection, ResourceRegistryError,
    resource_registry_contract_id,
};

pub struct ResourceRegistryModule {
    module_id: ModuleId,
    shared: Arc<SharedResourceRegistryState>,
}

struct ResourceRegistryState {
    started: bool,
    health: Health,
    by_resource: BTreeMap<ResourceId, ResourceDescriptor>,
    by_module: BTreeMap<ModuleId, ResourceId>,
}

struct SharedResourceRegistryState {
    inner: Mutex<ResourceRegistryState>,
}

impl ResourceRegistryModule {
    pub fn new() -> Self {
        Self {
            module_id: resource_registry_module_id().expect("static resource registry module id"),
            shared: Arc::new(SharedResourceRegistryState {
                inner: Mutex::new(ResourceRegistryState {
                    started: false,
                    health: Health::Unavailable,
                    by_resource: BTreeMap::new(),
                    by_module: BTreeMap::new(),
                }),
            }),
        }
    }
}

impl Default for ResourceRegistryModule {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleRuntime for ResourceRegistryModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![resource_registry_contract_key().declaration()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        let service: Arc<dyn ResourceRegistryService> =
            Arc::clone(&self.shared) as Arc<dyn ResourceRegistryService>;
        Ok(vec![ModuleContract::new(
            &resource_registry_contract_key(),
            Arc::new(ResourceRegistry::new(service)),
        )])
    }

    fn bind(&mut self, _bindings: &ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        let mut state = self
            .shared
            .inner
            .lock()
            .expect("resource registry state lock");
        state.started = false;
        state.health = Health::Unavailable;
        state.by_resource.clear();
        state.by_module.clear();
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        let mut state = self
            .shared
            .inner
            .lock()
            .expect("resource registry state lock");
        state.started = true;
        state.health = Health::Healthy;
        Ok(())
    }

    fn stop(&mut self) {
        let mut state = self
            .shared
            .inner
            .lock()
            .expect("resource registry state lock");
        state.by_resource.clear();
        state.by_module.clear();
        state.started = false;
        state.health = Health::Unavailable;
    }

    fn health(&self) -> Health {
        self.shared
            .inner
            .lock()
            .expect("resource registry state lock")
            .health
    }
}

impl Module for ResourceRegistryModule {
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::new(self.module_id.clone())
            .with_provided_contracts(self.provided_contract_declarations())
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self::new()))
    }
}

impl ResourceRegistryService for SharedResourceRegistryState {
    fn register(
        &self,
        module: &dyn ModuleRuntime,
        resource: ResourceDescriptor,
    ) -> Result<ResourceDescriptor, ResourceRegistryError> {
        let resource = resource.bind_to_module(module, &[resource_registry_contract_id()]);
        let mut state = self.inner.lock().expect("resource registry state lock");
        ensure_started(&state)?;
        resource.validate()?;
        if let Some(existing_resource_id) = state.by_module.get(resource.module_id()) {
            let existing = state
                .by_resource
                .get(existing_resource_id)
                .expect("module resource registration must exist");
            if existing.resource_id() == resource.resource_id()
                && existing.module_id() == resource.module_id()
            {
                return Ok(existing.clone());
            }
            return Err(ResourceRegistryError::DuplicateResourceParticipant {
                module_id: resource.module_id().clone(),
                existing_resource_id: existing_resource_id.clone(),
            });
        }
        if let Some(existing) = state.by_resource.get(resource.resource_id()) {
            return Err(ResourceRegistryError::DuplicateResourceParticipation {
                resource_id: resource.resource_id().clone(),
                existing_module_id: existing.module_id().clone(),
            });
        }
        state
            .by_module
            .insert(resource.module_id().clone(), resource.resource_id().clone());
        state
            .by_resource
            .insert(resource.resource_id().clone(), resource.clone());
        Ok(resource)
    }

    fn unregister(
        &self,
        module_id: &ModuleId,
    ) -> Result<ResourceDescriptor, ResourceRegistryError> {
        let mut state = self.inner.lock().expect("resource registry state lock");
        ensure_started(&state)?;
        let resource_id = state.by_module.remove(module_id).ok_or_else(|| {
            ResourceRegistryError::UnknownResourceParticipant {
                module_id: module_id.clone(),
            }
        })?;
        state.by_resource.remove(&resource_id).ok_or_else(|| {
            ResourceRegistryError::UnknownResource {
                resource_id: resource_id.clone(),
            }
        })
    }

    fn resource(
        &self,
        resource_id: &ResourceId,
    ) -> Result<ResourceDescriptor, ResourceRegistryError> {
        let state = self.inner.lock().expect("resource registry state lock");
        ensure_started(&state)?;
        state.by_resource.get(resource_id).cloned().ok_or_else(|| {
            ResourceRegistryError::UnknownResource {
                resource_id: resource_id.clone(),
            }
        })
    }

    fn resources(&self) -> Vec<ResourceDescriptor> {
        let state = self.inner.lock().expect("resource registry state lock");
        if !state.started {
            return Vec::new();
        }
        state.by_resource.values().cloned().collect()
    }

    fn consume_configuration(
        &self,
        resource_id: &ResourceId,
        configuration: &ResourceConfiguration,
    ) -> Result<(), ResourceRegistryError> {
        let state = self.inner.lock().expect("resource registry state lock");
        ensure_started(&state)?;
        let resource = state.by_resource.get(resource_id).ok_or_else(|| {
            ResourceRegistryError::UnknownResource {
                resource_id: resource_id.clone(),
            }
        })?;
        let boundary = resource.configuration().ok_or_else(|| {
            ResourceRegistryError::ConfigurationUnsupported {
                resource_id: resource_id.clone(),
            }
        })?;
        let boundary = boundary.clone();
        drop(state);
        boundary.consume(configuration)
    }

    fn inspect(
        &self,
        resource_id: &ResourceId,
    ) -> Result<ResourceInspection, ResourceRegistryError> {
        let state = self.inner.lock().expect("resource registry state lock");
        ensure_started(&state)?;
        let resource = state.by_resource.get(resource_id).ok_or_else(|| {
            ResourceRegistryError::UnknownResource {
                resource_id: resource_id.clone(),
            }
        })?;
        let inspection = resource
            .inspection()
            .ok_or_else(|| ResourceRegistryError::InspectionUnsupported {
                resource_id: resource_id.clone(),
            })?
            .clone();
        drop(state);
        inspection.inspect()
    }
}

fn ensure_started(state: &ResourceRegistryState) -> Result<(), ResourceRegistryError> {
    if state.started {
        Ok(())
    } else {
        Err(ResourceRegistryError::Unavailable)
    }
}
