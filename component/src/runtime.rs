use std::any::Any;
use std::collections::BTreeMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use fabric_core::{
    ContractId, ContractKey, ContractRequirement, ContractRequirementDeclaration, Health,
};

use crate::ComponentResourceRequirementName;
use crate::component_scope::ComponentScopeService;
use crate::{
    Component, ComponentError, ComponentId, ComponentParticipation, ComponentScope,
    ComponentStatus, InvocationContext, OperationKey, OperationRailService, OperationRegistrar,
    OperationRegistrarService,
};

const COMPONENT_MATERIALIZER_CONTRACT_ID: &str = "fabric.component.materializer";

/// Instance-local transport for a Core-resolved Component capability dependency.
/// Its erased storage is private; semantic SDK scope extensions retrieve typed
/// Resource or System contracts through their respective APIs.
pub struct ComponentResourceDependency {
    component_id: ComponentId,
    requirement: ContractRequirementDeclaration,
    value: Mutex<Option<Arc<dyn Any + Send + Sync>>>,
}

impl ComponentResourceDependency {
    pub fn new<T>(component_id: ComponentId, requirement: ContractRequirementDeclaration) -> Self
    where
        T: Send + Sync + 'static,
    {
        let _ = std::marker::PhantomData::<T>;
        Self {
            component_id,
            requirement,
            value: Mutex::new(None),
        }
    }

    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    pub fn requirement(&self) -> &ContractRequirementDeclaration {
        &self.requirement
    }

    pub fn bind<T>(&self, value: Arc<T>)
    where
        T: Send + Sync + 'static,
    {
        *self.value.lock().expect("component dependency lock") = Some(value);
    }

    fn resolve<T>(&self, requirement: &ContractRequirement<T>) -> Result<Arc<T>, ComponentError>
    where
        T: Send + Sync + 'static,
    {
        if &self.requirement != requirement.declaration() {
            return Err(ComponentError::Unavailable);
        }
        let value = self
            .value
            .lock()
            .expect("component dependency lock")
            .clone()
            .ok_or(ComponentError::Unavailable)?;
        value
            .downcast::<T>()
            .map_err(|_| ComponentError::Unavailable)
    }
}

pub fn component_resource_dependency_contract_id(
    component_id: &ComponentId,
    requirement: &ContractRequirementDeclaration,
) -> ContractId {
    let encoded = format!(
        "{}:{}:{}",
        component_id.as_str(),
        requirement.id().as_str(),
        requirement.compatibility()
    )
    .as_bytes()
    .iter()
    .flat_map(|byte| [nibble(byte >> 4), nibble(byte & 15)])
    .collect::<String>();
    ContractId::new(format!("fabric.component.resource-dependency.{encoded}"))
        .expect("encoded component dependency contract id")
}

fn nibble(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        _ => char::from(b'a' + value - 10),
    }
}

pub fn component_resource_dependency_contract_key(
    component_id: &ComponentId,
    requirement: &ContractRequirementDeclaration,
) -> ContractKey<ComponentResourceDependency> {
    ContractKey::provisional(component_resource_dependency_contract_id(
        component_id,
        requirement,
    ))
}

pub fn component_named_resource_dependency_contract_key(
    component_id: &ComponentId,
    name: &ComponentResourceRequirementName,
    requirement: &ContractRequirementDeclaration,
) -> ContractKey<ComponentResourceDependency> {
    let encoded = format!(
        "{}:{}:{}:{}",
        component_id.as_str(),
        name.as_str(),
        requirement.id().as_str(),
        requirement.compatibility()
    )
    .as_bytes()
    .iter()
    .flat_map(|byte| [nibble(byte >> 4), nibble(byte & 15)])
    .collect::<String>();
    ContractKey::provisional(
        ContractId::new(format!("fabric.component.resource-dependency.{encoded}"))
            .expect("encoded component named resource dependency contract id"),
    )
}

pub fn component_system_dependency_contract_key(
    component_id: &ComponentId,
    requirement: &ContractRequirementDeclaration,
) -> ContractKey<ComponentResourceDependency> {
    let encoded = format!(
        "{}:{}:{}",
        component_id.as_str(),
        requirement.id().as_str(),
        requirement.compatibility()
    )
    .as_bytes()
    .iter()
    .flat_map(|byte| [nibble(byte >> 4), nibble(byte & 15)])
    .collect::<String>();
    ContractKey::provisional(
        ContractId::new(format!("fabric.component.system-dependency.{encoded}"))
            .expect("encoded component system dependency contract id"),
    )
}

pub fn component_materializer_contract_id() -> ContractId {
    ContractId::new(COMPONENT_MATERIALIZER_CONTRACT_ID)
        .expect("static component materializer contract id")
}

pub fn component_materializer_contract_key() -> ContractKey<ComponentMaterializer> {
    ContractKey::provisional(component_materializer_contract_id())
}

#[derive(Clone)]
pub struct ComponentRuntimeDefinition {
    component_id: ComponentId,
    prepare: Arc<ComponentRuntimePrepareFn>,
}

type ComponentRuntimePrepareFn =
    dyn Fn(&ComponentRuntimeScope) -> Result<Health, ComponentError> + Send + Sync;

