use std::fmt;

use fabric_core::InstanceId;

use crate::{ComponentError, ComponentHost};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentInstanceBinding {
    component_id: ComponentId,
    instance_id: InstanceId,
}

impl ComponentId {
    pub fn new(value: impl Into<String>) -> Result<Self, ComponentError> {
        let value = value.into();
        validate_component_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ComponentInstanceBinding {
    pub fn bind(component_id: ComponentId, runtime: &ComponentHost) -> Self {
        Self {
            component_id,
            instance_id: runtime.instance_id(),
        }
    }

    pub fn for_instance(component_id: ComponentId, instance_id: InstanceId) -> Self {
        Self {
            component_id,
            instance_id,
        }
    }
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_component_id(value: &str) -> Result<(), ComponentError> {
    if value.trim() != value || value.is_empty() {
        return Err(ComponentError::InvalidComponentId(value.to_owned()));
    }
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Ok(());
    }
    Err(ComponentError::InvalidComponentId(value.to_owned()))
}
