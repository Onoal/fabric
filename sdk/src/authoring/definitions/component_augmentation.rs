use std::marker::PhantomData;

use fabric_component::{
    ComponentAugmentationParticipationPreparation, ComponentAugmentationParticipationRealization,
    ComponentError, ComponentParticipationScope,
};
use fabric_core::{
    ContractKey, ContractProviderSelection, ContractRequirement, HostMaterializationRequirement,
    Module, ModuleDeclaration, ModuleId, ModuleRuntime,
};

use super::{
    AdaptableComponentDefinition, AdapterDefinition, ComponentDefinition, ComponentRealization,
    ComponentSpec,
};
use crate::authoring::requirement_for_key;

/// Externally owned semantic meaning attached to one configured ComponentInstanceBinding.
pub trait ComponentAugmentationDefinition<C>: Sized + Send + Sync + 'static
where
    C: ComponentDefinition,
{
    type Config: Clone + Send + Sync + 'static;
    type Contract: Clone + Send + Sync + 'static;
    fn contract_key() -> ContractKey<Self::Contract>;
}

/// Independently authored implementation for ComponentInstanceBinding augmentation `X`.
pub trait ComponentAugmentationSupportDefinition<C, X>: Clone + Send + Sync + 'static
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration;
    fn materialize(
        &self,
        config: &X::Config,
        provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>>;
    fn prepare(
        &self,
        config: &X::Config,
        scope: &ComponentParticipationScope,
    ) -> Result<(), ComponentError>;

    /// Prepares one participation-scoped contribution. Existing support can
    /// keep implementing [`Self::prepare`]; stateful support overrides this
    /// method to return cleanup ownership for this occurrence.
    fn prepare_with_teardown(
        &self,
        config: &X::Config,
        scope: &ComponentParticipationScope,
    ) -> Result<ComponentAugmentationParticipationPreparation, ComponentError> {
        self.prepare(config, scope)?;
        Ok(ComponentAugmentationParticipationPreparation::new())
    }
}

/// One semantic attachment carried by the configured ComponentInstanceBinding contribution.
pub struct ComponentAugmentation<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    pub(crate) component: ComponentSpec<C>,
    config: X::Config,
    module_id: ModuleId,
    marker: PhantomData<X>,
}

impl<C, X> ComponentAugmentation<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    pub fn component_id(&self) -> fabric_component::ComponentId {
        C::component_id()
    }
    pub fn config(&self) -> &X::Config {
        &self.config
    }
    pub fn requirement(&self) -> fabric_core::ContractRequirement<X::Contract> {
        requirement_for_key(&X::contract_key())
    }
    pub fn using<S>(self, support: S) -> ComponentAugmentationRealization<C, X, S>
    where
        S: ComponentAugmentationSupportDefinition<C, X>,
    {
        let provider_module_id = ModuleId::new(format!("{}.augmentation", self.module_id.as_str()))
            .expect("component augmentation provider module id");
        ComponentAugmentationRealization {
            attachment: self,
            support,
            provider_module_id,
        }
    }

    /// Keeps this semantic attachment as declarative truth without claiming a
    /// concrete provider for `X::Contract` exists.
    pub fn into_set(self) -> ComponentAugmentationSet<C> {
        let contract = X::contract_key();
        let component_id = self.component_id();
        ComponentAugmentationSet {
            component: self.component,
            providers: Vec::new(),
            manifest: vec![
                super::super::fabric::ComponentAugmentationManifestEntry::new(
                    contract.id().clone(),
                    contract.identity().clone(),
                    component_id,
                ),
            ],
        }
    }
}

/// One independently supported ComponentInstanceBinding augmentation attachment.
pub struct ComponentAugmentationRealization<C, X, S>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
    S: ComponentAugmentationSupportDefinition<C, X>,
{
    attachment: ComponentAugmentation<C, X>,
    support: S,
    provider_module_id: ModuleId,
}

/// A typed consumer requirement for `X` supplied by one augmentation attached
/// to a specific ComponentInstanceBinding semantic target.
pub struct ComponentAugmentationRequirement<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    requirement: ContractRequirement<X::Contract>,
    target_component_id: fabric_component::ComponentId,
    provider_module_id: ModuleId,
}

impl<C, X> Clone for ComponentAugmentationRequirement<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    fn clone(&self) -> Self {
        Self {
            requirement: self.requirement.clone(),
            target_component_id: self.target_component_id.clone(),
            provider_module_id: self.provider_module_id.clone(),
        }
    }
}

impl<C, X> ComponentAugmentationRequirement<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    pub fn augmentation(&self) -> &ContractRequirement<X::Contract> {
        &self.requirement
    }

    pub fn target_component_id(&self) -> &fabric_component::ComponentId {
        &self.target_component_id
    }

    /// Pins a consumer's `X` requirement to the support provider for this
    /// exact ComponentInstanceBinding augmentation attachment.
    pub fn provider_selection(&self, consumer: ModuleId) -> ContractProviderSelection {
        ContractProviderSelection::new(
            consumer,
            self.requirement.declaration().id().clone(),
            self.provider_module_id.clone(),
        )
    }
}

