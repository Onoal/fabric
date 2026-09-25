use fabric_core::ContractRequirementDeclaration;
use fabric_resource::ResourceId;
use fabric_system::SystemId;

use crate::OperationDefinition;

mod component;

pub use component::ComponentId;

/// Authored semantic endpoint name for a Component API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentApiEndpoint {
    name: String,
}

impl ComponentApiEndpoint {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Runtime-free semantic API metadata declared by a Component.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComponentApiMetadata {
    endpoints: Vec<ComponentApiEndpoint>,
}

impl ComponentApiMetadata {
    pub fn new(endpoints: Vec<ComponentApiEndpoint>) -> Self {
        Self { endpoints }
    }

    pub fn endpoints(&self) -> &[ComponentApiEndpoint] {
        &self.endpoints
    }
}

/// A stable local role for one capability relation owned by a Component.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentRelationName(String);

impl ComponentRelationName {
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.is_empty() {
            return Err("Component relation name must not be empty");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Runtime-free declaration of one named semantic capability relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentRelationDeclaration {
    name: ComponentRelationName,
    requirement: ContractRequirementDeclaration,
}

impl ComponentRelationDeclaration {
    pub fn new(name: ComponentRelationName, requirement: ContractRequirementDeclaration) -> Self {
        Self { name, requirement }
    }

    pub fn name(&self) -> &ComponentRelationName {
        &self.name
    }

    pub fn requirement(&self) -> &ContractRequirementDeclaration {
        &self.requirement
    }
}

/// Runtime-free declaration of a Resource capability required by a Component.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentResourceRequirementDeclaration {
    name: ComponentRelationName,
    resource_id: ResourceId,
    requirement: ContractRequirementDeclaration,
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentSystemRequirementDeclaration {
    name: ComponentRelationName,
    system_id: SystemId,
    requirement: ContractRequirementDeclaration,
}
impl ComponentSystemRequirementDeclaration {
    pub fn new(
        name: ComponentRelationName,
        system_id: SystemId,
        requirement: ContractRequirementDeclaration,
    ) -> Self {
        Self {
            name,
            system_id,
            requirement,
        }
    }
    pub fn name(&self) -> &ComponentRelationName {
        &self.name
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
        name: ComponentRelationName,
        resource_id: ResourceId,
        requirement: ContractRequirementDeclaration,
    ) -> Self {
        Self {
            name,
            resource_id,
            requirement,
        }
    }

    pub fn name(&self) -> &ComponentRelationName {
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
/// - a declaration with API endpoints describes invocable behavior (lowered internally as
///   operations);
/// - a declaration without API endpoints describes a valid Component whose
///   behavior is not invocation-based (pure consumer, coordinator, or a
///   Component using other semantic rails).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentDeclaration {
    component_id: ComponentId,
    operations: Vec<OperationDefinition>,
    api: ComponentApiMetadata,
    relations: Vec<ComponentRelationDeclaration>,
    resource_requirements: Vec<ComponentResourceRequirementDeclaration>,
    system_requirements: Vec<ComponentSystemRequirementDeclaration>,
}

impl ComponentDeclaration {
    pub fn new(component_id: ComponentId, operations: Vec<OperationDefinition>) -> Self {
        Self {
            component_id,
            operations,
            api: ComponentApiMetadata::default(),
            relations: Vec::new(),
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

    pub fn with_api_metadata(mut self, api: ComponentApiMetadata) -> Self {
        self.api = api;
        self
    }

    pub fn api(&self) -> &ComponentApiMetadata {
        &self.api
    }

    pub fn with_relations(mut self, relations: Vec<ComponentRelationDeclaration>) -> Self {
        self.relations = relations;
        self
    }

    pub fn relations(&self) -> &[ComponentRelationDeclaration] {
        &self.relations
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
