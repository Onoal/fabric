use fabric_core::{CompositionError, ContractKey, ContractRequirement};

use crate::SdkAuthoringError;
use crate::ids::{IntoContractId, contract};
use crate::versions::{
    IntoContractVersion, IntoContractVersionRequirement, contract_requirement, contract_version,
};

pub fn provisional_provider<T>(id: impl IntoContractId) -> Result<ContractKey<T>, CompositionError>
where
    T: Send + Sync + 'static,
{
    Ok(ContractKey::provisional(contract(id)?))
}

pub fn versioned_provider<T>(
    id: impl IntoContractId,
    version: impl IntoContractVersion,
) -> Result<ContractKey<T>, SdkAuthoringError>
where
    T: Send + Sync + 'static,
{
    Ok(ContractKey::versioned(
        contract(id)?,
        contract_version(version)?,
    ))
}

pub fn provisional_requirement<T>(
    id: impl IntoContractId,
) -> Result<ContractRequirement<T>, CompositionError>
where
    T: Send + Sync + 'static,
{
    Ok(ContractRequirement::provisional(contract(id)?))
}

pub fn versioned_requirement<T>(
    id: impl IntoContractId,
    requirement: impl IntoContractVersionRequirement,
) -> Result<ContractRequirement<T>, SdkAuthoringError>
where
    T: Send + Sync + 'static,
{
    Ok(ContractRequirement::versioned(
        contract(id)?,
        contract_requirement(requirement)?,
    ))
}
