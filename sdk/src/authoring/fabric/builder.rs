use std::error::Error;
use std::fmt;

use fabric_component::{
    ComponentDeclaration, ComponentRuntimeDefinition, ComponentRuntimeHandle,
    ComponentRuntimeModule, component_runtime_handle_contract_key,
};
use fabric_core::{
    Block, BlockId, Composition, CompositionError, CompositionExport, CompositionId, ContractId,
    ContractProviderSelection, Module, ModuleDeclaration, ModuleRuntime,
};

struct StoredTypedModule {
    inner: Box<dyn Module>,
}

impl StoredTypedModule {
    fn new(inner: Box<dyn Module>) -> Self {
        Self { inner }
    }
}

impl Module for StoredTypedModule {
    fn declaration(&self) -> ModuleDeclaration {
        self.inner.declaration()
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        self.inner.materialize()
    }
}

use super::augmentation::IntoFabricResourceAugmentation;
use super::manifest::{
    ComponentAugmentationManifestEntry, FabricManifest, ResourceAugmentationManifestEntry,
    ResourceManifestEntry, SystemAugmentationManifestEntry, SystemManifestEntry,
};
use super::resource::IntoFabricResource;
use super::system::IntoFabricSystem;
use super::system_augmentation::IntoFabricSystemAugmentation;
use crate::authoring::definitions::ComponentSpecParts;
use crate::authoring::{
    AdaptableComponentDefinition, AdapterDefinition, BlockAuthor, ComponentAugmentation,
    ComponentAugmentationDefinition, ComponentAugmentationRealization, ComponentAugmentationSet,
    ComponentAugmentationSetAdapterRealization, ComponentAugmentationSetRealization,
    ComponentAugmentationSupportDefinition, ComponentAugmentedAdapterRealization,
    ComponentDefinition, ComponentRealization, ComponentSpec, FabricBuilder,
};
use fabric_component::ComponentId;
type FabricComponentContribution = (
    ComponentSpecParts,
    Vec<Box<dyn Module>>,
    Vec<ContractProviderSelection>,
    Vec<ComponentAugmentationManifestEntry>,
);

#[doc(hidden)]
pub trait IntoFabricComponent {
    fn into_fabric_component(self) -> FabricComponentContribution;
}

impl<C> IntoFabricComponent for ComponentSpec<C>
where
    C: ComponentDefinition,
{
    fn into_fabric_component(
        self,
    ) -> (
        ComponentSpecParts,
        Vec<Box<dyn Module>>,
        Vec<ContractProviderSelection>,
        Vec<ComponentAugmentationManifestEntry>,
    ) {
        (self.into_parts(), Vec::new(), Vec::new(), Vec::new())
    }
}

impl<C, A> IntoFabricComponent for ComponentRealization<C, A>
where
    C: AdaptableComponentDefinition,
    A: AdapterDefinition<Target = C>,
    A::Compatibility: crate::authoring::ComponentAdapterCompatibility<C>,
{
    fn into_fabric_component(
        self,
    ) -> (
        ComponentSpecParts,
        Vec<Box<dyn Module>>,
        Vec<ContractProviderSelection>,
        Vec<ComponentAugmentationManifestEntry>,
    ) {
        let (component, adapter, bridge, selection) = self.into_parts();
        (
            component.into_parts(),
            vec![Box::new(adapter), Box::new(bridge)],
            vec![selection],
            Vec::new(),
        )
    }
}

impl<C, X> IntoFabricComponent for ComponentAugmentation<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    fn into_fabric_component(self) -> FabricComponentContribution {
        let contract = X::contract_key();
        let component_id = self.component_id();
        (
            self.component.into_parts(),
            Vec::new(),
            Vec::new(),
            vec![ComponentAugmentationManifestEntry::new(
                contract.id().clone(),
                contract.identity().clone(),
                component_id,
            )],
        )
    }
}

