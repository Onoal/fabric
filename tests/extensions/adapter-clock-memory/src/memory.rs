use std::sync::{Arc, Mutex};

use fabric::prelude::{AdapterDefinition, ResourceDefinition};
use fabric_core::{
    Health, ModuleContract, ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_resource::{AdapterResourceSchemaSupport, ResourceSchemaRequirement};
use fabric_test_resource_clock::{
    Clock, ClockError, ClockRealization, ClockRealizationContract, ClockTick,
    clock_realization_contract_key,
};

pub struct MemoryClock {
    start: u64,
    next: Arc<Mutex<u64>>,
    lifecycle_capture: Option<Arc<Mutex<Vec<String>>>>,
}

impl MemoryClock {
    pub fn new(start: u64) -> Self {
        Self {
            start,
            next: Arc::new(Mutex::new(start)),
            lifecycle_capture: None,
        }
    }

    pub fn with_lifecycle_capture(mut self, lifecycle_capture: Arc<Mutex<Vec<String>>>) -> Self {
        self.lifecycle_capture = Some(lifecycle_capture);
        self
    }

    fn push_lifecycle(&self, event: &str, module_id: &ModuleId) {
        if let Some(capture) = &self.lifecycle_capture {
            capture
                .lock()
                .expect("memory clock lifecycle capture")
                .push(format!("{event}:{}", module_id.as_str()));
        }
    }
}

impl Clone for MemoryClock {
    fn clone(&self) -> Self {
        let mut cloned = Self::new(self.start);
        cloned.lifecycle_capture = self.lifecycle_capture.clone();
        cloned
    }
}

impl Default for MemoryClock {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ClockRealization for MemoryClock {
    fn current_tick(&self) -> Result<ClockTick, ClockError> {
        let mut next = self.next.lock().expect("clock state");
        let current = *next;
        *next += 1;
        Ok(ClockTick::new(current))
    }

    fn clear(&self) -> Result<(), ClockError> {
        *self.next.lock().expect("clock state") = self.start;
        Ok(())
    }
}

impl AdapterDefinition for MemoryClock {
    type Target = Clock;
    type Compatibility = AdapterResourceSchemaSupport;

    fn compatibility(&self) -> AdapterResourceSchemaSupport {
        AdapterResourceSchemaSupport::versioned(
            Clock::resource_id(),
            ResourceSchemaRequirement::parse("^1").expect("schema requirement"),
        )
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(provider_module_id)
            .with_provided_contracts(vec![clock_realization_contract_key().declaration()])
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(MemoryClockProvider::new(
            provider_module_id,
            self.clone(),
        )))
    }
}

#[derive(Clone)]
pub(crate) struct MemoryClockProvider {
    module_id: ModuleId,
    realization: MemoryClock,
}

impl MemoryClockProvider {
    pub(crate) fn new(module_id: ModuleId, realization: MemoryClock) -> Self {
        Self {
            module_id,
            realization,
        }
    }
}

impl ModuleRuntime for MemoryClockProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![clock_realization_contract_key().declaration()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &clock_realization_contract_key(),
            Arc::new(ClockRealizationContract::new(Arc::new(
                self.realization.clone(),
            ))),
        )])
    }

    fn bind(&mut self, _bindings: &fabric_core::ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.realization
            .push_lifecycle("initialize", &self.module_id);
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.realization.push_lifecycle("start", &self.module_id);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        self.realization.push_lifecycle("stop", &self.module_id);
        let _ = self.realization.clear();
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}
