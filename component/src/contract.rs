use std::sync::Arc;

use fabric_core::{ContractId, ContractKey, InstanceId};

use crate::{ComponentError, ComponentRuntimeLifecycle, ComponentRuntimeStatus};

const COMPONENT_RUNTIME_CONTRACT_ID: &str = "fabric.component.runtime";

pub fn component_runtime_contract_id() -> ContractId {
    ContractId::new(COMPONENT_RUNTIME_CONTRACT_ID).expect("static component runtime contract id")
}

pub fn component_runtime_contract_key() -> ContractKey<ComponentRuntime> {
    ContractKey::provisional(component_runtime_contract_id())
}

pub trait ComponentRuntimeService: Send + Sync {
    fn instance_id(&self) -> InstanceId;

    fn current_instance_id(&self) -> Result<InstanceId, ComponentError>;

    fn current_status(&self) -> ComponentRuntimeStatus;

    fn current_lifecycle(&self) -> ComponentRuntimeLifecycle;

    fn current_health(&self) -> fabric_core::Health;
}

#[derive(Clone)]
pub struct ComponentRuntime {
    inner: Arc<dyn ComponentRuntimeService>,
}

impl ComponentRuntime {
    pub fn new(inner: Arc<dyn ComponentRuntimeService>) -> Self {
        Self { inner }
    }

    pub fn instance_id(&self) -> InstanceId {
        self.inner.instance_id()
    }

    pub fn current_instance_id(&self) -> Result<InstanceId, ComponentError> {
        self.inner.current_instance_id()
    }

    pub fn current_status(&self) -> ComponentRuntimeStatus {
        self.inner.current_status()
    }

    pub fn current_lifecycle(&self) -> ComponentRuntimeLifecycle {
        self.inner.current_lifecycle()
    }

    pub fn current_health(&self) -> fabric_core::Health {
        self.inner.current_health()
    }
}
