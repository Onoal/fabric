use std::sync::Arc;

use fabric::authoring::{
    AdapterDefinition, ContractDependency, ResourceDefinition, SystemRequires,
};
use fabric_core::{
    ContractVersionRequirement, Health, ModuleBindings, ModuleContract, ModuleDeclaration,
    ModuleError, ModuleId, ModuleRuntime,
};
use fabric_resource::{AdapterResourceSchemaSupport, ResourceSchemaRequirement};
use fabric_test_resource_clock::{
    Clock, ClockError, ClockRealization, ClockRealizationContract, ClockTick,
    clock_realization_contract_key,
};

use crate::{OperationsContract, TestOperations};

#[derive(Clone)]
pub struct OperationsDrivenClockAdapter {
    offset: u64,
}

impl OperationsDrivenClockAdapter {
    pub fn new(offset: u64) -> Self {
        Self { offset }
    }
}

impl AdapterDefinition for OperationsDrivenClockAdapter {
    fn adapter_definition_id(&self) -> fabric::authoring::AdapterDefinitionId {
        fabric::authoring::AdapterDefinitionId::new("test.manual.operations-driven-clock-adapter")
            .expect("static test Adapter definition ID is valid")
    }

    type Target = Clock;
    type Compatibility = AdapterResourceSchemaSupport;

    fn compatibility(&self) -> AdapterResourceSchemaSupport {
        AdapterResourceSchemaSupport::versioned(
            Clock::resource_id(),
            ResourceSchemaRequirement::parse("^1").expect("static requirement"),
        )
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(provider_module_id)
            .with_provided_contracts(vec![clock_realization_contract_key().declaration()])
            .with_required_contracts(vec![
                SystemRequires::<TestOperations>::versioned(
                    ContractVersionRequirement::parse("^1.2").expect("static requirement"),
                )
                .declaration()
                .clone(),
            ])
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(OperationsDrivenClockProvider::new(
            provider_module_id,
            self.offset,
            SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("static requirement"),
            ),
        )))
    }
}

#[derive(Clone)]
struct OperationsDrivenClockRealization {
    operations: ContractDependency<OperationsContract>,
    offset: u64,
}

impl ClockRealization for OperationsDrivenClockRealization {
    fn current_tick(&self) -> Result<ClockTick, ClockError> {
        Ok(ClockTick::new(
            self.operations.current_marker().value() + self.offset,
        ))
    }
}

struct OperationsDrivenClockProvider {
    module_id: ModuleId,
    requirement: SystemRequires<TestOperations>,
    operations: ContractDependency<OperationsContract>,
    offset: u64,
}

impl OperationsDrivenClockProvider {
    fn new(module_id: ModuleId, offset: u64, requirement: SystemRequires<TestOperations>) -> Self {
        Self {
            module_id,
            operations: ContractDependency::new(requirement.as_contract_requirement().clone()),
            requirement,
            offset,
        }
    }
}

impl ModuleRuntime for OperationsDrivenClockProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![clock_realization_contract_key().declaration()]
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        let service = Arc::new(OperationsDrivenClockRealization {
            operations: self.operations.clone(),
            offset: self.offset,
        }) as Arc<dyn ClockRealization>;
        Ok(vec![ModuleContract::new(
            &clock_realization_contract_key(),
            Arc::new(ClockRealizationContract::new(service)),
        )])
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.operations
            .bind(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))
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
