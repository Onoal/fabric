use crate::BindingError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingConsumerKind(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingConsumerId(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingConsumer {
    kind: BindingConsumerKind,
    id: BindingConsumerId,
}

impl BindingConsumerKind {
    pub fn new(value: impl Into<String>) -> Result<Self, BindingError> {
        let value = value.into();
        validate_kind(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl BindingConsumerId {
    pub fn new(value: impl Into<String>) -> Result<Self, BindingError> {
        let value = value.into();
        validate_identity(&value, "binding consumer id")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl BindingConsumer {
    pub fn new(kind: BindingConsumerKind, id: BindingConsumerId) -> Self {
        Self { kind, id }
    }

    pub fn kind(&self) -> &BindingConsumerKind {
        &self.kind
    }

    pub fn id(&self) -> &BindingConsumerId {
        &self.id
    }
}

fn validate_kind(value: &str) -> Result<(), BindingError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        });
    if valid {
        Ok(())
    } else {
        Err(BindingError::invalid_input("invalid binding consumer kind"))
    }
}

fn validate_identity(value: &str, label: &str) -> Result<(), BindingError> {
    let valid = !value.is_empty()
        && value.len() <= 512
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control());
    if valid {
        Ok(())
    } else {
        Err(BindingError::invalid_input(format!("invalid {label}")))
    }
}
