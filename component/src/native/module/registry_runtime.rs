use fabric_core::Health;

use super::*;

impl ComponentRegistryService for SharedComponentState {
    fn register(
        &self,
        component: Component,
        health: Health,
    ) -> Result<ComponentStatus, ComponentError> {
        let mut state = self.inner.lock().expect("component runtime state lock");
        if component.instance_id() != state.current_status().instance_id() {
            return Err(ComponentError::ComponentRegistryInstanceMismatch {
                component_id: component.component_id().clone(),
                component_instance_id: component.instance_id().clone(),
                instance_id: state.instance_id(),
            });
        }
        if !state
            .component_declarations
            .contains_key(component.component_id())
        {
            return Err(ComponentError::UnknownComponent(
                component.component_id().clone(),
            ));
        }
        if state.components.contains_key(component.component_id()) {
            return Err(ComponentError::DuplicateComponentId(
                component.component_id().clone(),
            ));
        }
        let participation = ComponentParticipation::new(
            component.clone(),
            state.current_generation()?,
            ComponentParticipationId::new(state.next_participation_id),
        );
        state.next_participation_id += 1;
        let status = ComponentStatus::new(participation, ParticipationState::Preparing, health);
        state
            .components
            .insert(component.component_id().clone(), status.clone());
        refresh_operational_status(&mut state);
        Ok(status)
    }

    fn update_health(
        &self,
        participation: &ComponentParticipation,
        health: Health,
    ) -> Result<ComponentStatus, ComponentError> {
        let mut state = self.inner.lock().expect("component runtime state lock");
        if participation.component().instance_id() != state.current_status().instance_id() {
            return Err(ComponentError::ComponentRegistryInstanceMismatch {
                component_id: participation.component().component_id().clone(),
                component_instance_id: participation.component().instance_id().clone(),
                instance_id: state.instance_id(),
            });
        }
        let status = state
            .components
            .get_mut(participation.component().component_id())
            .ok_or_else(|| {
                ComponentError::UnknownComponent(participation.component().component_id().clone())
            })?;
        if status.participation() != participation {
            return Err(ComponentError::StaleComponentParticipation(
                participation.clone(),
            ));
        }
        *status = status.with_health(health);
        let updated = status.clone();
        refresh_operational_status(&mut state);
        Ok(updated)
    }

    fn unregister(
        &self,
        participation: &ComponentParticipation,
    ) -> Result<ComponentStatus, ComponentError> {
        self.unregister_with_teardown(participation)
    }

    fn activate(
        &self,
        participation: &ComponentParticipation,
    ) -> Result<ComponentStatus, ComponentError> {
        let mut state = self.inner.lock().expect("component runtime state lock");
        if participation.component().instance_id() != state.current_status().instance_id() {
            return Err(ComponentError::ComponentRegistryInstanceMismatch {
                component_id: participation.component().component_id().clone(),
                component_instance_id: participation.component().instance_id().clone(),
                instance_id: state.instance_id(),
            });
        }
        let status = state
            .components
            .get_mut(participation.component().component_id())
            .ok_or_else(|| {
                ComponentError::UnknownComponent(participation.component().component_id().clone())
            })?;
        if status.participation() != participation {
            return Err(ComponentError::StaleComponentParticipation(
                participation.clone(),
            ));
        }
        match status.state() {
            ParticipationState::Preparing => {
                *status = status.with_state(ParticipationState::Active);
            }
            ParticipationState::Active => {
                return Err(ComponentError::ComponentParticipationAlreadyActive(
                    participation.clone(),
                ));
            }
        }
        let activated = status.clone();
        refresh_operational_status(&mut state);
        Ok(activated)
    }

    fn component(
        &self,
        component_id: &crate::ComponentId,
    ) -> Result<ComponentStatus, ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        state
            .components
            .get(component_id)
            .cloned()
            .ok_or_else(|| ComponentError::UnknownComponent(component_id.clone()))
    }

    fn components(&self) -> Vec<ComponentStatus> {
        self.inner
            .lock()
            .expect("component runtime state lock")
            .components
            .values()
            .cloned()
            .collect()
    }
}
