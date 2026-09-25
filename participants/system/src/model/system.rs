use std::fmt;

use crate::SystemCompatibilityError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemId(String);

impl SystemId {
    pub fn new(value: impl Into<String>) -> Result<Self, SystemCompatibilityError> {
        let value = value.into();
        validate_system_key(&value, "system id")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub(crate) fn validate_system_key(
    value: &str,
    label: &str,
) -> Result<(), SystemCompatibilityError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..");
    if valid {
        Ok(())
    } else {
        Err(SystemCompatibilityError::InvalidInput {
            message: format!("invalid {label}"),
        })
    }
}
