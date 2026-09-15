use std::collections::BTreeSet;
use std::sync::Arc;

use fabric_core::{ContractId, ContractKey};

use crate::{ComponentEffectiveHealth, ComponentError, ComponentId, ComponentRuntimeStatus};

const COMPONENT_READINESS_CONTRACT_ID: &str = "fabric.component.readiness";

pub fn component_readiness_contract_id() -> ContractId {
    ContractId::new(COMPONENT_READINESS_CONTRACT_ID).expect("static readiness contract id")
}

pub fn component_readiness_contract_key() -> ContractKey<ComponentReadinessRail> {
    ContractKey::provisional(component_readiness_contract_id())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentReadinessPolicy {
    required_components: BTreeSet<ComponentId>,
}

pub trait ComponentReadinessService: Send + Sync {
    fn policy(&self) -> ComponentReadinessPolicy;
    fn aggregate_readiness(&self) -> ComponentAggregateReadiness;
}

#[derive(Clone)]
pub struct ComponentReadinessRail {
    inner: Arc<dyn ComponentReadinessService>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentAggregateReadiness {
    status: ComponentRuntimeStatus,
    policy: ComponentReadinessPolicy,
    blockers: Vec<ComponentAggregateBlocker>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentAggregateBlocker {
    RequiredComponentNotParticipating {
        component_id: ComponentId,
    },
    RequiredComponentUnavailable {
        component_id: ComponentId,
        effective_health: ComponentEffectiveHealth,
    },
    RequiredComponentDegraded {
        component_id: ComponentId,
        effective_health: ComponentEffectiveHealth,
    },
}

impl ComponentReadinessPolicy {
    pub fn new(
        required_components: impl IntoIterator<Item = ComponentId>,
    ) -> Result<Self, ComponentError> {
        Ok(Self {
            required_components: required_components.into_iter().collect(),
        })
    }

    pub fn empty() -> Self {
        Self {
            required_components: BTreeSet::new(),
        }
    }

    pub fn required_components(&self) -> &BTreeSet<ComponentId> {
        &self.required_components
    }

    pub fn is_empty(&self) -> bool {
        self.required_components.is_empty()
    }
}

impl ComponentReadinessRail {
    pub fn new(inner: Arc<dyn ComponentReadinessService>) -> Self {
        Self { inner }
    }

    pub fn policy(&self) -> ComponentReadinessPolicy {
        self.inner.policy()
    }

    pub fn aggregate_readiness(&self) -> ComponentAggregateReadiness {
        self.inner.aggregate_readiness()
    }
}

impl ComponentAggregateReadiness {
    pub fn new(
        status: ComponentRuntimeStatus,
        policy: ComponentReadinessPolicy,
        blockers: Vec<ComponentAggregateBlocker>,
    ) -> Self {
        Self {
            status,
            policy,
            blockers,
        }
    }

    pub fn status(&self) -> &ComponentRuntimeStatus {
        &self.status
    }

    pub fn policy(&self) -> &ComponentReadinessPolicy {
        &self.policy
    }

    pub fn blockers(&self) -> &[ComponentAggregateBlocker] {
        &self.blockers
    }
}
