use fabric_core::{Health, InstanceGeneration, InstanceId};

use crate::ComponentError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Component-runtime lifecycle, deliberately distinct from Core Instance lifecycle.
pub enum ComponentRuntimeLifecycle {
    Starting,
    Ready,
    Stopping,
    Stopped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentRuntimeStatus {
    instance_id: InstanceId,
    generation: Option<InstanceGeneration>,
    lifecycle: ComponentRuntimeLifecycle,
    health: Health,
}

impl ComponentRuntimeLifecycle {
    pub fn transition_to(self, next: Self) -> Result<Self, ComponentError> {
        let valid = matches!(
            (self, next),
            (Self::Stopped, Self::Starting)
                | (Self::Starting, Self::Ready)
                | (Self::Starting, Self::Stopping)
                | (Self::Ready, Self::Stopping)
                | (Self::Stopping, Self::Stopped)
        );
        if valid {
            Ok(next)
        } else {
            Err(ComponentError::InvalidComponentRuntimeLifecycleTransition {
                from: self,
                to: next,
            })
        }
    }
}

impl ComponentRuntimeStatus {
    pub fn new(
        instance_id: InstanceId,
        generation: Option<InstanceGeneration>,
        lifecycle: ComponentRuntimeLifecycle,
        health: Health,
    ) -> Self {
        Self {
            instance_id,
            generation,
            lifecycle,
            health,
        }
    }

    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }

    pub fn lifecycle(&self) -> ComponentRuntimeLifecycle {
        self.lifecycle
    }

    pub fn generation(&self) -> Option<InstanceGeneration> {
        self.generation
    }

    pub fn health(&self) -> Health {
        self.health
    }

    pub fn transition_to(&self, next: ComponentRuntimeLifecycle) -> Result<Self, ComponentError> {
        Ok(Self {
            instance_id: self.instance_id.clone(),
            generation: self.generation,
            lifecycle: self.lifecycle.transition_to(next)?,
            health: self.health,
        })
    }

    pub fn with_health(&self, health: Health) -> Self {
        Self {
            instance_id: self.instance_id.clone(),
            generation: self.generation,
            lifecycle: self.lifecycle,
            health,
        }
    }

    pub fn bind_generation(&self, generation: InstanceGeneration) -> Self {
        Self {
            instance_id: self.instance_id.clone(),
            generation: Some(generation),
            lifecycle: self.lifecycle,
            health: self.health,
        }
    }

    pub fn bind_instance(&self, instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            generation: self.generation,
            lifecycle: self.lifecycle,
            health: self.health,
        }
    }

    pub fn bind_runtime(&self, instance_id: InstanceId, generation: InstanceGeneration) -> Self {
        Self {
            instance_id,
            generation: Some(generation),
            lifecycle: self.lifecycle,
            health: self.health,
        }
    }
}
