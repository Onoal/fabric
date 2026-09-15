use crate::{
    ResourceError, ResourceId, ResourceName, ResourceSchemaCompatibilityRequirement,
    ResourceSchemaDescriptor, ResourceSchemaIdentity, ResourceSchemaRequirement,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceRequirement {
    resource: ResourceId,
    pub name: ResourceName,
    compatibility: ResourceSchemaCompatibilityRequirement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterResourceSchemaSupport {
    resource: ResourceId,
    supported: ResourceSchemaCompatibilityRequirement,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ResourceCompatibilityError {
    #[error("resource compatibility expected `{expected}` but {role} named `{actual}`")]
    ResourceIdentityMismatch {
        expected: ResourceId,
        actual: ResourceId,
        role: ResourceCompatibilityRole,
    },

    #[error(
        "resource requirement `{resource}` with compatibility `{required}` does not accept schema `{provided}`"
    )]
    ResourceRequirementIncompatible {
        resource: ResourceId,
        required: ResourceSchemaCompatibilityRequirement,
        provided: ResourceSchemaIdentity,
    },

    #[error(
        "adapter support for `{resource}` with compatibility `{supported}` does not accept schema `{provided}`"
    )]
    AdapterSupportIncompatible {
        resource: ResourceId,
        supported: ResourceSchemaCompatibilityRequirement,
        provided: ResourceSchemaIdentity,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceCompatibilityRole {
    Schema,
    AdapterSupport,
}

impl std::fmt::Display for ResourceCompatibilityRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Schema => f.write_str("schema"),
            Self::AdapterSupport => f.write_str("adapter support"),
        }
    }
}

impl ResourceRequirement {
    pub fn new(resource: ResourceId, name: impl Into<String>) -> Result<Self, ResourceError> {
        Self::provisional(resource, name)
    }

    pub fn provisional(
        resource: ResourceId,
        name: impl Into<String>,
    ) -> Result<Self, ResourceError> {
        Ok(Self {
            resource,
            name: ResourceName::new(name)?,
            compatibility: ResourceSchemaCompatibilityRequirement::provisional(),
        })
    }

    pub fn versioned(
        resource: ResourceId,
        name: impl Into<String>,
        requirement: ResourceSchemaRequirement,
    ) -> Result<Self, ResourceError> {
        Ok(Self {
            resource,
            name: ResourceName::new(name)?,
            compatibility: ResourceSchemaCompatibilityRequirement::versioned(requirement),
        })
    }

    pub fn resource(&self) -> &ResourceId {
        &self.resource
    }

    pub fn compatibility(&self) -> &ResourceSchemaCompatibilityRequirement {
        &self.compatibility
    }

    pub fn accepts_schema(&self, schema: &ResourceSchemaDescriptor) -> bool {
        self.resource == *schema.resource() && self.compatibility.accepts(schema.identity())
    }

    pub fn evaluate_compatibility(
        &self,
        schema: &ResourceSchemaDescriptor,
        adapter_support: &AdapterResourceSchemaSupport,
    ) -> Result<(), ResourceCompatibilityError> {
        if self.resource != *schema.resource() {
            return Err(ResourceCompatibilityError::ResourceIdentityMismatch {
                expected: self.resource.clone(),
                actual: schema.resource().clone(),
                role: ResourceCompatibilityRole::Schema,
            });
        }
        if self.resource != *adapter_support.resource() {
            return Err(ResourceCompatibilityError::ResourceIdentityMismatch {
                expected: self.resource.clone(),
                actual: adapter_support.resource().clone(),
                role: ResourceCompatibilityRole::AdapterSupport,
            });
        }
        if !self.accepts_schema(schema) {
            return Err(
                ResourceCompatibilityError::ResourceRequirementIncompatible {
                    resource: self.resource.clone(),
                    required: self.compatibility.clone(),
                    provided: schema.identity().clone(),
                },
            );
        }
        adapter_support.accepts_schema(schema)
    }
}

impl AdapterResourceSchemaSupport {
    pub fn provisional(resource: ResourceId) -> Self {
        Self {
            resource,
            supported: ResourceSchemaCompatibilityRequirement::provisional(),
        }
    }

    pub fn versioned(resource: ResourceId, requirement: ResourceSchemaRequirement) -> Self {
        Self {
            resource,
            supported: ResourceSchemaCompatibilityRequirement::versioned(requirement),
        }
    }

    pub fn resource(&self) -> &ResourceId {
        &self.resource
    }

    pub fn compatibility(&self) -> &ResourceSchemaCompatibilityRequirement {
        &self.supported
    }

    pub fn accepts_schema(
        &self,
        schema: &ResourceSchemaDescriptor,
    ) -> Result<(), ResourceCompatibilityError> {
        if self.resource != *schema.resource() {
            return Err(ResourceCompatibilityError::ResourceIdentityMismatch {
                expected: self.resource.clone(),
                actual: schema.resource().clone(),
                role: ResourceCompatibilityRole::Schema,
            });
        }
        if self.supported.accepts(schema.identity()) {
            Ok(())
        } else {
            Err(ResourceCompatibilityError::AdapterSupportIncompatible {
                resource: self.resource.clone(),
                supported: self.supported.clone(),
                provided: schema.identity().clone(),
            })
        }
    }
}
