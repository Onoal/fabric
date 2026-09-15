use std::any::{Any, TypeId};
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::sync::Arc;

use semver::{Version, VersionReq};

use crate::identifiers::ContractId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContractVersion(Version);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractVersionRequirement(VersionReq);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContractIdentity {
    Provisional,
    Versioned(ContractVersion),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContractCompatibilityRequirement {
    Provisional,
    Versioned(ContractVersionRequirement),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProvidedContractDeclaration {
    id: ContractId,
    identity: ContractIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractRequirementDeclaration {
    id: ContractId,
    compatibility: ContractCompatibilityRequirement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractKey<T>
where
    T: Send + Sync + 'static,
{
    id: ContractId,
    identity: ContractIdentity,
    marker: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractRequirement<T>
where
    T: Send + Sync + 'static,
{
    declaration: ContractRequirementDeclaration,
    marker: PhantomData<T>,
}

#[derive(Clone)]
pub struct ModuleContract {
    pub(crate) id: ContractId,
    pub(crate) identity: ContractIdentity,
    pub(crate) type_id: TypeId,
    pub(crate) value: Arc<dyn Any + Send + Sync>,
}

impl ContractVersion {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, semver::Error> {
        Version::parse(value.as_ref()).map(Self)
    }

    pub fn as_semver(&self) -> &Version {
        &self.0
    }
}

impl std::fmt::Display for ContractVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialOrd for ContractVersionRequirement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContractVersionRequirement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.to_string().cmp(&other.0.to_string())
    }
}

impl ContractVersionRequirement {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, semver::Error> {
        VersionReq::parse(value.as_ref()).map(Self)
    }

    pub fn accepts(&self, version: &ContractVersion) -> bool {
        self.0.matches(version.as_semver())
    }
}

impl std::fmt::Display for ContractVersionRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ContractIdentity {
    pub fn provisional() -> Self {
        Self::Provisional
    }

    pub fn versioned(version: ContractVersion) -> Self {
        Self::Versioned(version)
    }
}

impl std::fmt::Display for ContractIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provisional => f.write_str("provisional"),
            Self::Versioned(version) => version.fmt(f),
        }
    }
}

impl ContractCompatibilityRequirement {
    pub fn provisional() -> Self {
        Self::Provisional
    }

    pub fn versioned(requirement: ContractVersionRequirement) -> Self {
        Self::Versioned(requirement)
    }

    pub fn accepts(&self, identity: &ContractIdentity) -> bool {
        match (self, identity) {
            (Self::Provisional, ContractIdentity::Provisional) => true,
            (Self::Versioned(requirement), ContractIdentity::Versioned(version)) => {
                requirement.accepts(version)
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for ContractCompatibilityRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provisional => f.write_str("provisional"),
            Self::Versioned(requirement) => requirement.fmt(f),
        }
    }
}

impl ProvidedContractDeclaration {
    pub fn provisional(id: ContractId) -> Self {
        Self {
            id,
            identity: ContractIdentity::provisional(),
        }
    }

    pub fn versioned(id: ContractId, version: ContractVersion) -> Self {
        Self {
            id,
            identity: ContractIdentity::versioned(version),
        }
    }

    pub fn id(&self) -> &ContractId {
        &self.id
    }

    pub fn identity(&self) -> &ContractIdentity {
        &self.identity
    }
}

impl ContractRequirementDeclaration {
    pub fn provisional(id: ContractId) -> Self {
        Self {
            id,
            compatibility: ContractCompatibilityRequirement::provisional(),
        }
    }

    pub fn versioned(id: ContractId, requirement: ContractVersionRequirement) -> Self {
        Self {
            id,
            compatibility: ContractCompatibilityRequirement::versioned(requirement),
        }
    }

    pub fn id(&self) -> &ContractId {
        &self.id
    }

    pub fn compatibility(&self) -> &ContractCompatibilityRequirement {
        &self.compatibility
    }
}

impl<T> ContractKey<T>
where
    T: Send + Sync + 'static,
{
    pub fn provisional(id: ContractId) -> Self {
        Self {
            id,
            identity: ContractIdentity::provisional(),
            marker: PhantomData,
        }
    }

    pub fn versioned(id: ContractId, version: ContractVersion) -> Self {
        Self {
            id,
            identity: ContractIdentity::versioned(version),
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> &ContractId {
        &self.id
    }

    pub fn identity(&self) -> &ContractIdentity {
        &self.identity
    }

    pub fn declaration(&self) -> ProvidedContractDeclaration {
        ProvidedContractDeclaration {
            id: self.id.clone(),
            identity: self.identity.clone(),
        }
    }
}

impl<T> ContractRequirement<T>
where
    T: Send + Sync + 'static,
{
    pub fn provisional(id: ContractId) -> Self {
        Self {
            declaration: ContractRequirementDeclaration::provisional(id),
            marker: PhantomData,
        }
    }

    pub fn versioned(id: ContractId, requirement: ContractVersionRequirement) -> Self {
        Self {
            declaration: ContractRequirementDeclaration::versioned(id, requirement),
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> &ContractId {
        self.declaration.id()
    }

    pub fn compatibility(&self) -> &ContractCompatibilityRequirement {
        self.declaration.compatibility()
    }

    pub fn declaration(&self) -> &ContractRequirementDeclaration {
        &self.declaration
    }
}

impl ModuleContract {
    pub fn new<T>(key: &ContractKey<T>, value: Arc<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self {
            id: key.id().clone(),
            identity: key.identity().clone(),
            type_id: TypeId::of::<T>(),
            value,
        }
    }

    pub fn declaration(&self) -> ProvidedContractDeclaration {
        ProvidedContractDeclaration {
            id: self.id.clone(),
            identity: self.identity.clone(),
        }
    }
}
