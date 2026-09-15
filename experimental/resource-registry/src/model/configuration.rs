use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::ResourceRegistryError;
use crate::model::key::validate_resource_registry_key;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceConfigurationKind(String);

#[derive(Clone)]
pub struct ResourceConfiguration {
    kind: ResourceConfigurationKind,
    value: Arc<dyn Any + Send + Sync>,
}

type ConfigConsumer =
    dyn Fn(&ResourceConfiguration) -> Result<(), ResourceRegistryError> + Send + Sync;

#[derive(Clone)]
pub struct ResourceConfigurationFacet {
    kind: ResourceConfigurationKind,
    consumer: Arc<ConfigConsumer>,
}

impl ResourceConfigurationKind {
    pub fn new(value: impl Into<String>) -> Result<Self, ResourceRegistryError> {
        let value = value.into();
        validate_resource_registry_key(&value, "resource configuration kind")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ResourceConfiguration {
    pub fn new<T>(kind: ResourceConfigurationKind, value: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self {
            kind,
            value: Arc::new(value),
        }
    }

    pub fn kind(&self) -> &ResourceConfigurationKind {
        &self.kind
    }

    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.value.as_ref().downcast_ref::<T>()
    }
}

impl fmt::Debug for ResourceConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceConfiguration")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl ResourceConfigurationFacet {
    pub fn typed_with<T>(
        kind: ResourceConfigurationKind,
        consume: impl Fn(&T) -> Result<(), ResourceRegistryError> + Send + Sync + 'static,
    ) -> Self
    where
        T: Send + Sync + 'static,
    {
        let boundary_kind = kind.clone();
        Self {
            kind,
            consumer: Arc::new(move |configuration| {
                if configuration.kind() != &boundary_kind {
                    return Err(ResourceRegistryError::ConfigurationRejected {
                        message: format!(
                            "configuration kind `{}` does not match declared envelope `{}`",
                            configuration.kind().as_str(),
                            boundary_kind.as_str()
                        ),
                    });
                }
                let typed = configuration.downcast_ref::<T>().ok_or_else(|| {
                    ResourceRegistryError::ConfigurationRejected {
                        message: format!(
                            "configuration `{}` did not satisfy the declared resource-owned envelope",
                            boundary_kind.as_str()
                        ),
                    }
                })?;
                consume(typed)
            }),
        }
    }

    pub fn kind(&self) -> &ResourceConfigurationKind {
        &self.kind
    }

    pub fn consume(
        &self,
        configuration: &ResourceConfiguration,
    ) -> Result<(), ResourceRegistryError> {
        (self.consumer)(configuration)
    }
}
