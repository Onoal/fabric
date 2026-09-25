use std::sync::Arc;

use fabric_core::Health;

use crate::{
    ComponentContract, ComponentError, ComponentInstanceBinding, ComponentParticipation,
    ComponentStatus, InvocationContext, OperationFuture, OperationKey, OperationRail,
    OperationRailService,
};

pub(crate) trait ComponentScopeService: Send + Sync {
    fn begin_invocation(
        &self,
        participation: &ComponentParticipation,
    ) -> Result<InvocationContext, ComponentError>;

    fn continue_invocation(
        &self,
        participation: &ComponentParticipation,
        context: &InvocationContext,
    ) -> Result<(), ComponentError>;

    fn update_health(
        &self,
        participation: &ComponentParticipation,
        health: Health,
    ) -> Result<ComponentStatus, ComponentError>;
}

#[derive(Clone)]
pub struct ComponentScope {
    participation: ComponentParticipation,
    operations: OperationRail,
    inner: Arc<dyn ComponentScopeService>,
}

#[derive(Clone)]
pub struct ComponentInvocation {
    participation: ComponentParticipation,
    context: InvocationContext,
    operations: OperationRail,
}

impl ComponentScope {
    pub(crate) fn new(
        participation: ComponentParticipation,
        operations: Arc<dyn OperationRailService>,
        inner: Arc<dyn ComponentScopeService>,
    ) -> Self {
        Self {
            participation,
            operations: OperationRail::new(operations),
            inner,
        }
    }

    pub fn component(&self) -> &ComponentInstanceBinding {
        self.participation.component()
    }

    pub fn participation(&self) -> &ComponentParticipation {
        &self.participation
    }

    pub fn begin_invocation(&self) -> Result<ComponentInvocation, ComponentError> {
        let context = self.inner.begin_invocation(&self.participation)?;
        Ok(ComponentInvocation::new(
            self.participation.clone(),
            context,
            self.operations.clone(),
        ))
    }

    pub fn continue_invocation(
        &self,
        context: &InvocationContext,
    ) -> Result<ComponentInvocation, ComponentError> {
        self.inner
            .continue_invocation(&self.participation, context)?;
        Ok(ComponentInvocation::new(
            self.participation.clone(),
            context.clone(),
            self.operations.clone(),
        ))
    }

    pub fn update_health(&self, health: Health) -> Result<ComponentStatus, ComponentError> {
        self.inner.update_health(&self.participation, health)
    }
}

impl ComponentInvocation {
    pub(crate) fn new(
        participation: ComponentParticipation,
        context: InvocationContext,
        operations: OperationRail,
    ) -> Self {
        Self {
            participation,
            context,
            operations,
        }
    }

    pub fn component(&self) -> &ComponentInstanceBinding {
        self.participation.component()
    }

    pub fn participation(&self) -> &ComponentParticipation {
        &self.participation
    }

    pub fn context(&self) -> &InvocationContext {
        &self.context
    }

    pub fn invoke<I, O>(&self, operation: &OperationKey<I, O>, input: I) -> OperationFuture<O>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
    {
        self.operations
            .invoke_with_context(self.context.clone(), operation, input)
    }

    pub fn call<T, R>(
        &self,
        contract: &ComponentContract<T>,
        call: impl FnOnce(&InvocationContext, &T) -> R,
    ) -> Result<R, ComponentError> {
        contract.call_with_context(&self.participation, &self.context, call)
    }
}
