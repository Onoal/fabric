use std::sync::Arc;

use fabric::prelude::{ContractDependency, SystemRequires};
use fabric_core::{Health, ModuleBindings, ModuleContract, ModuleError, ModuleId, ModuleRuntime};

use crate::{
    OperationsContract, SystemBackedResourceConfig, SystemBackedResourceContract,
    SystemBackedResourceService, TestOperations, system_backed_resource_contract_key,
};

#[derive(Clone)]
struct RealizedSystemBackedResource {
    operations: ContractDependency<OperationsContract>,
    multiplier: u64,
}

pub struct SystemBackedResourceRuntime {
    module_id: ModuleId,
    requirement: SystemRequires<TestOperations>,
    operations: ContractDependency<OperationsContract>,
    multiplier: u64,
}

impl SystemBackedResourceRuntime {
    pub fn new(
        module_id: ModuleId,
        config: SystemBackedResourceConfig,
        requirement: SystemRequires<TestOperations>,
    ) -> Self {
        Self {
            module_id,
            operations: ContractDependency::new(requirement.as_contract_requirement().clone()),
            requirement,
            multiplier: config.multiplier,
        }
    }
}

impl SystemBackedResourceService for RealizedSystemBackedResource {
    fn current_value(&self) -> u64 {
        self.operations.current_marker().value() * self.multiplier
    }

    fn system_provider(&self) -> String {
        self.operations.provider().as_str().to_owned()
    }

    fn system_identity(&self) -> String {
        self.operations.identity().to_string()
    }
}

impl ModuleRuntime for SystemBackedResourceRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![system_backed_resource_contract_key().declaration()]
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        let service = Arc::new(RealizedSystemBackedResource {
            operations: self.operations.clone(),
            multiplier: self.multiplier,
        }) as Arc<dyn SystemBackedResourceService>;
        Ok(vec![ModuleContract::new(
            &system_backed_resource_contract_key(),
            Arc::new(SystemBackedResourceContract::new(service)),
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

    fn stop(&mut self) {}

    fn health(&self) -> Health {
        Health::Healthy
    }
}