/// A configured ComponentInstanceBinding carrying one or more independently owned semantic
/// augmentation contributions.
pub struct ComponentAugmentationSet<C>
where
    C: ComponentDefinition,
{
    pub(crate) component: ComponentSpec<C>,
    pub(crate) providers: Vec<Box<dyn Module>>,
    pub(crate) manifest: Vec<super::super::fabric::ComponentAugmentationManifestEntry>,
}

/// One more semantic attachment being appended to a ComponentInstanceBinding augmentation
/// set. It may remain bare or receive an independently authored support.
pub struct ComponentAugmentationSetAttachment<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    attachment: ComponentAugmentation<C, X>,
    providers: Vec<Box<dyn Module>>,
    manifest: Vec<super::super::fabric::ComponentAugmentationManifestEntry>,
}

/// A supported augmentation appended to an existing ComponentInstanceBinding augmentation
/// set. It retains the typed target-bound requirement until the caller either
/// uses it as the ComponentInstanceBinding contribution or continues chaining.
pub struct ComponentAugmentationSetRealization<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    set: ComponentAugmentationSet<C>,
    requirement: ComponentAugmentationRequirement<C, X>,
}

/// A ComponentInstanceBinding augmentation set combined with the ordinary Adapter realization
/// of its base ComponentInstanceBinding.
pub struct ComponentAugmentationSetAdapterRealization<C, A>
where
    C: AdaptableComponentDefinition,
    A: AdapterDefinition<Target = C, Compatibility = fabric_component::ComponentId>,
{
    pub(crate) component: ComponentRealization<C, A>,
    pub(crate) providers: Vec<Box<dyn Module>>,
    pub(crate) manifest: Vec<super::super::fabric::ComponentAugmentationManifestEntry>,
}

/// An augmentation support contribution combined with the ordinary Adapter
/// realization of its base ComponentInstanceBinding.
pub struct ComponentAugmentedAdapterRealization<C, X, S, A>
where
    C: AdaptableComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
    S: ComponentAugmentationSupportDefinition<C, X>,
    A: AdapterDefinition<Target = C, Compatibility = fabric_component::ComponentId>,
{
    pub(crate) component: ComponentRealization<C, A>,
    pub(crate) provider: Box<dyn Module>,
    pub(crate) contract: ContractKey<X::Contract>,
    marker: PhantomData<S>,
}

impl<C, X, S> ComponentAugmentationRealization<C, X, S>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
    S: ComponentAugmentationSupportDefinition<C, X>,
{
    pub fn component_id(&self) -> fabric_component::ComponentId {
        self.attachment.component_id()
    }
    pub fn contract_key(&self) -> ContractKey<X::Contract> {
        X::contract_key()
    }

    /// Creates a typed requirement that remains bound to this supported
    /// attachment's ComponentInstanceBinding target and provider occurrence.
    pub fn requirement(&self) -> ComponentAugmentationRequirement<C, X> {
        ComponentAugmentationRequirement {
            requirement: self.attachment.requirement(),
            target_component_id: self.component_id(),
            provider_module_id: self.provider_module_id.clone(),
        }
    }
    pub fn into_parts(self) -> (ComponentSpec<C>, Box<dyn Module>) {
        let component_id = C::component_id();
        let attachment = self.attachment;
        let support = self.support.clone();
        let config = attachment.config.clone();
        let preparation_config = config.clone();
        let preparation = ComponentAugmentationParticipationRealization::new_with_teardown(
            component_id,
            move |scope| support.prepare_with_teardown(&preparation_config, scope),
        );
        let mut component = attachment.component;
        component.augmentation_preparations.push(preparation);
        (
            component,
            Box::new(ComponentAugmentationProvider {
                support: self.support,
                config,
                module_id: self.provider_module_id,
                marker: PhantomData,
            }),
        )
    }

    /// Converts this supported attachment into a set which can carry more
    /// independently owned ComponentInstanceBinding augmentations.
    pub fn into_set(self) -> ComponentAugmentationSet<C> {
        let contract = self.contract_key();
        let component_id = self.component_id();
        let (component, provider) = self.into_parts();
        ComponentAugmentationSet {
            component,
            providers: vec![provider],
            manifest: vec![
                super::super::fabric::ComponentAugmentationManifestEntry::new(
                    contract.id().clone(),
                    contract.identity().clone(),
                    component_id,
                ),
            ],
        }
    }
}

impl<C> ComponentAugmentationSet<C>
where
    C: ComponentDefinition,
{
    /// Attaches another independently owned semantic contribution to this
    /// same configured ComponentInstanceBinding.
    pub fn augment<X>(
        self,
        config: X::Config,
    ) -> Result<ComponentAugmentationSetAttachment<C, X>, ComponentError>
    where
        X: ComponentAugmentationDefinition<C>,
    {
        Ok(ComponentAugmentationSetAttachment {
            attachment: self.component.augment::<X>(config)?,
            providers: self.providers,
            manifest: self.manifest,
        })
    }
}

