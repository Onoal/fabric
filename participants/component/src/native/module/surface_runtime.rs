use super::*;

impl SurfaceRegistryService for SharedComponentState {
    fn register(
        &self,
        owner: ComponentInstanceBinding,
        surface_id: SurfaceId,
    ) -> Result<Surface, ComponentError> {
        let mut state = self.inner.lock().expect("component runtime state lock");
        if owner.instance_id() != state.current_status().instance_id() {
            return Err(ComponentError::SurfaceOwnerInstanceMismatch {
                surface_id,
                owner_instance_id: owner.instance_id().clone(),
                instance_id: state.instance_id(),
            });
        }
        if state.surfaces.contains_key(&surface_id) {
            return Err(ComponentError::DuplicateSurfaceId(surface_id));
        }
        let surface = Surface::new(owner, surface_id.clone());
        state.surfaces.insert(surface_id, surface.clone());
        Ok(surface)
    }

    fn surface(&self, surface_id: &SurfaceId) -> Result<Surface, ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        state
            .surfaces
            .get(surface_id)
            .cloned()
            .ok_or_else(|| ComponentError::UnknownSurface(surface_id.clone()))
    }
}
