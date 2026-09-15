use std::ops::Deref;
use std::sync::{Arc, OnceLock};

use fabric_core::{
    CompositionError, ContractCompatibilityRequirement, ContractIdentity, ContractRequirement,
    ContractRequirementDeclaration, ContractVersionRequirement, ModuleBindings, ModuleId,
};

struct BoundContractState<T> {
    provider: ModuleId,
    identity: ContractIdentity,
    value: Arc<T>,
}

struct ContractDependencyState<T> {
    resolved: OnceLock<BoundContractState<T>>,
}

impl<T> Default for ContractDependencyState<T> {
    fn default() -> Self {
        Self {
            resolved: OnceLock::new(),
        }
    }
}

pub struct ContractDependency<T>
where
    T: Send + Sync + 'static,
{
    requirement: ContractRequirement<T>,
    state: Arc<ContractDependencyState<T>>,
}

impl<T> Clone for ContractDependency<T>
where
    T: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            requirement: clone_requirement(&self.requirement),
            state: Arc::clone(&self.state),
        }
    }
}

impl<T> ContractDependency<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(requirement: ContractRequirement<T>) -> Self {
        Self {
            requirement,
            state: Arc::new(ContractDependencyState::default()),
        }
    }

    pub fn declaration(&self) -> &ContractRequirementDeclaration {
        self.requirement.declaration()
    }

    pub fn requirement(&self) -> &ContractRequirement<T> {
        &self.requirement
    }

    pub fn bind(&self, bindings: &ModuleBindings) -> Result<(), CompositionError> {
        let resolved = bindings.resolve_with_provider(&self.requirement)?;
        let _ = self.state.resolved.set(BoundContractState {
            provider: resolved.provider().clone(),
            identity: resolved.identity().clone(),
            value: resolved.into_value(),
        });
        Ok(())
    }

    pub fn value(&self) -> Arc<T> {
        Arc::clone(&self.bound_state().value)
    }

    pub fn provider(&self) -> ModuleId {
        self.bound_state().provider.clone()
    }

    pub fn identity(&self) -> ContractIdentity {
        self.bound_state().identity.clone()
    }

    fn bound_state(&self) -> &BoundContractState<T> {
        self.state
            .resolved
            .get()
            .expect("contract dependency must be bound before use")
    }
}

impl<T> Deref for ContractDependency<T>
where
    T: Send + Sync + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.bound_state().value.as_ref()
    }
}

fn clone_requirement<T>(requirement: &ContractRequirement<T>) -> ContractRequirement<T>
where
    T: Send + Sync + 'static,
{
    match requirement.compatibility() {
        ContractCompatibilityRequirement::Provisional => {
            ContractRequirement::provisional(requirement.id().clone())
        }
        ContractCompatibilityRequirement::Versioned(version) => ContractRequirement::versioned(
            requirement.id().clone(),
            clone_version_requirement(version),
        ),
    }
}

fn clone_version_requirement(
    requirement: &ContractVersionRequirement,
) -> ContractVersionRequirement {
    ContractVersionRequirement::parse(requirement.to_string())
        .expect("contract version requirement display should round-trip")
}
