use std::sync::Arc;

use crate::ResourceRegistryError;
use crate::model::key::validate_resource_registry_key;

type InspectionProvider =
    dyn Fn() -> Result<ResourceInspection, ResourceRegistryError> + Send + Sync;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceInspectionValue {
    Public(String),
    Redacted,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceInspectionEntry {
    key: String,
    value: ResourceInspectionValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ResourceInspection {
    entries: Vec<ResourceInspectionEntry>,
}

#[derive(Clone)]
pub struct ResourceInspectionFacet {
    inspector: Arc<InspectionProvider>,
}

impl ResourceInspectionEntry {
    pub fn public(
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, ResourceRegistryError> {
        let key = key.into();
        validate_resource_registry_key(&key, "resource inspection key")?;
        Ok(Self {
            key,
            value: ResourceInspectionValue::Public(value.into()),
        })
    }

    pub fn redacted(key: impl Into<String>) -> Result<Self, ResourceRegistryError> {
        let key = key.into();
        validate_resource_registry_key(&key, "resource inspection key")?;
        Ok(Self {
            key,
            value: ResourceInspectionValue::Redacted,
        })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &ResourceInspectionValue {
        &self.value
    }

    pub fn public_value(&self) -> Option<&str> {
        match &self.value {
            ResourceInspectionValue::Public(value) => Some(value),
            ResourceInspectionValue::Redacted => None,
        }
    }

    pub fn is_redacted(&self) -> bool {
        matches!(self.value, ResourceInspectionValue::Redacted)
    }
}

impl ResourceInspection {
    pub fn new(mut entries: Vec<ResourceInspectionEntry>) -> Result<Self, ResourceRegistryError> {
        entries.sort();
        for pair in entries.windows(2) {
            if pair[0].key == pair[1].key {
                return Err(ResourceRegistryError::invalid_input(format!(
                    "duplicate resource inspection key `{}`",
                    pair[0].key
                )));
            }
        }
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[ResourceInspectionEntry] {
        &self.entries
    }
}

impl ResourceInspectionFacet {
    pub fn new(
        inspector: impl Fn() -> Result<ResourceInspection, ResourceRegistryError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            inspector: Arc::new(inspector),
        }
    }

    pub fn inspect(&self) -> Result<ResourceInspection, ResourceRegistryError> {
        (self.inspector)()
    }
}
