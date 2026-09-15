use crate::ResourceError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceBoundaryId(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceScope(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceName(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceContext {
    boundary: ResourceBoundaryId,
    path: Vec<ResourceScope>,
}

impl ResourceBoundaryId {
    pub fn new(value: impl Into<String>) -> Result<Self, ResourceError> {
        let value = value.into();
        validate_boundary_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ResourceScope {
    pub fn new(value: impl Into<String>) -> Result<Self, ResourceError> {
        let value = value.into();
        validate_scope(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ResourceName {
    pub fn new(value: impl Into<String>) -> Result<Self, ResourceError> {
        let value = value.into();
        validate_name(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ResourceContext {
    pub fn root(boundary: ResourceBoundaryId) -> Self {
        Self {
            boundary,
            path: Vec::new(),
        }
    }

    pub fn child(&self, scope: ResourceScope) -> Self {
        let mut path = self.path.clone();
        path.push(scope);
        Self {
            boundary: self.boundary.clone(),
            path,
        }
    }

    pub fn boundary(&self) -> &ResourceBoundaryId {
        &self.boundary
    }

    pub fn path(&self) -> &[ResourceScope] {
        &self.path
    }
}

fn validate_boundary_id(value: &str) -> Result<(), ResourceError> {
    if value.is_empty()
        || value.len() > 256
        || matches!(value, "." | "..")
        || value.bytes().any(|byte| {
            byte.is_ascii_whitespace() || byte.is_ascii_control() || matches!(byte, b'/' | b'\\')
        })
    {
        return Err(ResourceError::InvalidInput {
            message: "invalid resource boundary id".to_owned(),
        });
    }
    Ok(())
}

fn validate_scope(value: &str) -> Result<(), ResourceError> {
    if value.is_empty()
        || value.len() > 256
        || matches!(value, "." | "..")
        || value.bytes().any(|byte| {
            byte.is_ascii_whitespace()
                || byte.is_ascii_control()
                || matches!(byte, b'/' | b'\\' | b':')
        })
    {
        return Err(ResourceError::InvalidInput {
            message: "invalid resource scope".to_owned(),
        });
    }
    Ok(())
}

fn validate_name(value: &str) -> Result<(), ResourceError> {
    if value.is_empty()
        || value.len() > 128
        || value.bytes().any(|byte| {
            byte.is_ascii_whitespace()
                || byte.is_ascii_control()
                || matches!(byte, b'/' | b'\\' | b':')
        })
    {
        return Err(ResourceError::InvalidInput {
            message: "invalid resource name".to_owned(),
        });
    }
    Ok(())
}
