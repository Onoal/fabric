use std::fmt;

use crate::HostError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HostOperatingSystem(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HostArchitecture(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HostFacilityId(String);

impl HostOperatingSystem {
    pub fn new(value: impl Into<String>) -> Result<Self, HostError> {
        let value = value.into();
        validate_host_identifier(&value, "host operating system").map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl HostArchitecture {
    pub fn new(value: impl Into<String>) -> Result<Self, HostError> {
        let value = value.into();
        validate_host_identifier(&value, "host architecture").map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl HostFacilityId {
    pub fn new(value: impl Into<String>) -> Result<Self, HostError> {
        let value = value.into();
        validate_host_identifier(&value, "host facility").map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HostOperatingSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for HostArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for HostFacilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_host_identifier(value: &str, kind: &'static str) -> Result<String, HostError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..");
    if valid {
        Ok(value.to_owned())
    } else {
        Err(HostError::InvalidIdentifier { kind })
    }
}
