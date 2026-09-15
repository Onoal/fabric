use std::sync::Arc;

use super::{PrimaryResourceContract, Requires, ResourceSelection};
use crate::authoring::{
    ComponentResourceBindingManifestEntry, ComponentSystemBindingManifestEntry,
    PrimarySystemContract, SystemRequires, SystemSelection,
};
use fabric_component::{
    ComponentDeclaration, ComponentError, ComponentId, ComponentResourceDependency,
    ComponentResourceRequirementDeclaration, ComponentResourceRequirementName,
    ComponentRuntimeDefinition, ComponentSystemRequirementDeclaration,
    component_named_resource_dependency_contract_key, component_system_dependency_contract_key,
};
use fabric_core::{
    ContractProviderSelection, Health, Module, ModuleBindings, ModuleContract, ModuleDeclaration,
    ModuleError, ModuleId, ModuleRuntime,
};

/// Typed Resource access for a Component self realization. The requirement
/// is declaration truth owned by the ComponentSpec and has already been
/// resolved by Core before this scope is made available.
pub trait ComponentResourceScope {
    fn resource<R>(
        &self,
        requirement: &Requires<R>,
    ) -> Result<Arc<R::Contract>, fabric_component::ComponentError>
    where
        R: PrimaryResourceContract;

    fn named_resource<R>(
        &self,
        requirement: &ComponentResourceRequirement<R>,
    ) -> Result<Arc<R::Contract>, fabric_component::ComponentError>
    where
        R: PrimaryResourceContract;
}

impl ComponentResourceScope for fabric_component::ComponentRuntimeScope {
    fn resource<R>(
        &self,
        requirement: &Requires<R>,
    ) -> Result<Arc<R::Contract>, fabric_component::ComponentError>
    where
        R: PrimaryResourceContract,
    {
        let name = ComponentResourceRequirementName::new(requirement.declaration().id().as_str())
            .expect("contract ids are valid default Component requirement names");
        self.named_resource_dependency(&name, requirement.as_contract_requirement())
    }

    fn named_resource<R>(
        &self,
        requirement: &ComponentResourceRequirement<R>,
    ) -> Result<Arc<R::Contract>, fabric_component::ComponentError>
    where
        R: PrimaryResourceContract,
    {
        self.named_resource_dependency(
            requirement.name(),
            requirement.requirement().as_contract_requirement(),
        )
    }
}

pub struct ComponentResourceRequirement<R: PrimaryResourceContract> {
    name: ComponentResourceRequirementName,
    requirement: Requires<R>,
}

impl<R: PrimaryResourceContract> Clone for ComponentResourceRequirement<R> {
    fn clone(&self) -> Self {
        Self::new(self.name.clone(), self.requirement.clone())
    }
}

impl<R: PrimaryResourceContract> ComponentResourceRequirement<R> {
    pub fn new(name: ComponentResourceRequirementName, requirement: Requires<R>) -> Self {
        Self { name, requirement }
    }
    pub fn name(&self) -> &ComponentResourceRequirementName {
        &self.name
    }
    pub fn requirement(&self) -> &Requires<R> {
        &self.requirement
    }
}

pub trait ComponentSystemScope {
    fn system<S>(
        &self,
        requirement: &SystemRequires<S>,
    ) -> Result<Arc<S::Contract>, fabric_component::ComponentError>
    where
        S: PrimarySystemContract;
}
impl ComponentSystemScope for fabric_component::ComponentRuntimeScope {
    fn system<S>(
        &self,
        requirement: &SystemRequires<S>,
    ) -> Result<Arc<S::Contract>, fabric_component::ComponentError>
    where
        S: PrimarySystemContract,
    {
        self.system_dependency(requirement.as_contract_requirement())
    }
}

