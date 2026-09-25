use std::cmp::Ordering;

use semver::{Version, VersionReq};

use crate::ResourceId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceSchemaVersion(Version);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceSchemaRequirement(VersionReq);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceSchemaIdentity {
    Provisional,
    Versioned(ResourceSchemaVersion),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceSchemaCompatibilityRequirement {
    Provisional,
    Versioned(ResourceSchemaRequirement),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceSchemaDescriptor {
    resource: ResourceId,
    identity: ResourceSchemaIdentity,
}

impl ResourceSchemaVersion {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, semver::Error> {
        Version::parse(value.as_ref()).map(Self)
    }

    pub fn as_semver(&self) -> &Version {
        &self.0
    }
}

impl std::fmt::Display for ResourceSchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialOrd for ResourceSchemaRequirement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResourceSchemaRequirement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.to_string().cmp(&other.0.to_string())
    }
}

impl ResourceSchemaRequirement {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, semver::Error> {
        VersionReq::parse(value.as_ref()).map(Self)
    }

    pub fn accepts(&self, version: &ResourceSchemaVersion) -> bool {
        self.0.matches(version.as_semver())
    }
}

impl std::fmt::Display for ResourceSchemaRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ResourceSchemaIdentity {
    pub fn provisional() -> Self {
        Self::Provisional
    }

    pub fn versioned(version: ResourceSchemaVersion) -> Self {
        Self::Versioned(version)
    }
}

impl std::fmt::Display for ResourceSchemaIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provisional => f.write_str("provisional"),
            Self::Versioned(version) => version.fmt(f),
        }
    }
}

impl ResourceSchemaCompatibilityRequirement {
    pub fn provisional() -> Self {
        Self::Provisional
    }

    pub fn versioned(requirement: ResourceSchemaRequirement) -> Self {
        Self::Versioned(requirement)
    }

    pub fn accepts(&self, identity: &ResourceSchemaIdentity) -> bool {
        match (self, identity) {
            (Self::Provisional, ResourceSchemaIdentity::Provisional) => true,
            (Self::Versioned(requirement), ResourceSchemaIdentity::Versioned(version)) => {
                requirement.accepts(version)
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for ResourceSchemaCompatibilityRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provisional => f.write_str("provisional"),
            Self::Versioned(requirement) => requirement.fmt(f),
        }
    }
}

impl ResourceSchemaDescriptor {
    pub fn provisional(resource: ResourceId) -> Self {
        Self {
            resource,
            identity: ResourceSchemaIdentity::provisional(),
        }
    }

    pub fn versioned(resource: ResourceId, version: ResourceSchemaVersion) -> Self {
        Self {
            resource,
            identity: ResourceSchemaIdentity::versioned(version),
        }
    }

    pub fn resource(&self) -> &ResourceId {
        &self.resource
    }

    pub fn identity(&self) -> &ResourceSchemaIdentity {
        &self.identity
    }
}
