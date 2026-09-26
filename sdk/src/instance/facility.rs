use std::error::Error;
use std::fmt;

use fabric_core::{CompositionId, InstanceGeneration, InstanceId, LifecycleState};
use fabric_host::HostDescriptor;

use crate::materialization::{MaterializationPlanProvenance, MaterializationProfile};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceFacilityName(String);

impl InstanceFacilityName {
    pub fn new(value: impl Into<String>) -> Result<Self, InstanceFacilityError> {
        let value = value.into();
        if value.trim() != value || value.is_empty() {
            return Err(InstanceFacilityError::InvalidName { value });
        }
        if value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            Ok(Self(value))
        } else {
            Err(InstanceFacilityError::InvalidName { value })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InstanceFacilityName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceFacilityContext {
    composition_id: CompositionId,
    instance_id: InstanceId,
    generation: InstanceGeneration,
    materialization_plan: MaterializationPlanProvenance,
    lifecycle: LifecycleState,
}

impl InstanceFacilityContext {
    pub(crate) fn new(
        composition_id: CompositionId,
        instance_id: InstanceId,
        generation: InstanceGeneration,
        materialization_plan: MaterializationPlanProvenance,
        lifecycle: LifecycleState,
    ) -> Self {
        Self {
            composition_id,
            instance_id,
            generation,
            materialization_plan,
            lifecycle,
        }
    }

    pub fn composition_id(&self) -> &CompositionId {
        &self.composition_id
    }

    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }

    pub fn generation(&self) -> InstanceGeneration {
        self.generation
    }

    pub fn materialization_plan(&self) -> &MaterializationPlanProvenance {
        &self.materialization_plan
    }

    pub fn materialization_profile(&self) -> &MaterializationProfile {
        self.materialization_plan.materialization_profile()
    }

    pub fn host(&self) -> Option<&HostDescriptor> {
        self.materialization_plan.host()
    }

    pub fn lifecycle(&self) -> LifecycleState {
        self.lifecycle
    }
}

/// Live-only operational machinery attached to one Fabric Instance generation.
///
/// Facilities observe bounded Instance lifecycle transitions. They do not
/// participate in Composition semantics and cannot veto Core lifecycle truth.
pub trait InstanceFacility: Send {
    fn name(&self) -> &InstanceFacilityName;

    fn attached(&mut self, _context: &InstanceFacilityContext) {}

    fn starting(&mut self, _context: &InstanceFacilityContext) {}

    fn started(&mut self, _context: &InstanceFacilityContext) {}

    fn stopping(&mut self, _context: &InstanceFacilityContext) {}

    fn stopped(&mut self, _context: &InstanceFacilityContext) {}

    fn detached(&mut self, _context: &InstanceFacilityContext) {}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceFacilityObservation {
    name: InstanceFacilityName,
}

impl InstanceFacilityObservation {
    pub(crate) fn new(name: InstanceFacilityName) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &InstanceFacilityName {
        &self.name
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum InstanceFacilityError {
    InvalidName { value: String },
    DuplicateFacility { name: InstanceFacilityName },
    FacilityNotFound { name: InstanceFacilityName },
    InstanceStopped { name: InstanceFacilityName },
}

impl fmt::Display for InstanceFacilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { value } => write!(f, "instance facility name `{value}` is invalid"),
            Self::DuplicateFacility { name } => {
                write!(f, "instance facility `{name}` is already attached")
            }
            Self::FacilityNotFound { name } => {
                write!(f, "instance facility `{name}` is not attached")
            }
            Self::InstanceStopped { name } => {
                write!(
                    f,
                    "instance facility `{name}` cannot be attached to a stopped Instance"
                )
            }
        }
    }
}

impl Error for InstanceFacilityError {}
