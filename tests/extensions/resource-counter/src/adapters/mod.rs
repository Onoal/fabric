use std::sync::Arc;

use fabric::prelude::{AdapterDefinition, ResourceDefinition};
use fabric_core::{
    Health, ModuleContract, ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_resource::{AdapterResourceSchemaSupport, ResourceId};

use crate::{CounterValue, definition::adapted_counter};

fabric::adapter! {
    pub FixedCounterAdapter for resource crate::AdaptedCounter implements crate::AdaptedCounterRealization {
        schema: provisional;
        realization: "1.0.0";

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
    pub IncompatibleSchemaCounterAdapter for resource crate::AdaptedCounter implements crate::AdaptedCounterRealization {
        schema: "^9";
        realization: "1.0.0";

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
    pub WrongVersionCounterAdapter for resource crate::AdaptedCounter implements crate::AdaptedCounterRealization {
        schema: provisional;
        realization: "2.0.0";

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

#[derive(Clone)]
pub struct ThirdPartyStyleCounterAdapter {
    value: u64,
}

impl ThirdPartyStyleCounterAdapter {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

impl adapted_counter::realization::raw::Service for ThirdPartyStyleCounterAdapter {
    fn current_value(&self) -> CounterValue {
        CounterValue::new(self.value)
    }
}

impl ThirdPartyStyleCounterAdapter {
    pub fn export_contract(&self) -> Arc<adapted_counter::realization::raw::Contract> {
        Arc::new(adapted_counter::realization::raw::Contract::new(Arc::new(
            self.clone(),
        )))
    }

    pub fn foreign_resource_id() -> ResourceId {
        ResourceId::new("fabric.test.counter.foreign").expect("static foreign resource id")
    }
}
