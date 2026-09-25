use std::sync::Arc;

use fabric_core::{ContractId, ContractKey, Health};

use crate::{ComponentError, ComponentId, ComponentInstanceBinding, ComponentParticipation};

const COMPONENT_REGISTRY_CONTRACT_ID: &str = "fabric.component.registry";

pub fn component_registry_contract_id() -> ContractId {
    ContractId::new(COMPONENT_REGISTRY_CONTRACT_ID).expect("static component registry contract id")
}

pub fn component_registry_contract_key() -> ContractKey<ComponentRegistry> {
    ContractKey::provisional(component_registry_contract_id())
}

pub trait ComponentRegistryService: Send + Sync {
    fn register(
        &self,
        component: ComponentInstanceBinding,
        health: Health,
    ) -> Result<ComponentStatus, ComponentError>;

    fn update_health(
        &self,
        participation: &ComponentParticipation,
        health: Health,
    ) -> Result<ComponentStatus, ComponentError>;

    fn activate(
        &self,
        participation: &ComponentParticipation,
    ) -> Result<ComponentStatus, ComponentError>;

    fn unregister(
        &self,
        participation: &ComponentParticipation,
    ) -> Result<ComponentStatus, ComponentError>;

    fn component(&self, component_id: &ComponentId) -> Result<ComponentStatus, ComponentError>;

    fn components(&self) -> Vec<ComponentStatus>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticipationState {
    Preparing,
    Active,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentStatus {
    participation: ComponentParticipation,
    state: ParticipationState,
    health: Health,
}

#[derive(Clone)]
pub struct ComponentRegistry {
    inner: Arc<dyn ComponentRegistryService>,
}

impl ComponentStatus {
    pub fn new(
        participation: ComponentParticipation,
        state: ParticipationState,
        health: Health,
    ) -> Self {
        Self {
            participation,
            state,
            health,
        }
    }

    pub fn component(&self) -> &ComponentInstanceBinding {
        self.participation.component()
    }

    pub fn participation(&self) -> &ComponentParticipation {
        &self.participation
    }

    pub fn state(&self) -> ParticipationState {
        self.state
    }

    pub fn is_active(&self) -> bool {
        self.state == ParticipationState::Active
    }

    pub fn health(&self) -> Health {
        self.health
    }

    pub fn is_available(&self) -> bool {
        self.health != Health::Unavailable
    }

    pub fn with_health(&self, health: Health) -> Self {
        Self {
            participation: self.participation.clone(),
            state: self.state,
            health,
        }
    }

    pub fn with_state(&self, state: ParticipationState) -> Self {
        Self {
            participation: self.participation.clone(),
            state,
            health: self.health,
        }
    }
}

impl ComponentRegistry {
    pub fn new(inner: Arc<dyn ComponentRegistryService>) -> Self {
        Self { inner }
    }

    pub fn register(
        &self,
        component: ComponentInstanceBinding,
        health: Health,
    ) -> Result<ComponentStatus, ComponentError> {
        self.inner.register(component, health)
    }

    pub fn update_health(
        &self,
        participation: &ComponentParticipation,
        health: Health,
    ) -> Result<ComponentStatus, ComponentError> {
        self.inner.update_health(participation, health)
    }

    pub fn activate(
        &self,
        participation: &ComponentParticipation,
    ) -> Result<ComponentStatus, ComponentError> {
        self.inner.activate(participation)
    }

    pub fn unregister(
        &self,
        participation: &ComponentParticipation,
    ) -> Result<ComponentStatus, ComponentError> {
        self.inner.unregister(participation)
    }

    pub fn component(&self, component_id: &ComponentId) -> Result<ComponentStatus, ComponentError> {
        self.inner.component(component_id)
    }

    pub fn components(&self) -> Vec<ComponentStatus> {
        self.inner.components()
    }
}
