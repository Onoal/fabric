use std::fmt;
use std::sync::Arc;

use fabric_core::{ContractId, ContractKey, InstanceGeneration, InstanceId};

use crate::{Component, ComponentError, ComponentParticipation};

const COMPONENT_INVOCATION_CONTRACT_ID: &str = "fabric.component.invocation";

pub fn invocation_contract_id() -> ContractId {
    ContractId::new(COMPONENT_INVOCATION_CONTRACT_ID)
        .expect("static component invocation contract id")
}

pub fn invocation_contract_key() -> ContractKey<InvocationRail> {
    ContractKey::provisional(invocation_contract_id())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InvocationId(u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvocationOrigin {
    External,
    Component(Component),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvocationContext {
    instance_id: InstanceId,
    generation: InstanceGeneration,
    invocation_id: InvocationId,
    root_origin: InvocationOrigin,
}

pub trait InvocationService: Send + Sync {
    fn begin_external(&self) -> Result<InvocationContext, ComponentError>;

    fn begin_component(
        &self,
        participation: ComponentParticipation,
    ) -> Result<InvocationContext, ComponentError>;

    fn validate_context(&self, context: &InvocationContext) -> Result<(), ComponentError>;
}

#[derive(Clone)]
pub struct InvocationRail {
    inner: Arc<dyn InvocationService>,
}

impl InvocationId {
    pub(crate) fn new(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for InvocationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl InvocationContext {
    pub(crate) fn new(
        instance_id: InstanceId,
        generation: InstanceGeneration,
        invocation_id: InvocationId,
        root_origin: InvocationOrigin,
    ) -> Self {
        Self {
            instance_id,
            generation,
            invocation_id,
            root_origin,
        }
    }

    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }

    pub fn invocation_id(&self) -> InvocationId {
        self.invocation_id
    }

    pub fn generation(&self) -> InstanceGeneration {
        self.generation
    }

    pub fn root_origin(&self) -> &InvocationOrigin {
        &self.root_origin
    }
}

impl InvocationRail {
    pub fn new(inner: Arc<dyn InvocationService>) -> Self {
        Self { inner }
    }

    pub fn begin_external(&self) -> Result<InvocationContext, ComponentError> {
        self.inner.begin_external()
    }

    pub fn begin_component(
        &self,
        participation: ComponentParticipation,
    ) -> Result<InvocationContext, ComponentError> {
        self.inner.begin_component(participation)
    }
}
