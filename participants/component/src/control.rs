use std::sync::Arc;

use fabric_core::{ContractId, ContractKey};

use crate::{ComponentControlSnapshot, ComponentError, ComponentId, ComponentInstanceBinding};

const COMPONENT_CONTROL_CONTRACT_ID: &str = "fabric.component.control";

pub fn component_control_contract_id() -> ContractId {
    ContractId::new(COMPONENT_CONTROL_CONTRACT_ID).expect("static component control contract id")
}

pub fn component_control_contract_key() -> ContractKey<ComponentControlRail> {
    ContractKey::provisional(component_control_contract_id())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentDesiredState {
    Enabled,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentControl {
    component: ComponentInstanceBinding,
    desired: ComponentDesiredState,
}

pub trait ComponentControlService: Send + Sync {
    fn set_desired(
        &self,
        component: ComponentInstanceBinding,
        desired: ComponentDesiredState,
    ) -> Result<ComponentControl, ComponentError>;

    fn control(&self, component_id: &ComponentId) -> Result<ComponentControl, ComponentError>;

    fn controls(&self) -> Vec<ComponentControl>;

    fn snapshot(&self) -> ComponentControlSnapshot;
}

#[derive(Clone)]
pub struct ComponentControlRail {
    inner: Arc<dyn ComponentControlService>,
}

impl ComponentControl {
    pub fn new(component: ComponentInstanceBinding, desired: ComponentDesiredState) -> Self {
        Self { component, desired }
    }

    pub fn component(&self) -> &ComponentInstanceBinding {
        &self.component
    }

    pub fn desired(&self) -> ComponentDesiredState {
        self.desired
    }

    pub fn with_desired(&self, desired: ComponentDesiredState) -> Self {
        Self {
            component: self.component.clone(),
            desired,
        }
    }
}

impl ComponentControlRail {
    pub fn new(inner: Arc<dyn ComponentControlService>) -> Self {
        Self { inner }
    }

    pub fn set_desired(
        &self,
        component: ComponentInstanceBinding,
        desired: ComponentDesiredState,
    ) -> Result<ComponentControl, ComponentError> {
        self.inner.set_desired(component, desired)
    }

    pub fn enable(
        &self,
        component: ComponentInstanceBinding,
    ) -> Result<ComponentControl, ComponentError> {
        self.set_desired(component, ComponentDesiredState::Enabled)
    }

    pub fn disable(
        &self,
        component: ComponentInstanceBinding,
    ) -> Result<ComponentControl, ComponentError> {
        self.set_desired(component, ComponentDesiredState::Disabled)
    }

    pub fn control(&self, component_id: &ComponentId) -> Result<ComponentControl, ComponentError> {
        self.inner.control(component_id)
    }

    pub fn controls(&self) -> Vec<ComponentControl> {
        self.inner.controls()
    }

    pub fn snapshot(&self) -> ComponentControlSnapshot {
        self.inner.snapshot()
    }
}
