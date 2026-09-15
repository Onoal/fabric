use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::contract::{ContractIdentity, ContractRequirement, ContractRequirementDeclaration};
use crate::error::CompositionError;
use crate::identifiers::{ContractId, ModuleId};

#[derive(Clone)]
pub(crate) struct BoundContract {
    pub(crate) provider: ModuleId,
    pub(crate) identity: ContractIdentity,
    pub(crate) type_id: TypeId,
    pub(crate) value: Arc<dyn Any + Send + Sync>,
}

#[derive(Clone)]
pub struct ResolvedContract<T> {
    provider: ModuleId,
    identity: ContractIdentity,
    value: Arc<T>,
}

pub struct ModuleBindings {
    consumer: ModuleId,
    declarations: BTreeMap<ContractId, ContractRequirementDeclaration>,
    contracts: BTreeMap<ContractId, BoundContract>,
}

pub(crate) type ResolvedBindings = BTreeMap<ModuleId, BTreeMap<ContractId, BoundContract>>;

impl ModuleBindings {
    pub(crate) fn new(
        consumer: ModuleId,
        declarations: BTreeMap<ContractId, ContractRequirementDeclaration>,
        contracts: BTreeMap<ContractId, BoundContract>,
    ) -> Self {
        Self {
            consumer,
            declarations,
            contracts,
        }
    }

    pub fn resolve<T>(
        &self,
        requirement: &ContractRequirement<T>,
    ) -> Result<Arc<T>, CompositionError>
    where
        T: Send + Sync + 'static,
    {
        self.resolve_with_provider(requirement)
            .map(ResolvedContract::into_value)
    }

    pub fn resolve_with_provider<T>(
        &self,
        requirement: &ContractRequirement<T>,
    ) -> Result<ResolvedContract<T>, CompositionError>
    where
        T: Send + Sync + 'static,
    {
        self.ensure_requirement_matches(requirement)?;
        let contract_id = requirement.id();
        let Some(bound) = self.contracts.get(contract_id) else {
            return Err(CompositionError::UnboundRequirement {
                module_id: self.consumer.clone(),
                contract_id: contract_id.clone(),
            });
        };
        if bound.type_id != TypeId::of::<T>() {
            return Err(CompositionError::ContractTypeMismatch {
                module_id: self.consumer.clone(),
                contract_id: contract_id.clone(),
            });
        }

        let value = bound.value.clone();
        value
            .downcast::<T>()
            .map(|value| ResolvedContract {
                provider: bound.provider.clone(),
                identity: bound.identity.clone(),
                value,
            })
            .map_err(|_| CompositionError::ContractTypeMismatch {
                module_id: self.consumer.clone(),
                contract_id: contract_id.clone(),
            })
    }

    pub fn resolve_optional<T>(
        &self,
        requirement: &ContractRequirement<T>,
    ) -> Result<Option<Arc<T>>, CompositionError>
    where
        T: Send + Sync + 'static,
    {
        self.ensure_requirement_matches(requirement)?;
        let contract_id = requirement.id();
        let Some(bound) = self.contracts.get(contract_id) else {
            return Ok(None);
        };
        if bound.type_id != TypeId::of::<T>() {
            return Err(CompositionError::ContractTypeMismatch {
                module_id: self.consumer.clone(),
                contract_id: contract_id.clone(),
            });
        }

        let value = bound.value.clone();
        value
            .downcast::<T>()
            .map(Some)
            .map_err(|_| CompositionError::ContractTypeMismatch {
                module_id: self.consumer.clone(),
                contract_id: contract_id.clone(),
            })
    }

    fn ensure_requirement_matches<T>(
        &self,
        requirement: &ContractRequirement<T>,
    ) -> Result<(), CompositionError>
    where
        T: Send + Sync + 'static,
    {
        let contract_id = requirement.id();
        let Some(declared) = self.declarations.get(contract_id) else {
            return Err(CompositionError::UnboundRequirement {
                module_id: self.consumer.clone(),
                contract_id: contract_id.clone(),
            });
        };
        if declared != requirement.declaration() {
            return Err(CompositionError::RequirementDeclarationMismatch {
                module_id: self.consumer.clone(),
                contract_id: contract_id.clone(),
                declared_compatibility: declared.compatibility().clone(),
                requested_compatibility: requirement.compatibility().clone(),
            });
        }
        Ok(())
    }
}

impl<T> ResolvedContract<T> {
    pub fn provider(&self) -> &ModuleId {
        &self.provider
    }

    pub fn identity(&self) -> &ContractIdentity {
        &self.identity
    }

    pub fn value(&self) -> &Arc<T> {
        &self.value
    }

    pub fn into_value(self) -> Arc<T> {
        self.value
    }
}
