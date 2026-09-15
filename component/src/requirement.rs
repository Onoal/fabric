use std::sync::Arc;

use fabric_core::{ContractId, ContractKey, Health, ModuleId};

use crate::communication::ProvidedComponentContract;
use crate::{Component, ComponentError, ComponentId, ComponentParticipation, ParticipationState};

const COMPONENT_REQUIREMENTS_CONTRACT_ID: &str = "fabric.component.requirements";

pub fn component_requirement_contract_id() -> ContractId {
    ContractId::new(COMPONENT_REQUIREMENTS_CONTRACT_ID)
        .expect("static component requirement contract id")
}

pub fn component_requirement_contract_key() -> ContractKey<ComponentRequirementRail> {
    ContractKey::provisional(component_requirement_contract_id())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentRequirementKind {
    Required,
    Optional,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedComponentRequirement {
    consumer: Component,
    contract_id: ContractId,
    provider: Component,
    provider_module: ModuleId,
    kind: ComponentRequirementKind,
}

pub trait ComponentRequirementService: Send + Sync {
    fn register(&self, requirement: ResolvedComponentRequirement) -> Result<(), ComponentError>;
    fn requirements(&self, consumer: &ComponentId) -> Vec<ResolvedComponentRequirement>;
    fn effective_availability(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentEffectiveAvailability, ComponentError>;
    fn effective_health(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentEffectiveHealth, ComponentError>;
}

#[derive(Clone)]
pub struct ComponentRequirementRail {
    inner: Arc<dyn ComponentRequirementService>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentEffectiveAvailability {
    Available,
    NotParticipating,
    NotActive {
        component: Component,
        participation: ComponentParticipation,
        state: ParticipationState,
    },
    IntrinsicUnavailable {
        component: Component,
        health: Health,
    },
    RequiredDependencyUnavailable(ComponentDependencyAvailabilityBlocker),
    MalformedCycle {
        component: Component,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentDependencyAvailabilityBlocker {
    contract_id: ContractId,
    provider: Component,
    provider_availability: Box<ComponentEffectiveAvailability>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentEffectiveHealth {
    Healthy,
    Degraded(DegradedComponentHealth),
    Unavailable(ComponentEffectiveAvailability),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DegradedComponentHealth {
    Intrinsic {
        component: Component,
        health: Health,
    },
    RequiredDependencyDegraded(ComponentDependencyHealthBlocker),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentDependencyHealthBlocker {
    contract_id: ContractId,
    provider: Component,
    provider_health: Box<ComponentEffectiveHealth>,
}

impl ResolvedComponentRequirement {
    #[cfg(test)]
    pub(crate) fn synthetic(
        consumer: Component,
        contract_id: ContractId,
        provider: Component,
        kind: ComponentRequirementKind,
    ) -> Self {
        Self::new_with_provider(
            consumer,
            contract_id,
            provider.clone(),
            synthetic_provider_module(&provider),
            kind,
        )
    }

    pub(crate) fn new_with_provider(
        consumer: Component,
        contract_id: ContractId,
        provider: Component,
        provider_module: ModuleId,
        kind: ComponentRequirementKind,
    ) -> Self {
        Self {
            consumer,
            contract_id,
            provider,
            provider_module,
            kind,
        }
    }
    pub fn consumer(&self) -> &Component {
        &self.consumer
    }
    pub fn contract_id(&self) -> &ContractId {
        &self.contract_id
    }
    pub fn provider(&self) -> &Component {
        &self.provider
    }
    pub fn provider_module(&self) -> &ModuleId {
        &self.provider_module
    }
    pub fn kind(&self) -> ComponentRequirementKind {
        self.kind
    }
}

#[cfg(test)]
fn synthetic_provider_module(provider: &Component) -> ModuleId {
    ModuleId::new(format!(
        "fabric.component.synthetic-provider.{}",
        provider.component_id().as_str()
    ))
    .expect("synthetic provider module id")
}

pub(crate) fn resolved_requirement_from_contract<T>(
    consumer: Component,
    contract_id: ContractId,
    kind: ComponentRequirementKind,
    resolved: fabric_core::ResolvedContract<ProvidedComponentContract<T>>,
) -> Result<(ResolvedComponentRequirement, Arc<T>), ComponentError> {
    let expected_provider_module = resolved.provider().clone();
    let provider_contract = resolved.into_value();
    if provider_contract.provider_module() != &expected_provider_module {
        return Err(
            ComponentError::ComponentContractProviderProvenanceMismatch {
                contract_id,
                expected_provider_module,
                actual_provider_module: provider_contract.provider_module().clone(),
            },
        );
    }
    let contract = provider_contract.contract().clone();
    let requirement = ResolvedComponentRequirement::new_with_provider(
        consumer,
        contract_id,
        provider_contract.provider().clone(),
        provider_contract.provider_module().clone(),
        kind,
    );
    Ok((requirement, contract))
}

impl ComponentEffectiveAvailability {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }
}

impl ComponentEffectiveHealth {
    pub fn health(&self) -> Health {
        match self {
            Self::Healthy => Health::Healthy,
            Self::Degraded(_) => Health::Degraded,
            Self::Unavailable(_) => Health::Unavailable,
        }
    }

    pub fn is_available(&self) -> bool {
        self.health() != Health::Unavailable
    }
}

impl ComponentDependencyAvailabilityBlocker {
    pub fn new(
        contract_id: ContractId,
        provider: Component,
        provider_availability: ComponentEffectiveAvailability,
    ) -> Self {
        Self {
            contract_id,
            provider,
            provider_availability: Box::new(provider_availability),
        }
    }

    pub fn contract_id(&self) -> &ContractId {
        &self.contract_id
    }

    pub fn provider(&self) -> &Component {
        &self.provider
    }

    pub fn provider_availability(&self) -> &ComponentEffectiveAvailability {
        self.provider_availability.as_ref()
    }
}

impl ComponentDependencyHealthBlocker {
    pub fn new(
        contract_id: ContractId,
        provider: Component,
        provider_health: ComponentEffectiveHealth,
    ) -> Self {
        Self {
            contract_id,
            provider,
            provider_health: Box::new(provider_health),
        }
    }

    pub fn contract_id(&self) -> &ContractId {
        &self.contract_id
    }

    pub fn provider(&self) -> &Component {
        &self.provider
    }

    pub fn provider_health(&self) -> &ComponentEffectiveHealth {
        self.provider_health.as_ref()
    }
}

impl ComponentRequirementRail {
    pub fn new(inner: Arc<dyn ComponentRequirementService>) -> Self {
        Self { inner }
    }
    pub fn register(
        &self,
        requirement: ResolvedComponentRequirement,
    ) -> Result<(), ComponentError> {
        self.inner.register(requirement)
    }

    pub fn register_resolved<T>(
        &self,
        consumer: Component,
        contract_id: ContractId,
        kind: ComponentRequirementKind,
        resolved: fabric_core::ResolvedContract<ProvidedComponentContract<T>>,
    ) -> Result<ResolvedComponentRequirement, ComponentError> {
        let (requirement, _contract) =
            resolved_requirement_from_contract(consumer, contract_id, kind, resolved)?;
        self.inner.register(requirement.clone())?;
        Ok(requirement)
    }
    pub fn requirements(&self, consumer: &ComponentId) -> Vec<ResolvedComponentRequirement> {
        self.inner.requirements(consumer)
    }

    pub fn effective_availability(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentEffectiveAvailability, ComponentError> {
        self.inner.effective_availability(component_id)
    }

    pub fn effective_health(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentEffectiveHealth, ComponentError> {
        self.inner.effective_health(component_id)
    }
}
