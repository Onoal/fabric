use std::sync::Arc;

use fabric_core::{ContractId, ContractKey};

const SYSTEM_BACKED_RESOURCE_CONTRACT_ID: &str = "fabric.test.resource.system-backed";

pub fn system_backed_resource_contract_id() -> ContractId {
    ContractId::new(SYSTEM_BACKED_RESOURCE_CONTRACT_ID)
        .expect("static system-backed resource contract id")
}

pub fn system_backed_resource_contract_key() -> ContractKey<SystemBackedResourceContract> {
    ContractKey::provisional(system_backed_resource_contract_id())
}

pub trait SystemBackedResourceService: Send + Sync {
    fn current_value(&self) -> u64;
    fn system_provider(&self) -> String;
    fn system_identity(&self) -> String;
}

#[derive(Clone)]
pub struct SystemBackedResourceContract {
    inner: Arc<dyn SystemBackedResourceService>,
}

impl SystemBackedResourceContract {
    pub fn new(inner: Arc<dyn SystemBackedResourceService>) -> Self {
        Self { inner }
    }

    pub fn current_value(&self) -> u64 {
        self.inner.current_value()
    }

    pub fn system_provider(&self) -> String {
        self.inner.system_provider()
    }

    pub fn system_identity(&self) -> String {
        self.inner.system_identity()
    }
}
