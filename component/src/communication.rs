use std::sync::Arc;

use fabric_core::{ContractId, ContractKey, ModuleId, ResolvedContract};

use crate::{
    ComponentError, ComponentInstanceBinding, ComponentParticipation, ComponentRequirementKind,
    InvocationContext, ResolvedComponentRequirement,
    requirement::resolved_requirement_from_contract,
};

const COMPONENT_COMMUNICATION_CONTRACT_ID: &str = "fabric.component.communication";

pub fn component_communication_contract_id() -> ContractId {
    ContractId::new(COMPONENT_COMMUNICATION_CONTRACT_ID)
        .expect("static component communication contract id")
}

pub fn component_communication_contract_key() -> ContractKey<ComponentCommunication> {
    ContractKey::provisional(component_communication_contract_id())
}

pub trait ComponentCommunicationService: Send + Sync {
    fn bind_requirement(
        &self,
        requirement: &ResolvedComponentRequirement,
    ) -> Result<(), ComponentError>;

    fn validate_call(
        &self,
        requirement: &ResolvedComponentRequirement,
        caller: &ComponentParticipation,
        context: Option<&InvocationContext>,
    ) -> Result<(), ComponentError>;
}

#[derive(Clone)]
pub struct ComponentCommunication {
    inner: Arc<dyn ComponentCommunicationService>,
}

#[derive(Clone)]
pub struct ComponentContract<T> {
    requirement: ResolvedComponentRequirement,
    contract: Arc<T>,
    communication: ComponentCommunication,
}

#[derive(Clone)]
pub struct ProvidedComponentContract<T> {
    provider: ComponentInstanceBinding,
    provider_module: ModuleId,
    contract: Arc<T>,
}

impl ComponentCommunication {
    pub fn new(inner: Arc<dyn ComponentCommunicationService>) -> Self {
        Self { inner }
    }

    /// Wraps a contract implementation that Fabric Core has already resolved.
    /// The handle is scoped to the resolved consumer-provider relationship.
    pub(crate) fn bind<T>(
        &self,
        requirement: ResolvedComponentRequirement,
        contract: Arc<T>,
    ) -> Result<ComponentContract<T>, ComponentError> {
        self.inner.bind_requirement(&requirement)?;
        Ok(ComponentContract {
            requirement,
            contract,
            communication: self.clone(),
        })
    }

    pub fn bind_resolved<T>(
        &self,
        consumer: ComponentInstanceBinding,
        contract_id: ContractId,
        kind: ComponentRequirementKind,
        resolved: ResolvedContract<ProvidedComponentContract<T>>,
    ) -> Result<ComponentContract<T>, ComponentError> {
        let (requirement, contract) =
            resolved_requirement_from_contract(consumer, contract_id, kind, resolved)?;
        self.bind(requirement, contract)
    }
}

impl<T> ComponentContract<T> {
    pub fn provider(&self) -> &ComponentInstanceBinding {
        self.requirement.provider()
    }

    pub fn consumer(&self) -> &ComponentInstanceBinding {
        self.requirement.consumer()
    }

    pub fn requirement(&self) -> &ResolvedComponentRequirement {
        &self.requirement
    }

    /// Validates participation before invoking the typed internal contract.
    /// The validation happens before `call` starts, so an already-started call is
    /// deliberately allowed to finish if either participant subsequently leaves.
    pub fn call<R>(
        &self,
        caller: &ComponentParticipation,
        call: impl FnOnce(&T) -> R,
    ) -> Result<R, ComponentError> {
        self.communication
            .inner
            .validate_call(&self.requirement, caller, None)?;
        Ok(call(self.contract.as_ref()))
    }

    pub fn call_with_context<R>(
        &self,
        caller: &ComponentParticipation,
        context: &InvocationContext,
        call: impl FnOnce(&InvocationContext, &T) -> R,
    ) -> Result<R, ComponentError> {
        self.communication
            .inner
            .validate_call(&self.requirement, caller, Some(context))?;
        Ok(call(context, self.contract.as_ref()))
    }
}

impl<T> ProvidedComponentContract<T> {
    pub fn new(
        provider: ComponentInstanceBinding,
        provider_module: ModuleId,
        contract: Arc<T>,
    ) -> Self {
        Self {
            provider,
            provider_module,
            contract,
        }
    }

    pub fn provider(&self) -> &ComponentInstanceBinding {
        &self.provider
    }

    pub fn provider_module(&self) -> &ModuleId {
        &self.provider_module
    }

    pub fn contract(&self) -> &Arc<T> {
        &self.contract
    }
}
