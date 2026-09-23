use std::sync::Arc;

use fabric_core::{ContractId, ContractKey, InstanceId};

use crate::{ComponentError, ComponentHostLifecycle, ComponentHostStatus};

const COMPONENT_RUNTIME_CONTRACT_ID: &str = "fabric.component.runtime";

pub fn component_host_contract_id() -> ContractId {
    ContractId::new(COMPONENT_RUNTIME_CONTRACT_ID).expect("static component runtime contract id")
}

pub fn component_host_contract_key() -> ContractKey<ComponentHost> {
    ContractKey::provisional(component_host_contract_id())
}

pub trait ComponentHostService: Send + Sync {
    fn instance_id(&self) -> InstanceId;

    fn current_instance_id(&self) -> Result<InstanceId, ComponentError>;

    fn current_status(&self) -> ComponentHostStatus;

    fn current_lifecycle(&self) -> ComponentHostLifecycle;

    fn current_health(&self) -> fabric_core::Health;
}

#[derive(Clone)]
pub struct ComponentHost {
    inner: Arc<dyn ComponentHostService>,
}

impl ComponentHost {
    pub fn new(inner: Arc<dyn ComponentHostService>) -> Self {
        Self { inner }
    }

    pub fn instance_id(&self) -> InstanceId {
        self.inner.instance_id()
    }

    pub fn current_instance_id(&self) -> Result<InstanceId, ComponentError> {
        self.inner.current_instance_id()
    }

    pub fn current_status(&self) -> ComponentHostStatus {
        self.inner.current_status()
    }

    pub fn current_lifecycle(&self) -> ComponentHostLifecycle {
        self.inner.current_lifecycle()
    }

    pub fn current_health(&self) -> fabric_core::Health {
        self.inner.current_health()
    }
}