trait ComponentResourceContribution: Send + Sync {
    fn declaration(&self) -> ComponentResourceRequirementDeclaration;
    fn carrier(&self, component_id: &ComponentId) -> Box<dyn Module>;
    fn selection(
        &self,
        component_id: &ComponentId,
        provider: ModuleId,
    ) -> ContractProviderSelection;
}

struct TypedComponentResourceContribution<R: PrimaryResourceContract> {
    requirement: ComponentResourceRequirement<R>,
}

trait ComponentSystemContribution: Send + Sync {
    fn declaration(&self) -> ComponentSystemRequirementDeclaration;
    fn carrier(&self, component_id: &ComponentId) -> Box<dyn Module>;
    fn selection(
        &self,
        component_id: &ComponentId,
        provider: ModuleId,
    ) -> ContractProviderSelection;
}
struct TypedComponentSystemContribution<S: PrimarySystemContract> {
    requirement: SystemRequires<S>,
}
impl<S: PrimarySystemContract> ComponentSystemContribution for TypedComponentSystemContribution<S> {
    fn declaration(&self) -> ComponentSystemRequirementDeclaration {
        ComponentSystemRequirementDeclaration::new(
            S::system_id(),
            self.requirement.declaration().clone(),
        )
    }
    fn carrier(&self, component_id: &ComponentId) -> Box<dyn Module> {
        Box::new(ComponentSystemCarrier::<S> {
            component_id: component_id.clone(),
            requirement: self.requirement.clone(),
        })
    }
    fn selection(
        &self,
        component_id: &ComponentId,
        provider: ModuleId,
    ) -> ContractProviderSelection {
        ContractProviderSelection::new(
            component_system_carrier_module_id(component_id, self.requirement.declaration()),
            self.requirement.declaration().id().clone(),
            provider,
        )
    }
}
struct ComponentSystemCarrier<S: PrimarySystemContract> {
    component_id: ComponentId,
    requirement: SystemRequires<S>,
}
struct ComponentSystemCarrierRuntime<S: PrimarySystemContract> {
    module_id: ModuleId,
    component_id: ComponentId,
    requirement: SystemRequires<S>,
    handoff: Arc<ComponentResourceDependency>,
}
impl<S: PrimarySystemContract> Module for ComponentSystemCarrier<S> {
    fn declaration(&self) -> ModuleDeclaration {
        let h = component_system_dependency_contract_key(
            &self.component_id,
            self.requirement.declaration(),
        );
        ModuleDeclaration::new(component_system_carrier_module_id(
            &self.component_id,
            self.requirement.declaration(),
        ))
        .with_required_contracts(vec![self.requirement.declaration().clone()])
        .with_provided_contracts(vec![h.declaration()])
    }
    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(ComponentSystemCarrierRuntime::<S> {
            module_id: component_system_carrier_module_id(
                &self.component_id,
                self.requirement.declaration(),
            ),
            component_id: self.component_id.clone(),
            requirement: self.requirement.clone(),
            handoff: Arc::new(ComponentResourceDependency::new::<S::Contract>(
                self.component_id.clone(),
                self.requirement.declaration().clone(),
            )),
        }))
    }
}
impl<S: PrimarySystemContract> ModuleRuntime for ComponentSystemCarrierRuntime<S> {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }
    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![
            component_system_dependency_contract_key(
                &self.component_id,
                self.requirement.declaration(),
            )
            .declaration(),
        ]
    }
    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &component_system_dependency_contract_key(
                &self.component_id,
                self.requirement.declaration(),
            ),
            self.handoff.clone(),
        )])
    }
    fn bind(&mut self, b: &ModuleBindings) -> Result<(), ModuleError> {
        self.handoff.bind(
            self.requirement
                .resolve(b)
                .map_err(|e| ModuleError::new(e.to_string()))?,
        );
        Ok(())
    }
    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn stop(&mut self) {}
    fn health(&self) -> Health {
        Health::Healthy
    }
}
fn component_system_carrier_module_id(
    component_id: &ComponentId,
    requirement: &fabric_core::ContractRequirementDeclaration,
) -> ModuleId {
    let encoded = format!(
        "{}:{}:{}",
        component_id.as_str(),
        requirement.id().as_str(),
        requirement.compatibility()
    )
    .as_bytes()
    .iter()
    .flat_map(|b| [nibble(b >> 4), nibble(b & 15)])
    .collect::<String>();
    ModuleId::new(format!("fabric.component.system-carrier.{encoded}")).expect("system carrier id")
}

