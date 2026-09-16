use std::marker::PhantomData;

use fabric_component::{
    ComponentAugmentationRuntimeDefinition, ComponentError, ComponentRuntimeScope,
};
use fabric_core::{ContractKey, Module, ModuleDeclaration, ModuleId, ModuleRuntime};

use super::{
    AdaptableComponentDefinition, AdapterDefinition, ComponentDefinition, ComponentRealization,
    ComponentSpec,
};

/// Externally owned semantic meaning attached to one configured Component.
pub trait ComponentAugmentationDefinition<C>: Sized + Send + Sync + 'static
where
    C: ComponentDefinition,
{
    type Config: Clone + Send + Sync + 'static;
    type Contract: Clone + Send + Sync + 'static;
    fn contract_key() -> ContractKey<Self::Contract>;
}

/// Independently authored implementation for Component augmentation `X`.
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
        scope: &ComponentRuntimeScope,
    ) -> Result<(), ComponentError>;
}

/// One semantic attachment carried by the configured Component contribution.
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
}

/// One independently supported Component augmentation attachment.
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

/// An augmentation support contribution combined with the ordinary Adapter
/// realization of its base Component.
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
    pub fn into_parts(self) -> (ComponentSpec<C>, Box<dyn Module>) {
        let component_id = C::component_id();
        let attachment = self.attachment;
        let support = self.support.clone();
        let config = attachment.config.clone();
        let preparation_config = config.clone();
        let preparation = ComponentAugmentationRuntimeDefinition::new(component_id, move |scope| {
            support.prepare(&preparation_config, scope)
        });
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
        ModuleDeclaration::new(d.module_id().clone())
            .with_required_contracts(d.required_contracts().to_vec())
            .with_optional_contracts(d.optional_contracts().to_vec())
            .with_provided_contracts(provided)
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
        let module_id = ModuleId::new(format!(
            "fabric.component.{}.augmentation",
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
