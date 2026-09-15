use std::marker::PhantomData;
use std::sync::Arc;

use fabric_core::{
    CompositionError, ContractRequirement, ContractRequirementDeclaration,
    ContractVersionRequirement, ModuleBindings, ResolvedContract,
};

use super::PrimaryResourceContract;

pub struct Requires<R>
where
    R: PrimaryResourceContract,
{
    requirement: ContractRequirement<R::Contract>,
    marker: PhantomData<R>,
}

impl<R> Clone for Requires<R>
where
    R: PrimaryResourceContract,
{
    fn clone(&self) -> Self {
        Self {
            requirement: self.requirement.clone(),
            marker: PhantomData,
        }
    }
}

impl<R> Requires<R>
where
    R: PrimaryResourceContract,
{
    pub fn provisional() -> Self {
        Self::from_requirement(ContractRequirement::provisional(
            R::primary_contract_key().id().clone(),
        ))
    }

    pub fn versioned(requirement: ContractVersionRequirement) -> Self {
        Self::from_requirement(ContractRequirement::versioned(
            R::primary_contract_key().id().clone(),
            requirement,
        ))
    }

    pub(crate) fn from_requirement(requirement: ContractRequirement<R::Contract>) -> Self {
        Self {
            requirement,
            marker: PhantomData,
        }
    }

    pub fn declaration(&self) -> &ContractRequirementDeclaration {
        self.requirement.declaration()
    }

    pub fn resolve(&self, bindings: &ModuleBindings) -> Result<Arc<R::Contract>, CompositionError> {
        bindings.resolve(&self.requirement)
    }

    pub fn resolve_optional(
        &self,
        bindings: &ModuleBindings,
    ) -> Result<Option<Arc<R::Contract>>, CompositionError> {
        bindings.resolve_optional(&self.requirement)
    }

    pub fn resolve_with_provider(
        &self,
        bindings: &ModuleBindings,
    ) -> Result<ResolvedContract<R::Contract>, CompositionError> {
        bindings.resolve_with_provider(&self.requirement)
    }

    pub fn as_contract_requirement(&self) -> &ContractRequirement<R::Contract> {
        &self.requirement
    }
}