impl<R: PrimaryResourceContract> ComponentResourceContribution
    for TypedComponentResourceContribution<R>
{
    fn declaration(&self) -> ComponentResourceRequirementDeclaration {
        ComponentResourceRequirementDeclaration::new(
            self.requirement.name().clone(),
            R::resource_id(),
            self.requirement.requirement().declaration().clone(),
        )
    }
    fn carrier(&self, component_id: &ComponentId) -> Box<dyn Module> {
        Box::new(ComponentResourceCarrier::<R> {
            component_id: component_id.clone(),
            requirement: self.requirement.clone(),
        })
    }
    fn selection(
        &self,
        component_id: &ComponentId,
        provider: ModuleId,
    ) -> ContractProviderSelection {
        ContractProviderSelection::new(
            component_resource_carrier_module_id(
                component_id,
                self.requirement.name(),
                self.requirement.requirement().declaration(),
            ),
            self.requirement.requirement().declaration().id().clone(),
            provider,
        )
    }
}

struct ComponentResourceCarrier<R: PrimaryResourceContract> {
    component_id: ComponentId,
    requirement: ComponentResourceRequirement<R>,
}
struct ComponentResourceCarrierRuntime<R: PrimaryResourceContract> {
    module_id: ModuleId,
    component_id: ComponentId,
    requirement: ComponentResourceRequirement<R>,
    handoff: Arc<ComponentResourceDependency>,
}

impl<R: PrimaryResourceContract> Module for ComponentResourceCarrier<R> {
    fn declaration(&self) -> ModuleDeclaration {
        let handoff = component_named_resource_dependency_contract_key(
            &self.component_id,
            self.requirement.name(),
            self.requirement.requirement().declaration(),
        );
        ModuleDeclaration::new(component_resource_carrier_module_id(
            &self.component_id,
            self.requirement.name(),
            self.requirement.requirement().declaration(),
        ))
        .with_required_contracts(vec![self.requirement.requirement().declaration().clone()])
        .with_provided_contracts(vec![handoff.declaration()])
    }
    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        let module_id = component_resource_carrier_module_id(
            &self.component_id,
            self.requirement.name(),
            self.requirement.requirement().declaration(),
        );
        Some(Box::new(ComponentResourceCarrierRuntime::<R> {
            module_id,
            component_id: self.component_id.clone(),
            requirement: self.requirement.clone(),
            handoff: Arc::new(ComponentResourceDependency::new::<R::Contract>(
                self.component_id.clone(),
                self.requirement.requirement().declaration().clone(),
            )),
        }))
    }
}

impl<R: PrimaryResourceContract> ModuleRuntime for ComponentResourceCarrierRuntime<R> {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }
    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![
            component_named_resource_dependency_contract_key(
                &self.component_id,
                self.requirement.name(),
                self.requirement.requirement().declaration(),
            )
            .declaration(),
        ]
    }
    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.requirement.requirement().declaration().clone()]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &component_named_resource_dependency_contract_key(
                &self.component_id,
                self.requirement.name(),
                self.requirement.requirement().declaration(),
            ),
            self.handoff.clone(),
        )])
    }
    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.handoff.bind(
            self.requirement
                .requirement()
                .resolve(bindings)
                .map_err(|error| ModuleError::new(error.to_string()))?,
        );
        Ok(())
    }
    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn stop(&mut self) {}
    fn health(&self) -> Health {
        Health::Healthy
    }
}

