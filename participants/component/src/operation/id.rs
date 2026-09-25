use std::fmt;

use crate::ComponentError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationId(String);

impl OperationId {
    pub fn new(value: impl Into<String>) -> Result<Self, ComponentError> {
        let value = value.into();
        validate_operation_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_operation_id(value: &str) -> Result<(), ComponentError> {
    if value.trim() != value || value.is_empty() {
        return Err(ComponentError::InvalidOperationId(value.to_owned()));
    }
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Ok(());
    }
    Err(ComponentError::InvalidOperationId(value.to_owned()))
}
