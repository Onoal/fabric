use crate::{Component, OperationId, OperationTypeId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationDefinition {
    id: OperationId,
    input_type: OperationTypeId,
    output_type: OperationTypeId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationDescriptor {
    definition: OperationDefinition,
    owner: Component,
}

impl OperationDefinition {
    pub fn new(id: OperationId, input_type: OperationTypeId, output_type: OperationTypeId) -> Self {
        Self {
            id,
            input_type,
            output_type,
        }
    }

    pub fn id(&self) -> &OperationId {
        &self.id
    }

    pub fn input_type(&self) -> &OperationTypeId {
        &self.input_type
    }

    pub fn output_type(&self) -> &OperationTypeId {
        &self.output_type
    }
}

impl OperationDescriptor {
    pub fn new(definition: OperationDefinition, owner: Component) -> Self {
        Self { definition, owner }
    }

    pub fn definition(&self) -> &OperationDefinition {
        &self.definition
    }

    pub fn owner(&self) -> &Component {
        &self.owner
    }
}