fn component_resource_carrier_module_id(
    component_id: &ComponentId,
    name: &ComponentResourceRequirementName,
    requirement: &fabric_core::ContractRequirementDeclaration,
) -> ModuleId {
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
    ModuleId::new(format!("fabric.component.resource-carrier.{encoded}"))
        .expect("encoded component carrier module id")
}
fn nibble(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        _ => char::from(b'a' + value - 10),
    }
}

pub struct ComponentSpec<C>
where
    C: ComponentDefinition,
{
    config: C::Config,
    declaration: ComponentDeclaration,
    self_realization: Option<ComponentRuntimeDefinition>,
    resource_contributions: Vec<Box<dyn ComponentResourceContribution>>,
    system_contributions: Vec<Box<dyn ComponentSystemContribution>>,
    provider_selections: Vec<ContractProviderSelection>,
    semantic_provider_selections: Vec<ComponentResourceBindingManifestEntry>,
    semantic_system_provider_selections: Vec<ComponentSystemBindingManifestEntry>,
}

pub(crate) struct ComponentSpecParts {
    pub(crate) declaration: ComponentDeclaration,
    pub(crate) self_realization: Option<ComponentRuntimeDefinition>,
    pub(crate) carriers: Vec<Box<dyn Module>>,
    pub(crate) provider_selections: Vec<ContractProviderSelection>,
    pub(crate) semantic_provider_selections: Vec<ComponentResourceBindingManifestEntry>,
    pub(crate) semantic_system_provider_selections: Vec<ComponentSystemBindingManifestEntry>,
}

pub trait ComponentDefinition: Sized + Send + Sync + 'static {
    type Config: Clone + Send + Sync + 'static;

    fn component_id() -> ComponentId;

    fn declaration() -> ComponentDeclaration;

    fn define(config: Self::Config) -> ComponentSpec<Self> {
        ComponentSpec::<Self>::new(config)
    }
}

pub trait SelfRealizingComponentDefinition: ComponentDefinition {
    fn self_realization(config: &Self::Config) -> ComponentRuntimeDefinition;
}

