use super::*;

impl ComponentCommunicationService for SharedComponentState {
    fn bind_requirement(
        &self,
        requirement: &ResolvedComponentRequirement,
    ) -> Result<(), ComponentError> {
        <Self as ComponentRequirementService>::register(self, requirement.clone())
    }

    fn validate_call(
        &self,
        requirement: &ResolvedComponentRequirement,
        caller: &ComponentParticipation,
        context: Option<&crate::InvocationContext>,
    ) -> Result<(), ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        let runtime_status = state.current_status();
        let runtime_instance_id = runtime_status.instance_id();
        if let Some(context) = context {
            validate_context_runtime(&state, context)?;
        }
        if requirement.provider().instance_id() != runtime_instance_id {
            return Err(ComponentError::ComponentContractProviderInstanceMismatch {
                component_id: requirement.provider().component_id().clone(),
                component_instance_id: requirement.provider().instance_id().clone(),
                instance_id: runtime_instance_id.clone(),
            });
        }
        if caller.component().instance_id() != runtime_instance_id {
            return Err(ComponentError::ComponentContractCallerInstanceMismatch {
                component_id: caller.component().component_id().clone(),
                component_instance_id: caller.component().instance_id().clone(),
                instance_id: runtime_instance_id.clone(),
            });
        }
        if caller.component() != requirement.consumer() {
            return Err(ComponentError::ComponentContractConsumerMismatch {
                expected_component_id: requirement.consumer().component_id().clone(),
                actual_component_id: caller.component().component_id().clone(),
            });
        }
        if state
            .components
            .get(requirement.provider().component_id())
            .is_none_or(|status| status.component() != requirement.provider())
        {
            return Err(ComponentError::ComponentContractProviderNotParticipating(
                requirement.provider().component_id().clone(),
            ));
        }
        let provider_availability = effective_availability_from_health(effective_health(
            &state,
            requirement.provider(),
            &mut std::collections::BTreeSet::new(),
        ));
        if !provider_availability.is_available() {
            return Err(ComponentError::ComponentUnavailable(
                requirement.provider().component_id().clone(),
            ));
        }
        if !current_participation(&state, caller) {
            return Err(ComponentError::ComponentContractCallerNotParticipating(
                caller.component().component_id().clone(),
            ));
        }
        if !current_active_participation(&state, caller)
            || !participation_is_available(&state, caller)
        {
            return Err(ComponentError::ComponentUnavailable(
                caller.component().component_id().clone(),
            ));
        }
        Ok(())
    }
}