impl<C, X> ComponentAugmentationSetAttachment<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    /// Retains this attachment as semantic truth without support.
    pub fn without_support(self) -> ComponentAugmentationSet<C> {
        let mut appended = self.attachment.into_set();
        let mut providers = self.providers;
        providers.append(&mut appended.providers);
        let mut manifest = self.manifest;
        manifest.append(&mut appended.manifest);
        ComponentAugmentationSet {
            component: appended.component,
            providers,
            manifest,
        }
    }

    /// Adds an independently supplied realization support provider for this
    /// semantic attachment.
    pub fn using<S>(self, support: S) -> ComponentAugmentationSetRealization<C, X>
    where
        S: ComponentAugmentationSupportDefinition<C, X>,
    {
        let requirement = self.attachment.using(support);
        let typed_requirement = requirement.requirement();
        let mut appended = requirement.into_set();
        let mut providers = self.providers;
        providers.append(&mut appended.providers);
        let mut manifest = self.manifest;
        manifest.append(&mut appended.manifest);
        ComponentAugmentationSetRealization {
            set: ComponentAugmentationSet {
                component: appended.component,
                providers,
                manifest,
            },
            requirement: typed_requirement,
        }
    }
}

impl<C, X> ComponentAugmentationSetRealization<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    /// Returns the typed requirement pinned to this supported augmentation's
    /// target ComponentInstanceBinding and provider occurrence.
    pub fn requirement(&self) -> ComponentAugmentationRequirement<C, X> {
        self.requirement.clone()
    }

    /// Continues ComponentInstanceBinding augmentation authoring while retaining any copied
    /// typed requirement handles obtained from this value.
    pub fn into_set(self) -> ComponentAugmentationSet<C> {
        self.set
    }
}

impl<C, X, S> ComponentAugmentationRealization<C, X, S>
where
    C: AdaptableComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
    S: ComponentAugmentationSupportDefinition<C, X>,
{
    pub fn using_adapter<A>(
        self,
        adapter: A,
    ) -> Result<ComponentAugmentedAdapterRealization<C, X, S, A>, ComponentError>
    where
        A: AdapterDefinition<Target = C, Compatibility = fabric_component::ComponentId>,
    {
        let contract = self.contract_key();
        let (component, provider) = self.into_parts();
        Ok(ComponentAugmentedAdapterRealization {
            component: component.using(adapter)?,
            provider,
            contract,
            marker: PhantomData,
        })
    }
}

impl<C> ComponentAugmentationSet<C>
where
    C: AdaptableComponentDefinition,
{
    /// Combines the base ComponentInstanceBinding's ordinary Adapter realization with all
    /// already attached ComponentInstanceBinding augmentation contributions.
    pub fn using_adapter<A>(
        self,
        adapter: A,
    ) -> Result<ComponentAugmentationSetAdapterRealization<C, A>, ComponentError>
    where
        A: AdapterDefinition<Target = C, Compatibility = fabric_component::ComponentId>,
    {
        Ok(ComponentAugmentationSetAdapterRealization {
            component: self.component.using(adapter)?,
            providers: self.providers,
            manifest: self.manifest,
        })
    }
}

struct ComponentAugmentationProvider<C, X, S>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
    S: ComponentAugmentationSupportDefinition<C, X>,
{
    support: S,
    config: X::Config,
    module_id: ModuleId,
    marker: PhantomData<(C, X)>,
}

impl<C, X, S> Module for ComponentAugmentationProvider<C, X, S>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
    S: ComponentAugmentationSupportDefinition<C, X>,
{
    fn declaration(&self) -> ModuleDeclaration {
        let d = self.support.declaration(self.module_id.clone());
        let mut provided = d.provided_contracts().to_vec();
        provided.push(X::contract_key().declaration());
        let declaration = ModuleDeclaration::new(self.module_id.clone())
            .with_required_contracts(d.required_contracts().to_vec())
            .with_optional_contracts(d.optional_contracts().to_vec())
            .with_provided_contracts(provided);
        match d.host_requirement() {
            Some(requirement) => {
                declaration.with_host_requirement(HostMaterializationRequirement::new(
                    self.module_id.clone(),
                    requirement.requirement().clone(),
                ))
            }
            None => declaration,
        }
    }
    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        self.support
            .materialize(&self.config, self.module_id.clone())
    }
}

impl<C> ComponentSpec<C>
where
    C: ComponentDefinition,
{
    pub fn augment<X>(
        self,
        config: X::Config,
    ) -> Result<ComponentAugmentation<C, X>, ComponentError>
    where
        X: ComponentAugmentationDefinition<C>,
    {
        let semantic = X::contract_key()
            .id()
            .as_str()
            .bytes()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let module_id = ModuleId::new(format!(
            "fabric.component.{}.augmentation.{semantic}",
            C::component_id().as_str()
        ))
        .map_err(|e| ComponentError::InvalidComponentId(e.to_string()))?;
        Ok(ComponentAugmentation {
            component: self,
            config,
            module_id,
            marker: PhantomData,
        })
    }
}
