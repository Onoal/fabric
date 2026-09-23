use fabric_core::Health;

use super::*;

impl crate::InvocationService for SharedComponentState {
    fn begin_external(&self) -> Result<crate::InvocationContext, ComponentError> {
        let mut state = self.inner.lock().expect("component runtime state lock");
        match state.current_status().lifecycle() {
            ComponentHostLifecycle::Ready => {}
            ComponentHostLifecycle::Starting
            | ComponentHostLifecycle::Stopping
            | ComponentHostLifecycle::Stopped => return Err(ComponentError::Unavailable),
        }
        let invocation_id = crate::InvocationId::new(state.next_invocation_id);
        state.next_invocation_id += 1;
        Ok(crate::InvocationContext::new(
            state.instance_id(),
            state.current_generation()?,
            invocation_id,
            crate::InvocationOrigin::External,
        ))
    }

    fn begin_component(
        &self,
        participation: ComponentParticipation,
    ) -> Result<crate::InvocationContext, ComponentError> {
        let mut state = self.inner.lock().expect("component runtime state lock");
        let instance_id = state.instance_id();
        if participation.component().instance_id() != &instance_id {
            return Err(ComponentError::InvocationContextInstanceMismatch {
                context_instance_id: participation.component().instance_id().clone(),
                instance_id: instance_id.clone(),
            });
        }
        if !current_participation(&state, &participation) {
            return Err(ComponentError::InvocationOriginNotParticipating(
                participation.component().component_id().clone(),
            ));
        }
        if !current_active_participation(&state, &participation)
            || !participation_is_available(&state, &participation)
        {
            return Err(ComponentError::ComponentUnavailable(
                participation.component().component_id().clone(),
            ));
        }
        let invocation_id = crate::InvocationId::new(state.next_invocation_id);
        state.next_invocation_id += 1;
        Ok(crate::InvocationContext::new(
            instance_id,
            state.current_generation()?,
            invocation_id,
            crate::InvocationOrigin::ComponentInstanceBinding(participation.component().clone()),
        ))
    }

    fn validate_context(&self, context: &crate::InvocationContext) -> Result<(), ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        validate_context_runtime(&state, context)
    }
}

impl ComponentScopeService for SharedComponentState {
    fn begin_invocation(
        &self,
        participation: &ComponentParticipation,
    ) -> Result<crate::InvocationContext, ComponentError> {
        <Self as crate::InvocationService>::begin_component(self, participation.clone())
    }

    fn continue_invocation(
        &self,
        participation: &ComponentParticipation,
        context: &crate::InvocationContext,
    ) -> Result<(), ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        if participation.component().instance_id() != state.current_status().instance_id() {
            return Err(ComponentError::InvocationContextInstanceMismatch {
                context_instance_id: participation.component().instance_id().clone(),
                instance_id: state.instance_id(),
            });
        }
        validate_context_runtime(&state, context)?;
        if !current_participation(&state, participation) {
            return Err(ComponentError::InvocationOriginNotParticipating(
                participation.component().component_id().clone(),
            ));
        }
        if !current_active_participation(&state, participation)
            || !participation_is_available(&state, participation)
        {
            return Err(ComponentError::ComponentUnavailable(
                participation.component().component_id().clone(),
            ));
        }
        Ok(())
    }

    fn update_health(
        &self,
        participation: &ComponentParticipation,
        health: Health,
    ) -> Result<ComponentStatus, ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        if participation.component().instance_id() != state.current_status().instance_id() {
            return Err(ComponentError::ComponentRegistryInstanceMismatch {
                component_id: participation.component().component_id().clone(),
                component_instance_id: participation.component().instance_id().clone(),
                instance_id: state.instance_id(),
            });
        }
        let Some(status) = state
            .components
            .get(participation.component().component_id())
        else {
            return Err(ComponentError::UnknownComponent(
                participation.component().component_id().clone(),
            ));
        };
        if status.participation() != participation {
            return Err(ComponentError::StaleComponentParticipation(
                participation.clone(),
            ));
        }
        if status.state() != ParticipationState::Active {
            return Err(ComponentError::ComponentParticipationNotActive(
                participation.clone(),
            ));
        }
        drop(state);
        <Self as ComponentRegistryService>::update_health(self, participation, health)
    }
}
