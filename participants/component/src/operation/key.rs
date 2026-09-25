use std::marker::PhantomData;

use crate::{OperationDefinition, OperationId, OperationTypeId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationKey<I, O>
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
{
    definition: OperationDefinition,
    marker: PhantomData<(I, O)>,
}

impl<I, O> OperationKey<I, O>
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
{
    pub fn new(id: OperationId, input_type: OperationTypeId, output_type: OperationTypeId) -> Self {
        Self {
            definition: OperationDefinition::new(id, input_type, output_type),
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> &OperationId {
        self.definition.id()
    }

    pub fn input_type(&self) -> &OperationTypeId {
        self.definition.input_type()
    }

    pub fn output_type(&self) -> &OperationTypeId {
        self.definition.output_type()
    }

    pub fn definition(&self) -> &OperationDefinition {
        &self.definition
    }
}