impl<C, X, S> IntoFabricComponent for ComponentAugmentationRealization<C, X, S>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
    S: ComponentAugmentationSupportDefinition<C, X>,
{
    fn into_fabric_component(
        self,
    ) -> (
        ComponentSpecParts,
        Vec<Box<dyn Module>>,
        Vec<ContractProviderSelection>,
        Vec<ComponentAugmentationManifestEntry>,
    ) {
        let contract = self.contract_key();
        let component_id = self.component_id();
        let (component, provider) = self.into_parts();
        (
            component.into_parts(),
            vec![provider],
            Vec::new(),
            vec![ComponentAugmentationManifestEntry::new(
                contract.id().clone(),
                contract.identity().clone(),
                component_id,
            )],
        )
    }
}

impl<C> IntoFabricComponent for ComponentAugmentationSet<C>
where
    C: ComponentDefinition,
{
    fn into_fabric_component(self) -> FabricComponentContribution {
        (
            self.component.into_parts(),
            self.providers,
            Vec::new(),
            self.manifest,
        )
    }
}

impl<C, X> IntoFabricComponent for ComponentAugmentationSetRealization<C, X>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
{
    fn into_fabric_component(self) -> FabricComponentContribution {
        self.into_set().into_fabric_component()
    }
}

impl<C, X, S, A> IntoFabricComponent for ComponentAugmentedAdapterRealization<C, X, S, A>
where
    C: AdaptableComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
    S: ComponentAugmentationSupportDefinition<C, X>,
    A: AdapterDefinition<Target = C, Compatibility = ComponentId>,
{
    fn into_fabric_component(
        self,
    ) -> (
        ComponentSpecParts,
        Vec<Box<dyn Module>>,
        Vec<ContractProviderSelection>,
        Vec<ComponentAugmentationManifestEntry>,
    ) {
        let component_id = C::component_id();
        let contract = self.contract;
        let (component, adapter, bridge, selection) = self.component.into_parts();
        (
            component.into_parts(),
            vec![Box::new(adapter), Box::new(bridge), self.provider],
            vec![selection],
            vec![ComponentAugmentationManifestEntry::new(
                contract.id().clone(),
                contract.identity().clone(),
                component_id,
            )],
        )
    }
}

impl<C, A> IntoFabricComponent for ComponentAugmentationSetAdapterRealization<C, A>
where
    C: AdaptableComponentDefinition,
    A: AdapterDefinition<Target = C, Compatibility = ComponentId>,
{
    fn into_fabric_component(self) -> FabricComponentContribution {
        let (component, adapter, bridge, selection) = self.component.into_parts();
        let mut modules: Vec<Box<dyn Module>> = vec![Box::new(adapter), Box::new(bridge)];
        modules.extend(self.providers);
        (
            component.into_parts(),
            modules,
            vec![selection],
            self.manifest,
        )
    }
}
use crate::ids::{IntoBlockId, IntoCompositionId};

const DEFAULT_BLOCK_ID: &str = "fabric.sdk.default";
const COMPONENT_RUNTIME_EXPORT_ID: &str = "fabric.sdk.export.component-runtime";

#[derive(Debug)]
pub enum FabricBuildError {
    Component(fabric_component::ComponentError),
    Composition(CompositionError),
}

impl fmt::Display for FabricBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Component(error) => error.fmt(f),
            Self::Composition(error) => error.fmt(f),
        }
    }
}

impl Error for FabricBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Component(error) => Some(error),
            Self::Composition(error) => Some(error),
        }
    }
}

impl From<fabric_component::ComponentError> for FabricBuildError {
    fn from(value: fabric_component::ComponentError) -> Self {
        Self::Component(value)
    }
}

impl From<CompositionError> for FabricBuildError {
    fn from(value: CompositionError) -> Self {
        Self::Composition(value)
    }
}

#[derive(Debug)]
pub struct BuiltFabric {
    composition: Composition,
    manifest: FabricManifest,
    component_runtime_export: Option<CompositionExport<ComponentRuntimeHandle>>,
}

impl BuiltFabric {
    pub fn composition(&self) -> &Composition {
        &self.composition
    }

    pub fn manifest(&self) -> &FabricManifest {
        &self.manifest
    }

    pub fn into_composition(self) -> Composition {
        self.composition
    }

    pub fn into_parts(self) -> (Composition, FabricManifest) {
        (self.composition, self.manifest)
    }

