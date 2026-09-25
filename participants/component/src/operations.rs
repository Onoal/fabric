use std::any::{Any, TypeId};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use fabric_core::{ContractId, ContractKey};

use crate::{
    ComponentError, ComponentInstanceBinding, ComponentParticipation, InvocationContext,
    OperationDefinition, OperationDescriptor, OperationId, OperationKey,
};

type ErasedOperationInput = Box<dyn Any + Send + Sync>;
type ErasedOperationOutput = Box<dyn Any + Send + Sync>;
pub type OperationFuture<T> =
    Pin<Box<dyn Future<Output = Result<T, ComponentError>> + Send + 'static>>;
type ErasedOperationHandler = Arc<
    dyn Fn(InvocationContext, ErasedOperationInput) -> OperationFuture<ErasedOperationOutput>
        + Send
        + Sync,
>;

const COMPONENT_OPERATION_RAIL_CONTRACT_ID: &str = "fabric.component.operation";
const COMPONENT_OPERATION_REGISTRAR_CONTRACT_ID: &str = "fabric.component.operation.registrar";

pub fn operation_rail_contract_id() -> ContractId {
    ContractId::new(COMPONENT_OPERATION_RAIL_CONTRACT_ID)
        .expect("static component operation rail contract id")
}

pub fn operation_rail_contract_key() -> ContractKey<OperationRail> {
    ContractKey::provisional(operation_rail_contract_id())
}

pub fn operation_registrar_contract_id() -> ContractId {
    ContractId::new(COMPONENT_OPERATION_REGISTRAR_CONTRACT_ID)
        .expect("static component operation registrar contract id")
}

pub fn operation_registrar_contract_key() -> ContractKey<OperationRegistrar> {
    ContractKey::provisional(operation_registrar_contract_id())
}

pub trait OperationRailService: Send + Sync {
    fn operation(&self, operation_id: &OperationId) -> Result<OperationDescriptor, ComponentError>;

    fn operations(&self) -> Vec<OperationDescriptor>;

    fn invoke_erased(
        &self,
        context: InvocationContext,
        operation_id: &OperationId,
        input_type: TypeId,
        output_type: TypeId,
        input: ErasedOperationInput,
    ) -> OperationFuture<ErasedOperationOutput>;
}

pub trait OperationRegistrarService: Send + Sync {
    fn register_erased(
        &self,
        owner: ComponentParticipation,
        definition: OperationDefinition,
        input_type: TypeId,
        output_type: TypeId,
        handler: ErasedOperationHandler,
    ) -> Result<(), ComponentError>;
}

#[derive(Clone)]
pub struct OperationRail {
    inner: Arc<dyn OperationRailService>,
}

#[derive(Clone)]
pub struct OperationRegistrar {
    inner: Arc<dyn OperationRegistrarService>,
}

impl OperationRail {
    pub fn new(inner: Arc<dyn OperationRailService>) -> Self {
        Self { inner }
    }

    pub fn owner<I, O>(
        &self,
        operation: &OperationKey<I, O>,
    ) -> Result<ComponentInstanceBinding, ComponentError>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
    {
        Ok(self.inner.operation(operation.id())?.owner().clone())
    }

    pub fn operation(
        &self,
        operation_id: &OperationId,
    ) -> Result<OperationDescriptor, ComponentError> {
        self.inner.operation(operation_id)
    }

    pub fn operations(&self) -> Vec<OperationDescriptor> {
        self.inner.operations()
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
        let operation_id = operation.id().clone();
        let inner = Arc::clone(&self.inner);
        Box::pin(async move {
            let output = inner
                .invoke_erased(
                    context,
                    &operation_id,
                    TypeId::of::<I>(),
                    TypeId::of::<O>(),
                    Box::new(input),
                )
                .await?;
            output
                .downcast::<O>()
                .map(|value| *value)
                .map_err(|_| ComponentError::OperationTypeMismatch(operation_id))
        })
    }

    pub fn invoke_erased(
        &self,
        context: InvocationContext,
        operation_id: &OperationId,
        input_type: TypeId,
        output_type: TypeId,
        input: Box<dyn Any + Send + Sync>,
    ) -> OperationFuture<Box<dyn Any + Send + Sync>> {
        self.inner
            .invoke_erased(context, operation_id, input_type, output_type, input)
    }
}

impl OperationRegistrar {
    pub fn new(inner: Arc<dyn OperationRegistrarService>) -> Self {
        Self { inner }
    }

    pub fn register<I, O, F, Fut>(
        &self,
        owner: ComponentParticipation,
        operation: OperationKey<I, O>,
        handler: F,
    ) -> Result<(), ComponentError>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, ComponentError>> + Send + 'static,
    {
        let operation_id = operation.id().clone();
        let typed_operation_id = operation_id.clone();
        let erased_handler: ErasedOperationHandler = Arc::new(move |_context, input| {
            let typed_operation_id = typed_operation_id.clone();
            match input.downcast::<I>() {
                Ok(input) => {
                    let future = handler(*input);
                    Box::pin(async move {
                        let output = future.await?;
                        Ok(Box::new(output) as ErasedOperationOutput)
                    })
                }
                Err(_) => Box::pin(async move {
                    Err(ComponentError::OperationTypeMismatch(typed_operation_id))
                }),
            }
        });
        self.inner.register_erased(
            owner,
            operation.definition().clone(),
            TypeId::of::<I>(),
            TypeId::of::<O>(),
            erased_handler,
        )
    }

    pub fn register_with_context<I, O, F, Fut>(
        &self,
        owner: ComponentParticipation,
        operation: OperationKey<I, O>,
        handler: F,
    ) -> Result<(), ComponentError>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
        F: Fn(InvocationContext, I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, ComponentError>> + Send + 'static,
    {
        let operation_id = operation.id().clone();
        let typed_operation_id = operation_id.clone();
        let erased_handler: ErasedOperationHandler = Arc::new(move |context, input| {
            let typed_operation_id = typed_operation_id.clone();
            match input.downcast::<I>() {
                Ok(input) => {
                    let future = handler(context, *input);
                    Box::pin(async move { Ok(Box::new(future.await?) as ErasedOperationOutput) })
                }
                Err(_) => Box::pin(async move {
                    Err(ComponentError::OperationTypeMismatch(typed_operation_id))
                }),
            }
        });
        self.inner.register_erased(
            owner,
            operation.definition().clone(),
            TypeId::of::<I>(),
            TypeId::of::<O>(),
            erased_handler,
        )
    }
}
