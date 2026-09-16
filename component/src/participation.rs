use std::fmt;

use fabric_core::InstanceGeneration;

use crate::Component;

/// Runtime-local identity for one active component participation incarnation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentParticipationId(u64);

/// Authority granted by one current registration of a Component.
///
/// This is deliberately distinct from stable component membership. A new
/// registration receives a new handle, so a departed runtime cannot regain
/// authority when the same component later rejoins.
#[derive(Clone, Debug, PartialEq, Eq)]
/// Generation-scoped authority for one active Component runtime participation.
///
/// A participation is a runtime occurrence, not the Component's semantic
/// definition or a Core-wide lifecycle identity.
pub struct ComponentParticipation {
    component: Component,
    generation: InstanceGeneration,
    participation_id: ComponentParticipationId,
}

impl ComponentParticipationId {
    pub(crate) fn new(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for ComponentParticipationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ComponentParticipation {
    pub(crate) fn new(
        component: Component,
        generation: InstanceGeneration,
        participation_id: ComponentParticipationId,
    ) -> Self {
        Self {
            component,
            generation,
            participation_id,
        }
    }

    pub fn component(&self) -> &Component {
        &self.component
    }

    pub fn generation(&self) -> InstanceGeneration {
        self.generation
    }

    pub fn participation_id(&self) -> ComponentParticipationId {
        self.participation_id
    }

    pub fn with_component(&self, component: Component) -> Self {
        Self {
            component,
            generation: self.generation,
            participation_id: self.participation_id,
        }
    }
}
