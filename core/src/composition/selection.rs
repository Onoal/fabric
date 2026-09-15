use crate::{ContractId, ModuleId};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContractProviderSelection {
    consumer: ModuleId,
    contract_id: ContractId,
    provider: ModuleId,
}

impl ContractProviderSelection {
    pub fn new(consumer: ModuleId, contract_id: ContractId, provider: ModuleId) -> Self {
        Self {
            consumer,
            contract_id,
            provider,
        }
    }

    pub fn consumer(&self) -> &ModuleId {
        &self.consumer
    }

    pub fn contract_id(&self) -> &ContractId {
        &self.contract_id
    }

    pub fn provider(&self) -> &ModuleId {
        &self.provider
    }
}
