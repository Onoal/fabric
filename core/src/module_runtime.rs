use crate::composition::ModuleBindings;
use crate::contract::{
    ContractRequirementDeclaration, ModuleContract, ProvidedContractDeclaration,
};
use crate::error::ModuleError;
use crate::health::Health;
use crate::identifiers::ModuleId;
use crate::instance::InstanceRuntimeContext;

pub trait ModuleRuntime: Send {
    fn id(&self) -> &ModuleId;

    fn provided_contract_declarations(&self) -> Vec<ProvidedContractDeclaration> {
        Vec::new()
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        Vec::new()
    }

    fn optional_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        Vec::new()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError>;

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError>;

    fn bind_instance_context(
        &mut self,
        _context: &InstanceRuntimeContext,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError>;

    fn start(&mut self) -> Result<(), ModuleError>;

    fn stop(&mut self);

    fn health(&self) -> Health;
}
