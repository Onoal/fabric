#[cfg(test)]
use std::sync::{Arc, Mutex};

use fabric_core::{Health, ModuleContract, ModuleError, ModuleId, ModuleRuntime};
use fabric_sdk::prelude::AdapterDefinition;
use fabric_system::{AdapterSystemSchemaSupport, SystemSchemaRequirement};

use crate::{AdaptedOperations, adapted_operations_system_id};

#[cfg(test)]
use fabric_core::{ContractKey, ContractVersion};

#[cfg(test)]
use crate::OperationMarker;

fabric_sdk::adapter! {
    pub FixedOperationsAdapter for system crate::adapted::definition::AdaptedOperations implements crate::AdaptedOperationsRealization {
        schema: "^2";
        realization: "1.0.0";

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

fabric_sdk::adapter! {
    pub AlternateOperationsAdapter for system crate::adapted::definition::AdaptedOperations implements crate::AdaptedOperationsRealization {
        schema: "^2";
        realization: "1.0.0";

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

fabric_sdk::adapter! {
    pub IncompatibleSchemaOperationsAdapter for system crate::adapted::definition::AdaptedOperations implements crate::AdaptedOperationsRealization {
        schema: "^3";
        realization: "1.0.0";

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

fabric_sdk::adapter! {
    pub WrongVersionOperationsAdapter for system crate::adapted::definition::AdaptedOperations implements crate::AdaptedOperationsRealization {
        schema: "^2";
        realization: "2.0.0";

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
    type SchemaSupport = AdapterSystemSchemaSupport;

    fn schema_support(&self) -> AdapterSystemSchemaSupport {
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

#[cfg(test)]
#[derive(Clone)]
pub(crate) struct LifecycleCaptureOperationsAdapter {
    value: u64,
    lifecycle_capture: Arc<Mutex<Vec<String>>>,
}

#[cfg(test)]
impl LifecycleCaptureOperationsAdapter {
    pub(crate) fn new(value: u64, lifecycle_capture: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            value,
            lifecycle_capture,
        }
    }
}

#[cfg(test)]
impl AdapterDefinition for LifecycleCaptureOperationsAdapter {
    type Target = AdaptedOperations;
    type SchemaSupport = AdapterSystemSchemaSupport;

    fn schema_support(&self) -> AdapterSystemSchemaSupport {
        AdapterSystemSchemaSupport::versioned(
            adapted_operations_system_id(),
            SystemSchemaRequirement::parse("^2").expect("schema requirement"),
        )
    }

    fn declaration(&self, provider_module_id: ModuleId) -> fabric_core::ModuleDeclaration {
        let key = crate::adapted::definition::adapted_operations::realization::raw::contract_key(
            ContractVersion::parse("1.0.0").expect("version"),
        );
        fabric_core::ModuleDeclaration::new(provider_module_id)
            .with_provided_contracts(vec![key.declaration()])
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(LifecycleCaptureOperationsProvider::new(
            provider_module_id,
            crate::adapted::definition::adapted_operations::realization::raw::contract_key(
                ContractVersion::parse("1.0.0").expect("version"),
            ),
            self.value,
            Arc::clone(&self.lifecycle_capture),
        )))
    }
}

#[cfg(test)]
#[derive(Clone)]
struct FixedMarkerRealization(u64);

#[cfg(test)]
impl crate::adapted::definition::adapted_operations::realization::raw::Service
    for FixedMarkerRealization
{
    fn current_marker(&self) -> OperationMarker {
        OperationMarker::new(self.0)
    }
}

#[cfg(test)]
#[derive(Clone)]
struct LifecycleCaptureOperationsProvider {
    module_id: ModuleId,
    key: ContractKey<crate::adapted::definition::adapted_operations::realization::raw::Contract>,
    value: u64,
    lifecycle_capture: Arc<Mutex<Vec<String>>>,
}

#[cfg(test)]
impl LifecycleCaptureOperationsProvider {
    fn new(
        module_id: ModuleId,
        key: ContractKey<
            crate::adapted::definition::adapted_operations::realization::raw::Contract,
        >,
        value: u64,
        lifecycle_capture: Arc<Mutex<Vec<String>>>,
    ) -> Self {
        Self {
            module_id,
            key,
            value,
            lifecycle_capture,
        }
    }

    fn push_lifecycle(&self, event: &str) {
        self.lifecycle_capture
            .lock()
            .expect("fixed operations lifecycle capture")
            .push(format!("{event}:{}", self.module_id.as_str()));
    }
}

#[cfg(test)]
impl ModuleRuntime for LifecycleCaptureOperationsProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![self.key.declaration()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &self.key,
            Arc::new(
                crate::adapted::definition::adapted_operations::realization::raw::Contract::new(
                    Arc::new(FixedMarkerRealization(self.value)),
                ),
            ),
        )])
    }

    fn bind(&mut self, _bindings: &fabric_core::ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.push_lifecycle("initialize");
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.push_lifecycle("start");
        Ok(())
    }

    fn stop(&mut self) {
        self.push_lifecycle("stop");
    }

    fn health(&self) -> Health {
        Health::Healthy
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

    fn stop(&mut self) {}

    fn health(&self) -> Health {
        Health::Healthy
    }
}
