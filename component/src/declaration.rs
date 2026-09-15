use fabric_core::ContractRequirementDeclaration;
use fabric_resource::ResourceId;
use fabric_system::SystemId;

use crate::{ComponentId, OperationDefinition};

/// A stable name for a Resource requirement local to one Component declaration.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentResourceRequirementName(String);

impl ComponentResourceRequirementName {
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.is_empty() {
            return Err("Component Resource requirement name must not be empty");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Runtime-free declaration of a Resource capability required by a Component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentResourceRequirementDeclaration {
    name: ComponentResourceRequirementName,
    resource_id: ResourceId,
    requirement: ContractRequirementDeclaration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentSystemRequirementDeclaration {
    system_id: SystemId,
    requirement: ContractRequirementDeclaration,
}
impl ComponentSystemRequirementDeclaration {
    pub fn new(system_id: SystemId, requirement: ContractRequirementDeclaration) -> Self {
        Self {
            system_id,
            requirement,
        }
    }
    pub fn system_id(&self) -> &SystemId {
        &self.system_id
    }
    pub fn requirement(&self) -> &ContractRequirementDeclaration {
        &self.requirement
    }
}

impl ComponentResourceRequirementDeclaration {
    pub fn new(
        name: ComponentResourceRequirementName,
        resource_id: ResourceId,
        requirement: ContractRequirementDeclaration,
    ) -> Self {
        Self {
            name,
            resource_id,
            requirement,
        }
    }

    pub fn name(&self) -> &ComponentResourceRequirementName {
        &self.name
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn requirement(&self) -> &ContractRequirementDeclaration {
        &self.requirement
    }
}

/// Runtime-free declarative truth for one Component.
///
/// A declaration carries stable semantic behavior identity (the
/// `ComponentId`) plus zero or more behavior endpoint declarations. It is
/// independent of runtime scope, health, instances, and handler attachment:
///
/// - a declaration with operations describes invocable behavior endpoints;
/// - a declaration without operations describes a valid Component whose
///   behavior is not invocation-based (pure consumer, coordinator, or a
///   Component using other semantic rails).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentDeclaration {
    component_id: ComponentId,
    operations: Vec<OperationDefinition>,
    resource_requirements: Vec<ComponentResourceRequirementDeclaration>,
    system_requirements: Vec<ComponentSystemRequirementDeclaration>,
}

impl ComponentDeclaration {
    pub fn new(component_id: ComponentId, operations: Vec<OperationDefinition>) -> Self {
        Self {
            component_id,
            operations,
            resource_requirements: Vec::new(),
            system_requirements: Vec::new(),
        }
    }

    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    pub fn operations(&self) -> &[OperationDefinition] {
        &self.operations
    }

    pub fn with_resource_requirements(
        mut self,
        requirements: Vec<ComponentResourceRequirementDeclaration>,
    ) -> Self {
        self.resource_requirements = requirements;
        self
    }

    pub fn resource_requirements(&self) -> &[ComponentResourceRequirementDeclaration] {
        &self.resource_requirements
    }

    pub fn with_system_requirements(
        mut self,
        requirements: Vec<ComponentSystemRequirementDeclaration>,
    ) -> Self {
        self.system_requirements = requirements;
        self
    }
    pub fn system_requirements(&self) -> &[ComponentSystemRequirementDeclaration] {
        &self.system_requirements
    }
}
