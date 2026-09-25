use super::*;

impl ComponentRequirementService for SharedComponentState {
    fn register(&self, requirement: ResolvedComponentRequirement) -> Result<(), ComponentError> {
        let mut state = self.inner.lock().expect("component runtime state lock");
        let instance = state.instance_id();
        if requirement.consumer().instance_id() != &instance
            || requirement.provider().instance_id() != &instance
        {
            return Err(ComponentError::ComponentRegistryInstanceMismatch {
                component_id: requirement.consumer().component_id().clone(),
                component_instance_id: requirement.consumer().instance_id().clone(),
                instance_id: instance,
            });
        }
        if !state.requirements.contains(&requirement) {
            state.requirements.push(requirement);
            refresh_operational_status(&mut state);
        }
        Ok(())
    }

    fn requirements(&self, consumer: &crate::ComponentId) -> Vec<ResolvedComponentRequirement> {
        self.inner
            .lock()
            .expect("component runtime state lock")
            .requirements
            .iter()
            .filter(|requirement| requirement.consumer().component_id() == consumer)
            .cloned()
            .collect()
    }

    fn effective_availability(
        &self,
        component_id: &crate::ComponentId,
    ) -> Result<ComponentEffectiveAvailability, ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        let component = bound_component(&state, component_id);
        Ok(effective_availability_from_health(effective_health(
            &state,
            &component,
            &mut std::collections::BTreeSet::new(),
        )))
    }

    fn effective_health(
        &self,
        component_id: &crate::ComponentId,
    ) -> Result<ComponentEffectiveHealth, ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        let component = bound_component(&state, component_id);
        Ok(effective_health(
            &state,
            &component,
            &mut std::collections::BTreeSet::new(),
        ))
    }
}
