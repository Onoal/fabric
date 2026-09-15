use std::marker::PhantomData;
use std::sync::Arc;

use fabric_core::{
    CompositionError, ContractRequirement, ContractRequirementDeclaration,
    ContractVersionRequirement, ModuleBindings, ResolvedContract,
};

use super::PrimarySystemContract;

pub struct SystemRequires<S>
where
    S: PrimarySystemContract,
{
    requirement: ContractRequirement<S::Contract>,
    marker: PhantomData<S>,
}

impl<S> Clone for SystemRequires<S>
where
    S: PrimarySystemContract,
{
    fn clone(&self) -> Self {
        Self {
            requirement: self.requirement.clone(),
            marker: PhantomData,
        }
    }
}

impl<S> SystemRequires<S>
where
    S: PrimarySystemContract,
{
    pub fn provisional() -> Self {
        Self::from_requirement(ContractRequirement::provisional(
            S::primary_contract_key().id().clone(),
        ))
    }

    pub fn versioned(requirement: ContractVersionRequirement) -> Self {
        Self::from_requirement(ContractRequirement::versioned(
            S::primary_contract_key().id().clone(),
            requirement,
        ))
    }

    pub(crate) fn from_requirement(requirement: ContractRequirement<S::Contract>) -> Self {
        Self {
            requirement,
            marker: PhantomData,
        }
    }

    pub fn declaration(&self) -> &ContractRequirementDeclaration {
        self.requirement.declaration()
    }

    pub fn resolve(&self, bindings: &ModuleBindings) -> Result<Arc<S::Contract>, CompositionError> {
        bindings.resolve(&self.requirement)
    }

    pub fn resolve_optional(
        &self,
        bindings: &ModuleBindings,
    ) -> Result<Option<Arc<S::Contract>>, CompositionError> {
        bindings.resolve_optional(&self.requirement)
    }

    pub fn resolve_with_provider(
        &self,
        bindings: &ModuleBindings,
    ) -> Result<ResolvedContract<S::Contract>, CompositionError> {
        bindings.resolve_with_provider(&self.requirement)
    }

    pub fn as_contract_requirement(&self) -> &ContractRequirement<S::Contract> {
        &self.requirement
    }
}
