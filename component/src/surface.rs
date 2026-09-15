use std::fmt;
use std::sync::Arc;

use fabric_core::{ContractId, ContractKey};

use crate::{Component, ComponentError};

const COMPONENT_SURFACE_CONTRACT_ID: &str = "fabric.component.surface";

pub fn surface_contract_id() -> ContractId {
    ContractId::new(COMPONENT_SURFACE_CONTRACT_ID).expect("static component surface contract id")
}

pub fn surface_contract_key() -> ContractKey<SurfaceRegistry> {
    ContractKey::provisional(surface_contract_id())
}

pub trait SurfaceRegistryService: Send + Sync {
    fn register(&self, owner: Component, surface_id: SurfaceId) -> Result<Surface, ComponentError>;

    fn surface(&self, surface_id: &SurfaceId) -> Result<Surface, ComponentError>;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceId(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Surface {
    owner: Component,
    surface_id: SurfaceId,
}

#[derive(Clone)]
pub struct SurfaceRegistry {
    inner: Arc<dyn SurfaceRegistryService>,
}

impl SurfaceId {
    pub fn new(value: impl Into<String>) -> Result<Self, ComponentError> {
        let value = value.into();
        validate_component_surface_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Surface {
    pub fn new(owner: Component, surface_id: SurfaceId) -> Self {
        Self { owner, surface_id }
    }

    pub fn owner(&self) -> &Component {
        &self.owner
    }

    pub fn surface_id(&self) -> &SurfaceId {
        &self.surface_id
    }
}

impl SurfaceRegistry {
    pub fn new(inner: Arc<dyn SurfaceRegistryService>) -> Self {
        Self { inner }
    }

    pub fn register(
        &self,
        owner: Component,
        surface_id: SurfaceId,
    ) -> Result<Surface, ComponentError> {
        self.inner.register(owner, surface_id)
    }

    pub fn surface(&self, surface_id: &SurfaceId) -> Result<Surface, ComponentError> {
        self.inner.surface(surface_id)
    }
}

impl fmt::Display for SurfaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_component_surface_id(value: &str) -> Result<(), ComponentError> {
    if value.trim() != value || value.is_empty() {
        return Err(ComponentError::InvalidSurfaceId(value.to_owned()));
    }
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Ok(());
    }
    Err(ComponentError::InvalidSurfaceId(value.to_owned()))
}
