use crate::BindingError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingName(String);

impl BindingName {
    pub fn new(value: impl Into<String>) -> Result<Self, BindingError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 128
            && value.bytes().all(|byte| {
                !byte.is_ascii_whitespace()
                    && !byte.is_ascii_control()
                    && !matches!(byte, b'/' | b'\\' | b':')
            });
        if valid {
            Ok(Self(value))
        } else {
            Err(BindingError::invalid_input("invalid binding name"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
