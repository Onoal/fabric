use super::*;

impl ComponentControlService for SharedComponentState {
    fn set_desired(
        &self,
        component: Component,
        desired: ComponentDesiredState,
    ) -> Result<ComponentControl, ComponentError> {
        let mut state = self.inner.lock().expect("component runtime state lock");
        if component.instance_id() != state.current_status().instance_id() {
            return Err(ComponentError::ComponentControlInstanceMismatch {
                component_id: component.component_id().clone(),
                component_instance_id: component.instance_id().clone(),
                instance_id: state.instance_id(),
            });
        }
        let control = match state.component_controls.get(component.component_id()) {
            Some(existing) => existing.with_desired(desired),
            None => ComponentControl::new(component.clone(), desired),
        };
        state
            .component_controls
            .insert(component.component_id().clone(), control.clone());
        Ok(control)
    }

    fn control(
        &self,
        component_id: &crate::ComponentId,
    ) -> Result<ComponentControl, ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        state
            .component_controls
            .get(component_id)
            .cloned()
            .ok_or_else(|| ComponentError::UnknownComponentControl(component_id.clone()))
    }

    fn controls(&self) -> Vec<ComponentControl> {
        self.inner
            .lock()
            .expect("component runtime state lock")
            .component_controls
            .values()
            .cloned()
            .collect()
    }

    fn snapshot(&self) -> ComponentControlSnapshot {
        let state = self.inner.lock().expect("component runtime state lock");
        ComponentControlSnapshot::from_controls(
            state.instance_id(),
            state.component_controls.values().cloned(),
        )
    }
}
