use fabric_core::{ContractVersion, ContractVersionRequirement};

use crate::SdkAuthoringError;

pub trait IntoContractVersion {
    fn into_contract_version(self) -> Result<ContractVersion, SdkAuthoringError>;
}

pub trait IntoContractVersionRequirement {
    fn into_contract_version_requirement(
        self,
    ) -> Result<ContractVersionRequirement, SdkAuthoringError>;
}

pub fn contract_version(
    value: impl IntoContractVersion,
) -> Result<ContractVersion, SdkAuthoringError> {
    value.into_contract_version()
}

pub fn contract_requirement(
    value: impl IntoContractVersionRequirement,
) -> Result<ContractVersionRequirement, SdkAuthoringError> {
    value.into_contract_version_requirement()
}

impl IntoContractVersion for ContractVersion {
    fn into_contract_version(self) -> Result<ContractVersion, SdkAuthoringError> {
        Ok(self)
    }
}

impl IntoContractVersionRequirement for ContractVersionRequirement {
    fn into_contract_version_requirement(
        self,
    ) -> Result<ContractVersionRequirement, SdkAuthoringError> {
        Ok(self)
    }
}

impl IntoContractVersion for &ContractVersion {
    fn into_contract_version(self) -> Result<ContractVersion, SdkAuthoringError> {
        Ok(self.clone())
    }
}

impl IntoContractVersionRequirement for &ContractVersionRequirement {
    fn into_contract_version_requirement(
        self,
    ) -> Result<ContractVersionRequirement, SdkAuthoringError> {
        Ok(self.clone())
    }
}

impl IntoContractVersion for &str {
    fn into_contract_version(self) -> Result<ContractVersion, SdkAuthoringError> {
        ContractVersion::parse(self)
            .map_err(|error| SdkAuthoringError::invalid_contract_version(self.to_owned(), error))
    }
}

impl IntoContractVersionRequirement for &str {
    fn into_contract_version_requirement(
        self,
    ) -> Result<ContractVersionRequirement, SdkAuthoringError> {
        ContractVersionRequirement::parse(self).map_err(|error| {
            SdkAuthoringError::invalid_contract_version_requirement(self.to_owned(), error)
        })
    }
}

impl IntoContractVersion for String {
    fn into_contract_version(self) -> Result<ContractVersion, SdkAuthoringError> {
        let value = self.clone();
        ContractVersion::parse(&self)
            .map_err(|error| SdkAuthoringError::invalid_contract_version(value, error))
    }
}

impl IntoContractVersionRequirement for String {
    fn into_contract_version_requirement(
        self,
    ) -> Result<ContractVersionRequirement, SdkAuthoringError> {
        let value = self.clone();
        ContractVersionRequirement::parse(&self)
            .map_err(|error| SdkAuthoringError::invalid_contract_version_requirement(value, error))
    }
}
