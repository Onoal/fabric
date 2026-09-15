use crate::BindingError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingTargetKind(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingTargetId(String);

impl BindingTargetKind {
    pub fn new(value: impl Into<String>) -> Result<Self, BindingError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 128
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'.' | b'-' | b'_')
            });
        if valid {
            Ok(Self(value))
        } else {
            Err(BindingError::invalid_input("invalid binding target kind"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl BindingTargetId {
    pub fn new(value: impl Into<String>) -> Result<Self, BindingError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 512
            && !value
                .bytes()
                .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control());
        if valid {
            Ok(Self(value))
        } else {
            Err(BindingError::invalid_input("invalid binding target id"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