pub trait ComponentMaterializerService: Send + Sync {
    fn known_component_ids(&self) -> Vec<ComponentId>;

    fn materialize(&self, component_id: &ComponentId) -> Result<ComponentStatus, ComponentError>;

    fn dematerialize(&self, component_id: &ComponentId) -> Result<ComponentStatus, ComponentError>;
}

#[derive(Clone)]
pub struct ComponentMaterializer {
    inner: Arc<dyn ComponentMaterializerService>,
}

#[derive(Clone)]
pub struct ComponentRuntimeScope {
    participation: ComponentParticipation,
    operations: OperationRegistrar,
    component_scope: ComponentScope,
    resource_dependencies: Arc<BTreeMap<ContractId, Arc<ComponentResourceDependency>>>,
}

impl ComponentRuntimeDefinition {
    pub fn new(
        component_id: ComponentId,
        prepare: impl Fn(&ComponentRuntimeScope) -> Result<Health, ComponentError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            component_id,
            prepare: Arc::new(prepare),
        }
    }

    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    /// Prepares one generation-scoped Component participation.
    pub fn prepare(&self, scope: &ComponentRuntimeScope) -> Result<Health, ComponentError> {
        (self.prepare)(scope)
    }
}

impl ComponentMaterializer {
    pub fn new(inner: Arc<dyn ComponentMaterializerService>) -> Self {
        Self { inner }
    }

    pub fn known_component_ids(&self) -> Vec<ComponentId> {
        self.inner.known_component_ids()
    }

    pub fn materialize(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentStatus, ComponentError> {
        self.inner.materialize(component_id)
    }

    pub fn dematerialize(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentStatus, ComponentError> {
        self.inner.dematerialize(component_id)
    }
}

impl ComponentRuntimeScope {
    pub(crate) fn new(
        participation: ComponentParticipation,
        operation_invocation: Arc<dyn OperationRailService>,
        operations: Arc<dyn OperationRegistrarService>,
        component_scope: Arc<dyn ComponentScopeService>,
        resource_dependencies: Arc<BTreeMap<ContractId, Arc<ComponentResourceDependency>>>,
    ) -> Self {
        Self {
            component_scope: ComponentScope::new(
                participation.clone(),
                operation_invocation,
                component_scope,
            ),
            participation,
            operations: OperationRegistrar::new(operations),
            resource_dependencies,
        }
    }

    pub fn component(&self) -> &Component {
        self.participation.component()
    }

    pub fn participation(&self) -> &ComponentParticipation {
        &self.participation
    }

    pub fn component_scope(&self) -> ComponentScope {
        self.component_scope.clone()
    }

    /// Retrieves this preparing Component's already Core-resolved Resource handoff.
    pub fn resource_dependency<T>(
        &self,
        requirement: &ContractRequirement<T>,
    ) -> Result<Arc<T>, ComponentError>
    where
        T: Send + Sync + 'static,
    {
        let key = component_resource_dependency_contract_id(
            self.component().component_id(),
            requirement.declaration(),
        );
        let dependency = self
            .resource_dependencies
            .get(&key)
            .ok_or(ComponentError::Unavailable)?;
        if dependency.component_id() != self.component().component_id() {
            return Err(ComponentError::Unavailable);
        }
        dependency.resolve(requirement)
    }

    pub fn named_resource_dependency<T>(
        &self,
        name: &ComponentResourceRequirementName,
        requirement: &ContractRequirement<T>,
    ) -> Result<Arc<T>, ComponentError>
    where
        T: Send + Sync + 'static,
    {
        let key = component_named_resource_dependency_contract_key(
            self.component().component_id(),
            name,
            requirement.declaration(),
        );
        let dependency = self
            .resource_dependencies
            .get(key.id())
            .ok_or(ComponentError::Unavailable)?;
        if dependency.component_id() != self.component().component_id() {
            return Err(ComponentError::Unavailable);
        }
        dependency.resolve(requirement)
    }

    pub fn system_dependency<T>(
        &self,
        requirement: &ContractRequirement<T>,
    ) -> Result<Arc<T>, ComponentError>
    where
        T: Send + Sync + 'static,
    {
        let key = component_system_dependency_contract_key(
            self.component().component_id(),
            requirement.declaration(),
        );
        let dependency = self
            .resource_dependencies
            .get(key.id())
            .ok_or(ComponentError::Unavailable)?;
        dependency.resolve(requirement)
    }

    pub fn operation<I, O, F, Fut>(
        &self,
        operation: OperationKey<I, O>,
        handler: F,
    ) -> Result<(), ComponentError>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, ComponentError>> + Send + 'static,
    {
        self.operations
            .register(self.participation.clone(), operation, handler)
    }

    pub fn operation_with_context<I, O, F, Fut>(
        &self,
        operation: OperationKey<I, O>,
        handler: F,
    ) -> Result<(), ComponentError>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
        F: Fn(InvocationContext, I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, ComponentError>> + Send + 'static,
    {
        self.operations
            .register_with_context(self.participation.clone(), operation, handler)
    }
}
