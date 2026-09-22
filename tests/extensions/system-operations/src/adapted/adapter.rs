use fabric::authoring::AdapterDefinition;
use fabric_core::{Health, ModuleContract, ModuleError, ModuleId, ModuleRuntime};
use fabric_system::{AdapterSystemSchemaSupport, SystemSchemaRequirement};

use crate::{AdaptedOperations, adapted_operations_system_id};

fabric::adapter! {
    pub FixedOperationsAdapter for crate::adapted::definition::AdaptedOperations {

        config {
            value: u64;
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                crate::OperationMarker::new(self.config.value)
            }
        }
    }
}

fabric::adapter! {
    pub AlternateOperationsAdapter for crate::adapted::definition::AdaptedOperations {

        config {
            value: u64;
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                crate::OperationMarker::new(self.config.value)
            }
        }
    }
}

fabric::adapter! {
    pub IncompatibleSchemaOperationsAdapter for crate::adapted::definition::AdaptedOperations {
        supports: "^3";

        config {
            value: u64;
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                crate::OperationMarker::new(self.config.value)
            }
        }
    }
}

fabric::adapter! {
    pub WrongVersionOperationsAdapter for crate::adapted::definition::AdaptedOperations {

        config {
            value: u64;
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                crate::OperationMarker::new(self.config.value)
            }
        }
    }
}

#[derive(Clone)]
pub struct MissingContractOperationsAdapter;

impl MissingContractOperationsAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MissingContractOperationsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterDefinition for MissingContractOperationsAdapter {
    type Target = AdaptedOperations;
    type Compatibility = AdapterSystemSchemaSupport;

    fn compatibility(&self) -> AdapterSystemSchemaSupport {
        AdapterSystemSchemaSupport::versioned(
            adapted_operations_system_id(),
            SystemSchemaRequirement::parse("^2").expect("schema requirement"),
        )
    }

    fn declaration(&self, provider_module_id: ModuleId) -> fabric_core::ModuleDeclaration {
        fabric_core::ModuleDeclaration::new(provider_module_id)
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(EmptyOperationsProvider {
            module_id: provider_module_id,
        }))
    }
}

#[derive(Clone)]
struct EmptyOperationsProvider {
    module_id: ModuleId,
}

impl ModuleRuntime for EmptyOperationsProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, _bindings: &fabric_core::ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}