    pub(crate) fn component_runtime_export(
        &self,
    ) -> Option<&CompositionExport<ComponentRuntimeHandle>> {
        self.component_runtime_export.as_ref()
    }
}

pub struct Fabric {
    composition_id: CompositionId,
    blocks: Vec<Block>,
    raw_block_ids: Vec<BlockId>,
    provider_selections: Vec<ContractProviderSelection>,
    component_resource_provider_selections:
        Vec<super::manifest::ComponentResourceBindingManifestEntry>,
    component_system_provider_selections: Vec<super::manifest::ComponentSystemBindingManifestEntry>,
    resources: Vec<ResourceManifestEntry>,
    resource_augmentations: Vec<ResourceAugmentationManifestEntry>,
    systems: Vec<SystemManifestEntry>,
    system_augmentations: Vec<SystemAugmentationManifestEntry>,
    component_augmentations: Vec<ComponentAugmentationManifestEntry>,
    components: Vec<ComponentDeclaration>,
    module_declarations: Vec<ModuleDeclaration>,
    typed_modules: Vec<Box<dyn Module>>,
    component_declarations: Vec<ComponentDeclaration>,
    component_self_realizations: Vec<ComponentRuntimeDefinition>,
    component_augmentation_preparations:
        Vec<fabric_component::ComponentAugmentationRuntimeDefinition>,
}

impl Fabric {
    pub fn new(composition_id: impl IntoCompositionId) -> Result<Self, CompositionError> {
        Ok(Self::from_id(composition_id.into_composition_id()?))
    }

    pub fn from_id(composition_id: CompositionId) -> Self {
        Self {
            composition_id,
            blocks: Vec::new(),
            raw_block_ids: Vec::new(),
            provider_selections: Vec::new(),
            component_resource_provider_selections: Vec::new(),
            component_system_provider_selections: Vec::new(),
            resources: Vec::new(),
            resource_augmentations: Vec::new(),
            systems: Vec::new(),
            system_augmentations: Vec::new(),
            component_augmentations: Vec::new(),
            components: Vec::new(),
            module_declarations: Vec::new(),
            typed_modules: Vec::new(),
            component_declarations: Vec::new(),
            component_self_realizations: Vec::new(),
            component_augmentation_preparations: Vec::new(),
        }
    }

    pub fn resource(mut self, resource: impl IntoFabricResource) -> Self {
        let contribution = resource.into_fabric_resource();
        self.resources.push(contribution.entry().clone());
        self.module_declarations
            .extend(contribution.declarations().iter().cloned());
        self.provider_selections
            .extend(contribution.provider_selections().iter().cloned());
        self.typed_modules.extend(contribution.modules());
        self
    }

    /// Adds externally owned semantic meaning to one selected Resource occurrence.
    pub fn resource_augmentation(
        mut self,
        augmentation: impl IntoFabricResourceAugmentation,
    ) -> Self {
        let contribution = augmentation.into_fabric_resource_augmentation();
        self.resource_augmentations
            .push(contribution.entry().clone());
        let (modules, declarations, selections) = contribution.into_parts();
        self.typed_modules.extend(modules);
        self.module_declarations.extend(declarations);
        self.provider_selections.extend(selections);
        self
    }

    pub fn system(mut self, system: impl IntoFabricSystem) -> Self {
        let contribution = system.into_fabric_system();
        self.systems.push(contribution.entry().clone());
        self.module_declarations
            .extend(contribution.declarations().iter().cloned());
        self.provider_selections
            .extend(contribution.provider_selections().iter().cloned());
        self.typed_modules.extend(contribution.modules());
        self
    }

    /// Adds externally owned semantic meaning to one selected System occurrence.
    pub fn system_augmentation(mut self, augmentation: impl IntoFabricSystemAugmentation) -> Self {
        let contribution = augmentation.into_fabric_system_augmentation();
        self.system_augmentations.push(contribution.entry().clone());
        let (modules, declarations, selections) = contribution.into_parts();
        self.typed_modules.extend(modules);
        self.module_declarations.extend(declarations);
        self.provider_selections.extend(selections);
        self
    }

