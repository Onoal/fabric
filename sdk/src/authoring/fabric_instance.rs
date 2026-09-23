use fabric_component::{
    ComponentError, ComponentHostHandle, ComponentId, ComponentStatus, OperationFuture,
    OperationKey,
};
use fabric_core::{
    CompositionError, Instance, InstanceGeneration, InstanceId, InstanceReport, LifecycleState,
};
use fabric_host::HostDescriptor;

use super::{ComponentDefinition, Composition};
use crate::ids::IntoInstanceId;

/// High-level live Fabric Instance. It delegates lifecycle and inspection to
/// Core while exposing only the deliberate ComponentInstanceBinding operator boundary.
pub struct FabricInstance {
    core: Instance,
    components: Option<FabricComponents>,
}

impl std::fmt::Debug for FabricInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FabricInstance")
            .field("instance_id", self.instance_id())
            .field("generation", &self.generation())
            .field("lifecycle", &self.lifecycle())
            .finish()
    }
}

#[derive(Clone)]
pub struct FabricComponents {
    handle: std::sync::Arc<ComponentHostHandle>,
}

impl Composition {
    pub fn materialize_named<I>(&self, instance_id: I) -> Result<FabricInstance, CompositionError>
    where
        I: IntoInstanceId,
    {
        self.materialize_named_with_host(instance_id, None)
    }

    pub fn materialize_named_on<I>(
        &self,
        instance_id: I,
        host: &HostDescriptor,
    ) -> Result<FabricInstance, CompositionError>
    where
        I: IntoInstanceId,
    {
        self.materialize_named_with_host(instance_id, Some(host))
    }

    fn materialize_named_with_host<I>(
        &self,
        instance_id: I,
        host: Option<&HostDescriptor>,
    ) -> Result<FabricInstance, CompositionError>
    where
        I: IntoInstanceId,
    {
        let core = match host {
            Some(host) => self
                .core()
                .materialize_on(instance_id.into_instance_id()?, host)?,
            None => self.core().materialize(instance_id.into_instance_id()?)?,
        };
        let components = self.component_host_export().map(|export| FabricComponents {
            handle: core
                .export(export)
                .expect("Composition component export must be retained by its Instance"),
        });
        Ok(FabricInstance { core, components })
    }
}

impl FabricInstance {
    pub fn core(&self) -> &Instance {
        &self.core
    }
    pub fn core_mut(&mut self) -> &mut Instance {
        &mut self.core
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
    pub fn report(&self) -> InstanceReport {
        self.core.report()
    }
    pub fn start(&mut self) -> Result<(), fabric_core::InstanceError> {
        self.core.start()
    }
    pub fn stop(&mut self) -> Result<(), fabric_core::RuntimeCleanupError> {
        self.core.stop()
    }

    /// Returns the bounded ComponentInstanceBinding control surface, if this Composition has
    /// a ComponentInstanceBinding host. Resource/System-only Compositions intentionally have
    /// no fake ComponentInstanceBinding operator.
    pub fn components(&self) -> Option<&FabricComponents> {
        self.components.as_ref()
    }
}

impl FabricComponents {
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
