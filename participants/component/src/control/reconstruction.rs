use std::sync::Arc;

use fabric_core::{ContractId, ContractKey};

use crate::{
    ComponentControl, ComponentDesiredState, ComponentError, ComponentId, ComponentParticipation,
    ComponentStatus,
};

const COMPONENT_RECONSTRUCTION_CONTRACT_ID: &str = "fabric.component.reconstruction";

pub fn component_reconstruction_contract_id() -> ContractId {
    ContractId::new(COMPONENT_RECONSTRUCTION_CONTRACT_ID)
        .expect("static component reconstruction contract id")
}

pub fn component_reconstruction_contract_key() -> ContractKey<ComponentReconstructionRail> {
    ContractKey::provisional(component_reconstruction_contract_id())
}

pub trait ComponentReconstructionService: Send + Sync {
    fn reconstruct(&self) -> Result<ComponentReconstructionReport, ComponentError>;
}

#[derive(Clone)]
pub struct ComponentReconstructionRail {
    inner: Arc<dyn ComponentReconstructionService>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentReconstructionReport {
    outcomes: Vec<ComponentReconstructionOutcome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentReconstructionOutcome {
    control: ComponentControl,
    observed_runtime: ComponentReconstructionRuntimeState,
    result: ComponentReconstructionResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentReconstructionRuntimeState {
    Absent,
    Preparing(ComponentStatus),
    Active(ComponentStatus),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentReconstructionResult {
    AlreadyConverged,
    Materialized(ComponentStatus),
    Dematerialized(ComponentStatus),
    BlockedPreparing(ComponentParticipation),
    MissingRuntimeAttachment,
    UndeclaredComponent,
    MaterializationFailed(ComponentError),
    DematerializationFailed(ComponentError),
    PresentButUnmanaged,
}

impl ComponentReconstructionRail {
    pub fn new(inner: Arc<dyn ComponentReconstructionService>) -> Self {
        Self { inner }
    }

    pub fn reconstruct(&self) -> Result<ComponentReconstructionReport, ComponentError> {
        self.inner.reconstruct()
    }
}

impl ComponentReconstructionReport {
    pub fn new(outcomes: Vec<ComponentReconstructionOutcome>) -> Self {
        Self { outcomes }
    }

    pub fn outcomes(&self) -> &[ComponentReconstructionOutcome] {
        &self.outcomes
    }
}

impl ComponentReconstructionOutcome {
    pub fn new(
        control: ComponentControl,
        observed_runtime: ComponentReconstructionRuntimeState,
        result: ComponentReconstructionResult,
    ) -> Self {
        Self {
            control,
            observed_runtime,
            result,
        }
    }

    pub fn component_id(&self) -> &ComponentId {
        self.control.component().component_id()
    }

    pub fn desired(&self) -> ComponentDesiredState {
        self.control.desired()
    }

    pub fn control(&self) -> &ComponentControl {
        &self.control
    }

    pub fn observed_runtime(&self) -> &ComponentReconstructionRuntimeState {
        &self.observed_runtime
    }

    pub fn result(&self) -> &ComponentReconstructionResult {
        &self.result
    }
}

impl ComponentReconstructionRuntimeState {
    pub fn from_status(status: ComponentStatus) -> Self {
        if status.is_active() {
            Self::Active(status)
        } else {
            Self::Preparing(status)
        }
    }
}
