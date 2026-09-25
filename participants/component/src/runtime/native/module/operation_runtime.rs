use std::any::TypeId;

use super::*;

impl OperationRailService for SharedComponentState {
    fn operation(&self, operation_id: &OperationId) -> Result<OperationDescriptor, ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        let operation = state
            .operations
            .get(operation_id)
            .ok_or_else(|| ComponentError::UnknownOperation(operation_id.clone()))?;
        if !current_active_participation(&state, &operation.owner) {
            return Err(ComponentError::UnknownOperation(operation_id.clone()));
        }
        Ok(OperationDescriptor::new(
            operation.definition.clone(),
            operation.owner.component().clone(),
        ))
    }

    fn operations(&self) -> Vec<OperationDescriptor> {
        let state = self.inner.lock().expect("component runtime state lock");
        state
            .operations
            .values()
            .filter(|operation| current_active_participation(&state, &operation.owner))
            .map(|operation| {
                OperationDescriptor::new(
                    operation.definition.clone(),
                    operation.owner.component().clone(),
                )
            })
            .collect()
    }

    fn invoke_erased(
        &self,
        context: crate::InvocationContext,
        operation_id: &OperationId,
        input_type: TypeId,
        output_type: TypeId,
        input: ErasedOperationInput,
    ) -> OperationFuture<ErasedOperationOutput> {
        let (operation, context) = {
            let state = self.inner.lock().expect("component runtime state lock");
            match state.current_status().lifecycle() {
                ComponentHostLifecycle::Ready => {}
                ComponentHostLifecycle::Starting
                | ComponentHostLifecycle::Stopping
                | ComponentHostLifecycle::Stopped => {
                    return Box::pin(async { Err(ComponentError::Unavailable) });
                }
            }
            let operation = match state.operations.get(operation_id) {
                Some(operation) => operation.clone(),
                None => {
                    let operation_id = operation_id.clone();
                    return Box::pin(
                        async move { Err(ComponentError::UnknownOperation(operation_id)) },
                    );
                }
            };
            if operation.input_type != input_type || operation.output_type != output_type {
                let operation_id = operation_id.clone();
                return Box::pin(async move {
                    Err(ComponentError::OperationTypeMismatch(operation_id))
                });
            }
            if let Err(error) = validate_context_runtime(&state, &context) {
                return Box::pin(async move { Err(error) });
            }
            if !current_participation(&state, &operation.owner) {
                let operation_id = operation_id.clone();
                let component_id = operation.owner.component().component_id().clone();
                return Box::pin(async move {
                    Err(ComponentError::OperationOwnerNotParticipating {
                        operation_id,
                        component_id,
                    })
                });
            }
            if !current_active_participation(&state, &operation.owner)
                || !participation_is_available(&state, &operation.owner)
            {
                let component_id = operation.owner.component().component_id().clone();
                return Box::pin(
                    async move { Err(ComponentError::ComponentUnavailable(component_id)) },
                );
            }
            (operation, context)
        };

        (operation.handler)(context, input)
    }
}

impl OperationRegistrarService for SharedComponentState {
    fn register_erased(
        &self,
        owner: ComponentParticipation,
        definition: OperationDefinition,
        input_type: TypeId,
        output_type: TypeId,
        handler: RegisteredOperationHandler,
    ) -> Result<(), ComponentError> {
        let mut state = self.inner.lock().expect("component runtime state lock");
        let operation_id = definition.id().clone();
        if owner.component().instance_id() != state.current_status().instance_id() {
            return Err(ComponentError::OperationOwnerInstanceMismatch {
                operation_id: operation_id.clone(),
                owner_instance_id: owner.component().instance_id().clone(),
                instance_id: state.instance_id(),
            });
        }
        if !current_participation(&state, &owner) {
            return Err(ComponentError::OperationOwnerNotParticipating {
                operation_id: operation_id.clone(),
                component_id: owner.component().component_id().clone(),
            });
        }
        if !current_preparing_participation(&state, &owner) {
            return Err(ComponentError::OperationOwnerNotPreparing {
                operation_id: operation_id.clone(),
                component_id: owner.component().component_id().clone(),
            });
        }
        // Runtime handlers may only realize endpoints the owner's declaration
        // already describes. Runtime never rewrites declarative truth.
        let owner_id = owner.component().component_id().clone();
        let Some(declaration) = state.component_declarations.get(&owner_id) else {
            return Err(ComponentError::UnknownComponent(owner_id));
        };
        match declaration
            .operations()
            .iter()
            .find(|operation| operation.id() == &operation_id)
        {
            None => {
                return Err(ComponentError::UndeclaredComponentOperation {
                    component_id: owner_id,
                    operation_id,
                });
            }
            Some(declared)
                if declared.input_type() != definition.input_type()
                    || declared.output_type() != definition.output_type() =>
            {
                return Err(ComponentError::OperationDefinitionTypeMismatch {
                    operation_id,
                    expected_input_type: declared.input_type().clone(),
                    actual_input_type: definition.input_type().clone(),
                    expected_output_type: declared.output_type().clone(),
                    actual_output_type: definition.output_type().clone(),
                });
            }
            Some(_) => {}
        }
        if let Some(existing) = state.operations.get(&operation_id) {
            if existing.definition != definition {
                return Err(ComponentError::OperationDefinitionTypeMismatch {
                    operation_id,
                    expected_input_type: existing.definition.input_type().clone(),
                    actual_input_type: definition.input_type().clone(),
                    expected_output_type: existing.definition.output_type().clone(),
                    actual_output_type: definition.output_type().clone(),
                });
            }
            return Err(ComponentError::DuplicateOperationId(
                definition.id().clone(),
            ));
        }
        state.operations.insert(
            operation_id,
            RegisteredOperation {
                owner,
                definition,
                input_type,
                output_type,
                handler,
            },
        );
        Ok(())
    }
}
