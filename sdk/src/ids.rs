use fabric_core::{BlockId, CompositionError, CompositionId, ContractId, InstanceId, ModuleId};

pub trait IntoCompositionId {
    fn into_composition_id(self) -> Result<CompositionId, CompositionError>;
}

pub trait IntoBlockId {
    fn into_block_id(self) -> Result<BlockId, CompositionError>;
}

pub trait IntoInstanceId {
    fn into_instance_id(self) -> Result<InstanceId, CompositionError>;
}

pub trait IntoModuleId {
    fn into_module_id(self) -> Result<ModuleId, CompositionError>;
}

pub trait IntoContractId {
    fn into_contract_id(self) -> Result<ContractId, CompositionError>;
}

pub fn composition(value: impl IntoCompositionId) -> Result<CompositionId, CompositionError> {
    value.into_composition_id()
}

pub fn block(value: impl IntoBlockId) -> Result<BlockId, CompositionError> {
    value.into_block_id()
}

pub fn instance(value: impl IntoInstanceId) -> Result<InstanceId, CompositionError> {
    value.into_instance_id()
}

pub fn module(value: impl IntoModuleId) -> Result<ModuleId, CompositionError> {
    value.into_module_id()
}

pub fn contract(value: impl IntoContractId) -> Result<ContractId, CompositionError> {
    value.into_contract_id()
}

impl IntoCompositionId for CompositionId {
    fn into_composition_id(self) -> Result<CompositionId, CompositionError> {
        Ok(self)
    }
}

impl IntoBlockId for BlockId {
    fn into_block_id(self) -> Result<BlockId, CompositionError> {
        Ok(self)
    }
}

impl IntoInstanceId for InstanceId {
    fn into_instance_id(self) -> Result<InstanceId, CompositionError> {
        Ok(self)
    }
}

impl IntoModuleId for ModuleId {
    fn into_module_id(self) -> Result<ModuleId, CompositionError> {
        Ok(self)
    }
}

impl IntoContractId for ContractId {
    fn into_contract_id(self) -> Result<ContractId, CompositionError> {
        Ok(self)
    }
}

impl IntoCompositionId for &CompositionId {
    fn into_composition_id(self) -> Result<CompositionId, CompositionError> {
        Ok(self.clone())
    }
}

impl IntoBlockId for &BlockId {
    fn into_block_id(self) -> Result<BlockId, CompositionError> {
        Ok(self.clone())
    }
}

impl IntoInstanceId for &InstanceId {
    fn into_instance_id(self) -> Result<InstanceId, CompositionError> {
        Ok(self.clone())
    }
}

impl IntoModuleId for &ModuleId {
    fn into_module_id(self) -> Result<ModuleId, CompositionError> {
        Ok(self.clone())
    }
}

impl IntoContractId for &ContractId {
    fn into_contract_id(self) -> Result<ContractId, CompositionError> {
        Ok(self.clone())
    }
}

impl IntoCompositionId for &str {
    fn into_composition_id(self) -> Result<CompositionId, CompositionError> {
        CompositionId::new(self.to_owned())
    }
}

impl IntoBlockId for &str {
    fn into_block_id(self) -> Result<BlockId, CompositionError> {
        BlockId::new(self.to_owned())
    }
}

impl IntoInstanceId for &str {
    fn into_instance_id(self) -> Result<InstanceId, CompositionError> {
        InstanceId::new(self.to_owned())
    }
}

impl IntoModuleId for &str {
    fn into_module_id(self) -> Result<ModuleId, CompositionError> {
        ModuleId::new(self.to_owned())
    }
}

impl IntoContractId for &str {
    fn into_contract_id(self) -> Result<ContractId, CompositionError> {
        ContractId::new(self.to_owned())
    }
}

impl IntoCompositionId for String {
    fn into_composition_id(self) -> Result<CompositionId, CompositionError> {
        CompositionId::new(self)
    }
}

impl IntoBlockId for String {
    fn into_block_id(self) -> Result<BlockId, CompositionError> {
        BlockId::new(self)
    }
}

impl IntoInstanceId for String {
    fn into_instance_id(self) -> Result<InstanceId, CompositionError> {
        InstanceId::new(self)
    }
}

impl IntoModuleId for String {
    fn into_module_id(self) -> Result<ModuleId, CompositionError> {
        ModuleId::new(self)
    }
}

impl IntoContractId for String {
    fn into_contract_id(self) -> Result<ContractId, CompositionError> {
        ContractId::new(self)
    }
}
