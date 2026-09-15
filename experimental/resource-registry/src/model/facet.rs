use std::fmt;

use crate::ResourceRegistryError;
use crate::model::key::validate_resource_registry_key;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceFacetId(String);

impl ResourceFacetId {
    pub fn new(value: impl Into<String>) -> Result<Self, ResourceRegistryError> {
        let value = value.into();
        validate_resource_registry_key(&value, "resource facet id")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceFacetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub fn resource_configuration_facet_id() -> ResourceFacetId {
    ResourceFacetId::new("configuration").expect("static configuration facet id")
}

pub fn resource_inspection_facet_id() -> ResourceFacetId {
    ResourceFacetId::new("inspection").expect("static inspection facet id")
}
