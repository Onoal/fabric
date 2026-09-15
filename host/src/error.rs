use crate::{HostArchitecture, HostFacilityId, HostOperatingSystem};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum HostError {
    #[error("invalid host identifier for {kind}")]
    InvalidIdentifier { kind: &'static str },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum HostCompatibilityError {
    #[error(
        "host operating system `{actual}` is not supported; allowed operating systems: {allowed:?}"
    )]
    UnsupportedOperatingSystem {
        actual: HostOperatingSystem,
        allowed: Vec<HostOperatingSystem>,
    },

    #[error("host architecture `{actual}` is not supported; allowed architectures: {allowed:?}")]
    UnsupportedArchitecture {
        actual: HostArchitecture,
        allowed: Vec<HostArchitecture>,
    },

    #[error("host is missing required facilities {missing:?}")]
    MissingRequiredFacilities { missing: Vec<HostFacilityId> },
}
