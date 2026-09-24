use fabric::authoring::{AdapterDefinition, ResourceDefinition};
use fabric_core::{
    Health, ModuleContract, ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_resource::AdapterResourceSchemaSupport;

use crate::CounterValue;

fabric::adapter! {
    pub FixedCounterAdapter for crate::AdaptedCounter {
        id: "test.fixed-counter-adapter";

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
    pub IncompatibleSchemaCounterAdapter for crate::AdaptedCounter {
        id: "test.incompatible-schema-counter-adapter";
        supports: "^9";

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
    pub WrongVersionCounterAdapter for crate::AdaptedCounter {
        id: "test.wrong-version-counter-adapter";

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

#[derive(Clone)]
pub struct MissingContractCounterAdapter;

impl MissingContractCounterAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MissingContractCounterAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterDefinition for MissingContractCounterAdapter {
    fn adapter_definition_id(&self) -> fabric::authoring::AdapterDefinitionId {
        fabric::authoring::AdapterDefinitionId::new("test.manual.missing-contract-counter-adapter")
            .expect("static test Adapter definition ID is valid")
    }

    type Target = crate::AdaptedCounter;
    type Compatibility = AdapterResourceSchemaSupport;

    fn compatibility(&self) -> AdapterResourceSchemaSupport {
        AdapterResourceSchemaSupport::provisional(crate::AdaptedCounter::resource_id())
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(provider_module_id)
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(EmptyCounterProvider::new(provider_module_id)))
    }
}

#[derive(Clone)]
struct EmptyCounterProvider {
    module_id: ModuleId,
}

impl EmptyCounterProvider {
    fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }
}

impl ModuleRuntime for EmptyCounterProvider {
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
