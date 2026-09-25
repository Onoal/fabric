use fabric_component::{
    ComponentError, ComponentHostHandle, ComponentId, ComponentStatus, OperationFuture,
    OperationKey,
};
use fabric_core::{
    CompositionError, CompositionId, Instance as CoreInstance, InstanceGeneration, InstanceId,
    LifecycleState,
};
use fabric_host::HostDescriptor;
use std::sync::Arc;

use super::fabric::FabricManifest;
use super::{ComponentDefinition, Composition};
use crate::ids::IntoInstanceId;

/// One high-level live materialization of a semantic Fabric [`Composition`].
///
/// `Instance` owns generation-scoped live Core runtime state while retaining
/// the immutable semantic Composition context it was materialized from.
pub struct Instance {
    core: CoreInstance,
    #[allow(dead_code)]
    semantic_context: Arc<FabricManifest>,
    components: Option<InstanceComponents>,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("composition_id", self.composition_id())
            .field("instance_id", self.instance_id())
            .field("generation", &self.generation())
            .field("lifecycle", &self.lifecycle())
            .finish()
    }
}

#[derive(Clone)]
pub struct InstanceComponents {
    handle: std::sync::Arc<ComponentHostHandle>,
}

impl Composition {
    pub fn materialize<I>(&self, instance_id: I) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        self.materialize_with_host(instance_id, None)
    }

    pub fn materialize_on<I>(
        &self,
        instance_id: I,
        host: &HostDescriptor,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        self.materialize_with_host(instance_id, Some(host))
    }

    fn materialize_with_host<I>(
        &self,
        instance_id: I,
        host: Option<&HostDescriptor>,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        let core = match host {
            Some(host) => self
                .core()
                .materialize_on(instance_id.into_instance_id()?, host)?,
            None => self.core().materialize(instance_id.into_instance_id()?)?,
        };
        let components = self
            .component_host_export()
            .map(|export| InstanceComponents {
                handle: core
                    .export(export)
                    .expect("Composition component export must be retained by its Instance"),
            });
        Ok(Instance {
            core,
            semantic_context: self.semantic_context(),
            components,
        })
    }
}

impl Instance {
    /// Returns the immutable raw Core runtime occurrence for deliberate
    /// advanced diagnostics.
    pub fn core(&self) -> &CoreInstance {
        &self.core
    }

    pub fn composition_id(&self) -> &CompositionId {
        self.core.composition_id()
    }

    pub fn instance_id(&self) -> &InstanceId {
        self.core.instance_id()
    }

    pub fn generation(&self) -> InstanceGeneration {
        self.core.generation()
    }

    pub fn lifecycle(&self) -> LifecycleState {
        self.core.lifecycle()
    }

    pub fn start(&mut self) -> Result<(), fabric_core::InstanceError> {
        self.core.start()
    }

    pub fn stop(&mut self) -> Result<(), fabric_core::RuntimeCleanupError> {
        self.core.stop()
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn semantic_context(&self) -> &Arc<FabricManifest> {
        &self.semantic_context
    }

    /// Returns the bounded Component live/operator surface, if this
    /// Composition has a Component host. Resource/System-only Compositions
    /// intentionally have no fake Component operator.
    pub fn components(&self) -> Option<&InstanceComponents> {
        self.components.as_ref()
    }
}

impl InstanceComponents {
    pub fn materialize<C>(&self) -> Result<ComponentStatus, ComponentError>
    where
        C: ComponentDefinition,
    {
        self.handle.materialize(&C::component_id())
    }

    pub fn dematerialize<C>(&self) -> Result<ComponentStatus, ComponentError>
    where
        C: ComponentDefinition,
    {
        self.handle.dematerialize(&C::component_id())
    }

    pub fn materialize_id(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentStatus, ComponentError> {
        self.handle.materialize(component_id)
    }

    pub fn dematerialize_id(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentStatus, ComponentError> {
        self.handle.dematerialize(component_id)
    }

    pub fn invoke_external<I, O>(
        &self,
        operation: &OperationKey<I, O>,
        input: I,
    ) -> OperationFuture<O>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
    {
        self.handle.invoke_external(operation, input)
    }
}
