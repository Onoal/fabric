use std::sync::Arc;

use fabric_core::{ContractId, ContractKey};

use crate::{
    ComponentControl, ComponentControlRail, ComponentDesiredState, ComponentError, ComponentId,
    ComponentInstanceBinding, ComponentMaterializer, ComponentReconstructionRail,
    ComponentReconstructionReport, ComponentRegistry, ComponentStatus, InvocationContext,
    InvocationRail, OperationFuture, OperationKey, OperationRail,
};

const COMPONENT_RUNTIME_HANDLE_CONTRACT_ID: &str = "fabric.component.runtime-handle";

pub fn component_host_handle_contract_id() -> ContractId {
    ContractId::new(COMPONENT_RUNTIME_HANDLE_CONTRACT_ID)
        .expect("static component runtime handle contract id")
}

pub fn component_host_handle_contract_key() -> ContractKey<ComponentHostHandle> {
    ContractKey::provisional(component_host_handle_contract_id())
}

/// Operator-oriented, Instance-local Component host façade. It composes the
/// existing rails without exposing their binding or provider mechanics.
#[derive(Clone)]
pub struct ComponentHostHandle {
    materializer: Arc<ComponentMaterializer>,
    control: Arc<ComponentControlRail>,
    reconstruction: Arc<ComponentReconstructionRail>,
    registry: Arc<ComponentRegistry>,
    invocation: Arc<InvocationRail>,
    operations: Arc<OperationRail>,
}

impl std::fmt::Debug for ComponentHostHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentHostHandle")
            .finish_non_exhaustive()
    }
}

impl ComponentHostHandle {
    pub fn new(
        materializer: Arc<ComponentMaterializer>,
        control: Arc<ComponentControlRail>,
        reconstruction: Arc<ComponentReconstructionRail>,
        registry: Arc<ComponentRegistry>,
        invocation: Arc<InvocationRail>,
        operations: Arc<OperationRail>,
    ) -> Self {
        Self {
            materializer,
            control,
            reconstruction,
            registry,
            invocation,
            operations,
        }
    }

    pub fn materialize(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentStatus, ComponentError> {
        self.materializer.materialize(component_id)
    }

    pub fn dematerialize(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentStatus, ComponentError> {
        self.materializer.dematerialize(component_id)
    }

    pub fn set_desired(
        &self,
        component: ComponentInstanceBinding,
        desired: ComponentDesiredState,
    ) -> Result<ComponentControl, ComponentError> {
        self.control.set_desired(component, desired)
    }

    pub fn control(&self, component_id: &ComponentId) -> Result<ComponentControl, ComponentError> {
        self.control.control(component_id)
    }

    pub fn controls(&self) -> Vec<ComponentControl> {
        self.control.controls()
    }

    pub fn reconstruct(&self) -> Result<ComponentReconstructionReport, ComponentError> {
        self.reconstruction.reconstruct()
    }

    pub fn status(&self, component_id: &ComponentId) -> Result<ComponentStatus, ComponentError> {
        self.registry.component(component_id)
    }

    pub fn statuses(&self) -> Vec<ComponentStatus> {
        self.registry.components()
    }

    pub fn begin_external(&self) -> Result<InvocationContext, ComponentError> {
        self.invocation.begin_external()
    }

    pub fn invoke_with_context<I, O>(
        &self,
        context: InvocationContext,
        operation: &OperationKey<I, O>,
        input: I,
    ) -> OperationFuture<O>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
    {
        self.operations
            .invoke_with_context(context, operation, input)
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
        let context = self.begin_external();
        let operations = Arc::clone(&self.operations);
        let operation = OperationKey::new(
            operation.id().clone(),
            operation.input_type().clone(),
            operation.output_type().clone(),
        );
        Box::pin(async move {
            let context = context?;
            operations
                .invoke_with_context(context, &operation, input)
                .await
        })
    }
}