impl<C> ComponentSpec<C>
where
    C: ComponentDefinition,
{
    pub fn new(config: C::Config) -> Self {
        Self {
            self_realization: None,
            declaration: C::declaration(),
            config,
            resource_contributions: Vec::new(),
            system_contributions: Vec::new(),
            provider_selections: Vec::new(),
            semantic_provider_selections: Vec::new(),
            semantic_system_provider_selections: Vec::new(),
        }
    }

    pub fn declaration_only(config: C::Config) -> Self {
        Self::new(config)
    }

    pub fn self_realizing(config: C::Config) -> Result<Self, ComponentError>
    where
        C: SelfRealizingComponentDefinition,
    {
        let realization = C::self_realization(&config);
        if realization.component_id() != &C::component_id() {
            return Err(ComponentError::DuplicateComponentRuntimeDefinition(
                realization.component_id().clone(),
            ));
        }
        Ok(Self {
            self_realization: Some(realization),
            ..Self::new(config)
        })
    }

    pub fn config(&self) -> &C::Config {
        &self.config
    }

    pub fn declaration(&self) -> &ComponentDeclaration {
        &self.declaration
    }

    pub fn into_self_realization(self) -> Option<ComponentRuntimeDefinition> {
        self.self_realization
    }

    pub fn requires_resource<R>(mut self, requirement: Requires<R>) -> Self
    where
        R: PrimaryResourceContract,
    {
        let name = ComponentResourceRequirementName::new(requirement.declaration().id().as_str())
            .expect("contract ids are valid default Component requirement names");
        self = self.requires_named_resource(ComponentResourceRequirement::new(name, requirement));
        self
    }

    pub fn requires_named_resource<R>(
        mut self,
        requirement: ComponentResourceRequirement<R>,
    ) -> Self
    where
        R: PrimaryResourceContract,
    {
        let contribution: Box<dyn ComponentResourceContribution> =
            Box::new(TypedComponentResourceContribution::<R> { requirement });
        self.declaration = self.declaration.clone().with_resource_requirements({
            let mut declarations = self.declaration.resource_requirements().to_vec();
            declarations.push(contribution.declaration());
            declarations
        });
        self.resource_contributions.push(contribution);
        self
    }

    pub fn requires_system<S>(mut self, requirement: SystemRequires<S>) -> Self
    where
        S: PrimarySystemContract,
    {
        let contribution: Box<dyn ComponentSystemContribution> =
            Box::new(TypedComponentSystemContribution::<S> { requirement });
        self.declaration = self.declaration.clone().with_system_requirements({
            let mut values = self.declaration.system_requirements().to_vec();
            values.push(contribution.declaration());
            values
        });
        self.system_contributions.push(contribution);
        self
    }

    pub fn select_system_provider<S>(mut self, provider: &SystemSelection<S>) -> Self
    where
        S: PrimarySystemContract,
    {
        let component_id = self.declaration.component_id().clone();
        let contribution = self
            .system_contributions
            .iter()
            .find(|value| value.declaration().system_id() == &S::system_id())
            .expect("Component System requirement must be declared before selecting a provider");
        self.provider_selections
            .push(contribution.selection(&component_id, provider.module_id().clone()));
        let declaration = contribution.declaration();
        self.semantic_system_provider_selections
            .push(ComponentSystemBindingManifestEntry::new(
                component_id,
                declaration.system_id().clone(),
            ));
        self
    }

    pub fn select_resource_provider<R>(mut self, provider: &ResourceSelection<R>) -> Self
    where
        R: PrimaryResourceContract,
    {
        let component_id = self.declaration.component_id().clone();
        let contribution = self
            .resource_contributions
            .iter()
            .find(|contribution| contribution.declaration().resource_id() == &R::resource_id())
            .expect("Component resource requirement must be declared before selecting a provider");
        self.provider_selections
            .push(contribution.selection(&component_id, provider.module_id().clone()));
        let declaration = contribution.declaration();
        self.semantic_provider_selections
            .push(ComponentResourceBindingManifestEntry::new(
                component_id,
                declaration.name().clone(),
                declaration.resource_id().clone(),
                provider.name().clone(),
            ));
        self
    }

    pub fn select_named_resource_provider<R>(
        mut self,
        requirement: &ComponentResourceRequirement<R>,
        provider: &ResourceSelection<R>,
    ) -> Self
    where
        R: PrimaryResourceContract,
    {
        let component_id = self.declaration.component_id().clone();
        let contribution = self
            .resource_contributions
            .iter()
            .find(|value| value.declaration().name() == requirement.name())
            .expect("Component Resource requirement must be declared before selecting a provider");
        self.provider_selections
            .push(contribution.selection(&component_id, provider.module_id().clone()));
        let declaration = contribution.declaration();
        self.semantic_provider_selections
            .push(ComponentResourceBindingManifestEntry::new(
                component_id,
                declaration.name().clone(),
                declaration.resource_id().clone(),
                provider.name().clone(),
            ));
        self
    }

    pub(crate) fn into_parts(self) -> ComponentSpecParts {
        let component_id = self.declaration.component_id().clone();
        let carriers: Vec<Box<dyn Module>> = self
            .resource_contributions
            .iter()
            .map(|contribution| contribution.carrier(&component_id))
            .collect();
        let mut carriers = carriers;
        carriers.extend(
            self.system_contributions
                .iter()
                .map(|contribution| contribution.carrier(&component_id)),
        );
        ComponentSpecParts {
            declaration: self.declaration,
            self_realization: self.self_realization,
            carriers,
            provider_selections: self.provider_selections,
            semantic_provider_selections: self.semantic_provider_selections,
            semantic_system_provider_selections: self.semantic_system_provider_selections,
        }
    }
}
