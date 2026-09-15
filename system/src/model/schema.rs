use std::cmp::Ordering;

use semver::{Version, VersionReq};

use crate::{SystemCompatibilityError, SystemId};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemSchemaVersion(Version);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemSchemaRequirement(VersionReq);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SystemSchemaIdentity {
    Provisional,
    Versioned(SystemSchemaVersion),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemSchemaCompatibilityRequirement {
    Provisional,
    Versioned(SystemSchemaRequirement),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterSystemSchemaSupport {
    system: SystemId,
    supported: SystemSchemaCompatibilityRequirement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemSchemaDescriptor {
    system: SystemId,
    identity: SystemSchemaIdentity,
}

impl SystemSchemaVersion {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, semver::Error> {
        Version::parse(value.as_ref()).map(Self)
    }

    pub fn as_semver(&self) -> &Version {
        &self.0
    }
}

impl std::fmt::Display for SystemSchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialOrd for SystemSchemaRequirement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SystemSchemaRequirement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.to_string().cmp(&other.0.to_string())
    }
}

impl SystemSchemaRequirement {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, semver::Error> {
        VersionReq::parse(value.as_ref()).map(Self)
    }

    pub fn accepts(&self, version: &SystemSchemaVersion) -> bool {
        self.0.matches(version.as_semver())
    }
}

impl std::fmt::Display for SystemSchemaRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl SystemSchemaIdentity {
    pub fn provisional() -> Self {
        Self::Provisional
    }

    pub fn versioned(version: SystemSchemaVersion) -> Self {
        Self::Versioned(version)
    }
}

impl std::fmt::Display for SystemSchemaIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provisional => f.write_str("provisional"),
            Self::Versioned(version) => version.fmt(f),
        }
    }
}

impl SystemSchemaCompatibilityRequirement {
    pub fn provisional() -> Self {
        Self::Provisional
    }

    pub fn versioned(requirement: SystemSchemaRequirement) -> Self {
        Self::Versioned(requirement)
    }

    pub fn accepts(&self, identity: &SystemSchemaIdentity) -> bool {
        match (self, identity) {
            (Self::Provisional, SystemSchemaIdentity::Provisional) => true,
            (Self::Versioned(requirement), SystemSchemaIdentity::Versioned(version)) => {
                requirement.accepts(version)
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for SystemSchemaCompatibilityRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provisional => f.write_str("provisional"),
            Self::Versioned(requirement) => requirement.fmt(f),
        }
    }
}

impl SystemSchemaDescriptor {
    pub fn provisional(system: SystemId) -> Self {
        Self {
            system,
            identity: SystemSchemaIdentity::provisional(),
        }
    }

    pub fn versioned(system: SystemId, version: SystemSchemaVersion) -> Self {
        Self {
            system,
            identity: SystemSchemaIdentity::versioned(version),
        }
    }

    pub fn system(&self) -> &SystemId {
        &self.system
    }

    pub fn identity(&self) -> &SystemSchemaIdentity {
        &self.identity
    }

    pub fn ensure_system(&self, system: &SystemId) -> Result<(), SystemCompatibilityError> {
        if self.system == *system {
            Ok(())
        } else {
            Err(SystemCompatibilityError::SystemIdentityMismatch {
                expected: system.clone(),
                actual: self.system.clone(),
            })
        }
    }

    pub fn ensure_compatible(
        &self,
        requirement: &SystemSchemaCompatibilityRequirement,
    ) -> Result<(), SystemCompatibilityError> {
        if requirement.accepts(&self.identity) {
            Ok(())
        } else {
            Err(SystemCompatibilityError::SchemaIncompatible {
                required: requirement.clone(),
                actual: self.identity.clone(),
            })
        }
    }
}

impl AdapterSystemSchemaSupport {
    pub fn provisional(system: SystemId) -> Self {
        Self {
            system,
            supported: SystemSchemaCompatibilityRequirement::provisional(),
        }
    }

    pub fn versioned(system: SystemId, requirement: SystemSchemaRequirement) -> Self {
        Self {
            system,
            supported: SystemSchemaCompatibilityRequirement::versioned(requirement),
        }
    }

    pub fn system(&self) -> &SystemId {
        &self.system
    }

    pub fn compatibility(&self) -> &SystemSchemaCompatibilityRequirement {
        &self.supported
    }

    pub fn accepts_schema(
        &self,
        schema: &SystemSchemaDescriptor,
    ) -> Result<(), SystemCompatibilityError> {
        schema.ensure_system(&self.system)?;
        schema.ensure_compatible(&self.supported)
    }
}
