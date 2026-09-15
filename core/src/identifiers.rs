use std::fmt;

use crate::error::CompositionError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompositionId(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContractId(String);

impl BlockId {
    pub fn new(value: impl Into<String>) -> Result<Self, CompositionError> {
        validate_id("BlockId", value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl CompositionId {
    pub fn new(value: impl Into<String>) -> Result<Self, CompositionError> {
        validate_id("CompositionId", value.into()).map(Self)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl InstanceId {
    pub fn new(value: impl Into<String>) -> Result<Self, CompositionError> {
        validate_id("InstanceId", value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ModuleId {
    pub fn new(value: impl Into<String>) -> Result<Self, CompositionError> {
        validate_id("ModuleId", value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ContractId {
    pub fn new(value: impl Into<String>) -> Result<Self, CompositionError> {
        validate_id("ContractId", value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for CompositionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for ContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_id(kind: &'static str, value: String) -> Result<String, CompositionError> {
    if value.trim() != value || value.is_empty() {
        return Err(CompositionError::InvalidIdentifier {
            kind,
            value: value.to_owned(),
        });
    }
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        Ok(value)
    } else {
        Err(CompositionError::InvalidIdentifier { kind, value })
    }
}
