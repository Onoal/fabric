use crate::{ContractId, ContractRequirement, ContractRequirementDeclaration};

/// A deliberately declared runtime capability exposed to an external
/// Composition operator. This is not a general Instance contract lookup: an
/// Instance retains only values declared through this type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionExport<T>
where
    T: Send + Sync + 'static,
{
    id: ContractId,
    requirement: ContractRequirement<T>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionExportDeclaration {
    id: ContractId,
    requirement: ContractRequirementDeclaration,
}

impl<T> CompositionExport<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(id: ContractId, requirement: ContractRequirement<T>) -> Self {
        Self { id, requirement }
    }

    pub fn id(&self) -> &ContractId {
        &self.id
    }

    pub fn requirement(&self) -> &ContractRequirement<T> {
        &self.requirement
    }

    pub fn declaration(&self) -> CompositionExportDeclaration {
        CompositionExportDeclaration {
            id: self.id.clone(),
            requirement: self.requirement.declaration().clone(),
        }
    }
}

impl CompositionExportDeclaration {
    pub fn id(&self) -> &ContractId {
        &self.id
    }

    pub fn requirement(&self) -> &ContractRequirementDeclaration {
        &self.requirement
    }
}