    pub fn component(mut self, component: impl IntoFabricComponent) -> Self {
        let (parts, modules, selections, augmentations) = component.into_fabric_component();
        self.component_augmentations.extend(augmentations);
        self.components.push(parts.declaration.clone());
        self.component_declarations.push(parts.declaration);
        self.component_augmentation_preparations
            .extend(parts.augmentation_preparations);
        self.typed_modules.extend(parts.carriers);
        self.typed_modules.extend(modules);
        self.provider_selections.extend(selections);
        self.provider_selections.extend(parts.provider_selections);
        self.component_resource_provider_selections
            .extend(parts.semantic_provider_selections);
        self.component_system_provider_selections
            .extend(parts.semantic_system_provider_selections);
        if let Some(self_realization) = parts.self_realization {
            self.component_self_realizations.push(self_realization);
        }
        self
    }

    pub fn with_block(mut self, block: Block) -> Self {
        self.raw_block_ids.push(block.id().clone());
        self.blocks.push(block);
        self
    }

    pub fn block(
        self,
        block_id: impl IntoBlockId,
        configure: impl FnOnce(BlockAuthor) -> BlockAuthor,
    ) -> Result<Self, CompositionError> {
        let block = configure(BlockAuthor::new(block_id)?).build();
        Ok(self.with_block(block))
    }

    pub fn select_provider(mut self, selection: ContractProviderSelection) -> Self {
        self.provider_selections.push(selection);
        self
    }

    pub fn build(self) -> Result<BuiltFabric, FabricBuildError> {
        let Self {
            composition_id,
            blocks,
            raw_block_ids,
            provider_selections,
            component_resource_provider_selections,
            component_system_provider_selections,
            resources,
            resource_augmentations,
            systems,
            system_augmentations,
            component_augmentations,
            components,
            mut module_declarations,
            typed_modules,
            component_declarations,
            component_self_realizations,
            component_augmentation_preparations,
        } = self;

        let mut default_modules = typed_modules;
        // The native host carries every composed Component declaration, even
        // when no local self realization exists. Declaration-only Components are
        // host-known without any fake runtime behavior.
        let component_runtime_export = if !component_declarations.is_empty() {
            let native_module = ComponentRuntimeModule::with_components_and_augmentations(
                component_declarations,
                component_self_realizations,
                component_augmentation_preparations,
            )?;
            module_declarations.push(native_module.declaration());
            default_modules.push(Box::new(native_module));
            Some(CompositionExport::new(
                ContractId::new(COMPONENT_RUNTIME_EXPORT_ID).expect("static export id"),
                fabric_core::ContractRequirement::provisional(
                    component_runtime_handle_contract_key().id().clone(),
                ),
            ))
        } else {
            None
        };

        let mut builder = FabricBuilder::from_id(composition_id);
        // The deterministic default Block is a grouping choice only. Core
        // binds, initializes, and starts runtime modules in resolved
        // dependency order regardless of Block or contribution order.
        if !default_modules.is_empty() {
            let mut default_block = BlockAuthor::from_id(default_block_id());
            for module in default_modules {
                default_block = default_block.module(StoredTypedModule::new(module));
            }
            builder = builder.with_block(default_block.build());
        }
        for block in blocks {
            builder = builder.with_block(block);
        }
        for selection in &provider_selections {
            builder = builder.select_provider(selection.clone());
        }
        if let Some(export) = &component_runtime_export {
            builder = builder.export(export.clone());
        }
        let composition = builder.build()?;

        let manifest = FabricManifest::new(
            resources,
            resource_augmentations,
            systems,
            system_augmentations,
            component_augmentations,
            components,
            module_declarations,
            provider_selections,
            component_resource_provider_selections,
            component_system_provider_selections,
            raw_block_ids,
            composition.exports().to_vec(),
        );
        Ok(BuiltFabric {
            composition,
            manifest,
            component_runtime_export,
        })
    }
}

pub(crate) fn default_block_id() -> fabric_core::BlockId {
    fabric_core::BlockId::new(DEFAULT_BLOCK_ID).expect("static high-level fabric default block id")
}
