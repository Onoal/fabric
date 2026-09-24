use std::any::Any;
use std::collections::BTreeMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use fabric_core::{
    ContractId, ContractKey, ContractRequirement, ContractRequirementDeclaration, Health,
};

use crate::ComponentRelationName;
use crate::component_scope::ComponentScopeService;
use crate::{
    ComponentError, ComponentId, ComponentInstanceBinding, ComponentParticipation, ComponentScope,
    ComponentStatus, InvocationContext, OperationKey, OperationRailService, OperationRegistrar,
    OperationRegistrarService,
};

/// Identifies one preparation contribution that belongs to a ComponentInstanceBinding
/// participation. This is runtime machinery, not a second ComponentInstanceBinding identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentParticipationContribution {
    Base,
    Augmentation { index: usize },
}

/// One occurrence-local cleanup action returned by successful preparation.
///
/// The action is consumed exactly once when the participation is rolled back,
/// dematerialized, or its host stops.
pub type ComponentParticipationCleanup = Box<dyn FnOnce() -> Result<(), ComponentError> + Send>;

/// The result of preparing the base contribution for one ComponentInstanceBinding
/// participation.
pub struct ComponentParticipationPreparation {
    health: Health,
    teardown: Option<ComponentParticipationCleanup>,
}

impl ComponentParticipationPreparation {
    pub fn new(health: Health) -> Self {
        Self {
            health,
            teardown: None,
        }
    }

    pub fn with_teardown(
        health: Health,
        teardown: impl FnOnce() -> Result<(), ComponentError> + Send + 'static,
    ) -> Self {
        Self {
            health,
            teardown: Some(Box::new(teardown)),
        }
    }

    pub fn health(&self) -> Health {
        self.health
    }

    pub(crate) fn into_parts(self) -> (Health, Option<ComponentParticipationCleanup>) {
        (self.health, self.teardown)
    }
}

/// The result of preparing one additive augmentation contribution for a
/// Component participation.
pub struct ComponentAugmentationParticipationPreparation {
    teardown: Option<ComponentParticipationCleanup>,
}

impl ComponentAugmentationParticipationPreparation {
    pub fn new() -> Self {
        Self { teardown: None }
    }

    pub fn with_teardown(
        teardown: impl FnOnce() -> Result<(), ComponentError> + Send + 'static,
    ) -> Self {
        Self {
            teardown: Some(Box::new(teardown)),
        }
    }

    pub(crate) fn into_teardown(self) -> Option<ComponentParticipationCleanup> {
        self.teardown
    }
}

impl Default for ComponentAugmentationParticipationPreparation {
    fn default() -> Self {
        Self::new()
    }
}

const COMPONENT_MATERIALIZER_CONTRACT_ID: &str = "fabric.component.materializer";

/// Instance-local transport for a Core-resolved ComponentInstanceBinding capability dependency.
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
    name: &ComponentRelationName,
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

pub fn component_named_system_dependency_contract_key(
    component_id: &ComponentId,
    name: &ComponentRelationName,
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
        ContractId::new(format!("fabric.component.system-dependency.{encoded}"))
            .expect("encoded component named system dependency contract id"),
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
pub struct ComponentParticipationRealization {
    component_id: ComponentId,
    prepare: Arc<ComponentParticipationPrepareFn>,
}

/// One additive preparation contribution for a declared ComponentInstanceBinding.
///
/// This is deliberately distinct from [`ComponentParticipationRealization`]: a
/// ComponentInstanceBinding has one base runtime attachment, while zero or more externally
/// owned contributions may prepare the same participation without changing its
/// declared operations or intrinsic health.
#[derive(Clone)]
pub struct ComponentAugmentationParticipationRealization {
    component_id: ComponentId,
    prepare: Arc<ComponentAugmentationPrepareFn>,
}

type ComponentAugmentationPrepareFn = dyn Fn(
        &ComponentParticipationScope,
    ) -> Result<ComponentAugmentationParticipationPreparation, ComponentError>
    + Send
    + Sync;

type ComponentParticipationPrepareFn = dyn Fn(&ComponentParticipationScope) -> Result<ComponentParticipationPreparation, ComponentError>
    + Send
    + Sync;

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
pub struct ComponentParticipationScope {
    participation: ComponentParticipation,
    operations: OperationRegistrar,
    component_scope: ComponentScope,
    resource_dependencies: Arc<BTreeMap<ContractId, Arc<ComponentResourceDependency>>>,
}

impl ComponentParticipationRealization {
    pub fn new(
        component_id: ComponentId,
        prepare: impl Fn(&ComponentParticipationScope) -> Result<Health, ComponentError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            component_id,
            prepare: Arc::new(move |scope| {
                prepare(scope).map(ComponentParticipationPreparation::new)
            }),
        }
    }

    /// Defines preparation that returns cleanup ownership for this one
    /// participation. The returned teardown is never shared with another
    /// materialization.
    pub fn new_with_teardown(
        component_id: ComponentId,
        prepare: impl Fn(
            &ComponentParticipationScope,
        ) -> Result<ComponentParticipationPreparation, ComponentError>
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
    pub fn prepare(&self, scope: &ComponentParticipationScope) -> Result<Health, ComponentError> {
        Ok((self.prepare)(scope)?.health())
    }

    /// Prepares this occurrence and returns its cleanup ownership to the
    /// native ComponentInstanceBinding host.
    pub fn prepare_with_teardown(
        &self,
        scope: &ComponentParticipationScope,
    ) -> Result<ComponentParticipationPreparation, ComponentError> {
        (self.prepare)(scope)
    }
}

impl ComponentAugmentationParticipationRealization {
    pub fn new(
        component_id: ComponentId,
        prepare: impl Fn(&ComponentParticipationScope) -> Result<(), ComponentError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            component_id,
            prepare: Arc::new(move |scope| {
                prepare(scope).map(|()| ComponentAugmentationParticipationPreparation::new())
            }),
        }
    }

    /// Defines an additive preparation contribution with occurrence-local
    /// teardown ownership.
    pub fn new_with_teardown(
        component_id: ComponentId,
        prepare: impl Fn(
            &ComponentParticipationScope,
        )
            -> Result<ComponentAugmentationParticipationPreparation, ComponentError>
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

    /// Prepares the same participation already created for the base runtime.
    pub fn prepare(&self, scope: &ComponentParticipationScope) -> Result<(), ComponentError> {
        (self.prepare)(scope).map(|_| ())
    }

    /// Prepares the same participation and returns its additive teardown.
    pub fn prepare_with_teardown(
        &self,
        scope: &ComponentParticipationScope,
    ) -> Result<ComponentAugmentationParticipationPreparation, ComponentError> {
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

impl ComponentParticipationScope {
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

    pub fn component(&self) -> &ComponentInstanceBinding {
        self.participation.component()
    }

    pub fn participation(&self) -> &ComponentParticipation {
        &self.participation
    }

    pub fn component_scope(&self) -> ComponentScope {
        self.component_scope.clone()
    }

    /// Retrieves this preparing ComponentInstanceBinding's already Core-resolved Resource handoff.
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
        name: &ComponentRelationName,
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

    pub fn named_system_dependency<T>(
        &self,
        name: &ComponentRelationName,
        requirement: &ContractRequirement<T>,
    ) -> Result<Arc<T>, ComponentError>
    where
        T: Send + Sync + 'static,
    {
        let key = component_named_system_dependency_contract_key(
            self.component().component_id(),
            name,
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
