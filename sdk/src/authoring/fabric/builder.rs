#![allow(private_interfaces)]
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use fabric_component::{
    ComponentDeclaration, ComponentDesiredState, ComponentHostModule,
    ComponentParticipationRealization, component_host_handle_contract_key,
};
use fabric_core::{
    Block, BlockId, CompositionError, CompositionExport, CompositionId, ContractId,
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
use crate::composition::{
    AdapterRealizationMode, AdapterRealizationProvenance, ComponentAugmentationManifestEntry,
    Composition, FabricManifest, RealizationProvenance, RelationDeclarationProvenance,
    ResourceAugmentationManifestEntry, ResourceManifestEntry, SystemAugmentationManifestEntry,
    SystemManifestEntry, resolved_semantic_relation_bindings,
};
use fabric_component::ComponentId;
type FabricComponentContribution = (
    ComponentSpecParts,
    Vec<Box<dyn Module>>,
    Vec<ContractProviderSelection>,
    Vec<ComponentAugmentationManifestEntry>,
    RealizationProvenance,
);

#[doc(hidden)]
pub trait IntoFabricComponent {
    fn into_fabric_component(self) -> FabricComponentContribution;
}

fn component_spec_provenance<C>(component: &ComponentSpec<C>) -> RealizationProvenance
where
    C: ComponentDefinition,
{
    if component.has_self_realization() {
        RealizationProvenance::SelfRealization {
            runtime_module_id: None,
        }
    } else {
        RealizationProvenance::DeclarationOnly
    }
}

fn component_adapter_provenance<C, A>(
    component: &ComponentRealization<C, A>,
) -> RealizationProvenance
where
    C: AdaptableComponentDefinition,
    A: AdapterDefinition<Target = C>,
    A::Compatibility: crate::authoring::ComponentAdapterCompatibility<C>,
{
    RealizationProvenance::Adapter(AdapterRealizationProvenance {
        definition_id: component.adapter().adapter().adapter_definition_id(),
        mode: AdapterRealizationMode::Direct,
        provider_module_id: component.adapter().provider_module_id().clone(),
        semantic_owner_module_id: None,
    })
}

impl<C> IntoFabricComponent for ComponentSpec<C>
where
    C: ComponentDefinition,
{
    fn into_fabric_component(self) -> FabricComponentContribution {
        let realization = component_spec_provenance(&self);
        (
            self.into_parts(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            realization,
        )
    }
}

impl<C, A> IntoFabricComponent for ComponentRealization<C, A>
where
    C: AdaptableComponentDefinition,
    A: AdapterDefinition<Target = C>,
    A::Compatibility: crate::authoring::ComponentAdapterCompatibility<C>,
{
    fn into_fabric_component(self) -> FabricComponentContribution {
        let realization = component_adapter_provenance(&self);
        let (component, adapter, bridge, selection) = self.into_parts();
        (
            component.into_parts(),
            vec![Box::new(adapter), Box::new(bridge)],
            vec![selection],
            Vec::new(),
            realization,
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
        let realization = component_spec_provenance(&self.component);
        (
            self.component.into_parts(),
            Vec::new(),
            Vec::new(),
            vec![ComponentAugmentationManifestEntry::new(
                contract.id().clone(),
                contract.identity().clone(),
                component_id,
                None,
            )],
            realization,
        )
    }
}

impl<C, X, S> IntoFabricComponent for ComponentAugmentationRealization<C, X, S>
where
    C: ComponentDefinition,
    X: ComponentAugmentationDefinition<C>,
    S: ComponentAugmentationSupportDefinition<C, X>,
{
    fn into_fabric_component(self) -> FabricComponentContribution {
        let contract = self.contract_key();
        let component_id = self.component_id();
        let support_provider_module_id = self.provider_module_id().clone();
        let (component, provider) = self.into_parts();
        let realization = component_spec_provenance(&component);
        (
            component.into_parts(),
            vec![provider],
            Vec::new(),
            vec![ComponentAugmentationManifestEntry::new(
                contract.id().clone(),
                contract.identity().clone(),
                component_id,
                Some(support_provider_module_id),
            )],
            realization,
        )
    }
}

impl<C> IntoFabricComponent for ComponentAugmentationSet<C>
where
    C: ComponentDefinition,
{
    fn into_fabric_component(self) -> FabricComponentContribution {
        let realization = component_spec_provenance(&self.component);
        (
            self.component.into_parts(),
            self.providers,
            Vec::new(),
            self.manifest,
            realization,
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
    fn into_fabric_component(self) -> FabricComponentContribution {
        let component_id = C::component_id();
        let contract = self.contract;
        let realization = component_adapter_provenance(&self.component);
        let (component, adapter, bridge, selection) = self.component.into_parts();
        (
            component.into_parts(),
            vec![Box::new(adapter), Box::new(bridge), self.provider],
            vec![selection],
            vec![ComponentAugmentationManifestEntry::new(
                contract.id().clone(),
                contract.identity().clone(),
                component_id,
                Some(self.support_provider_module_id.clone()),
            )],
            realization,
        )
    }
}

impl<C, A> IntoFabricComponent for ComponentAugmentationSetAdapterRealization<C, A>
where
    C: AdaptableComponentDefinition,
    A: AdapterDefinition<Target = C, Compatibility = ComponentId>,
{
    fn into_fabric_component(self) -> FabricComponentContribution {
        let realization = component_adapter_provenance(&self.component);
        let (component, adapter, bridge, selection) = self.component.into_parts();
        let mut modules: Vec<Box<dyn Module>> = vec![Box::new(adapter), Box::new(bridge)];
        modules.extend(self.providers);
        (
            component.into_parts(),
            modules,
            vec![selection],
            self.manifest,
            realization,
        )
    }
}

use crate::ids::{IntoBlockId, IntoCompositionId};

const DEFAULT_BLOCK_ID: &str = "fabric.sdk.default";
const COMPONENT_RUNTIME_EXPORT_ID: &str = "fabric.sdk.export.component-runtime";

#[derive(Debug)]
pub enum FabricBuildError {
    ComponentInstanceBinding(fabric_component::ComponentError),
    Composition(CompositionError),
    MissingSemanticRelationBinding {
        consumer: fabric_core::ModuleId,
        contract_id: ContractId,
    },
    SemanticRelationRequirementMismatch {
        consumer: fabric_core::ModuleId,
        contract_id: ContractId,
    },
    UnmappedSemanticRelationProvider {
        provider: fabric_core::ModuleId,
        contract_id: ContractId,
    },
    InconsistentSemanticRelationTarget {
        provider: fabric_core::ModuleId,
        contract_id: ContractId,
    },
    DuplicateSemanticProviderModule {
        module_id: fabric_core::ModuleId,
    },
}

impl fmt::Display for FabricBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ComponentInstanceBinding(error) => error.fmt(f),
            Self::Composition(error) => error.fmt(f),
            Self::MissingSemanticRelationBinding {
                consumer,
                contract_id,
            } => write!(
                f,
                "missing resolved Core binding for semantic relation {consumer}/{contract_id}"
            ),
            Self::SemanticRelationRequirementMismatch {
                consumer,
                contract_id,
            } => write!(
                f,
                "resolved Core requirement differs from retained semantic relation {consumer}/{contract_id}"
            ),
            Self::UnmappedSemanticRelationProvider {
                provider,
                contract_id,
            } => write!(
                f,
                "resolved provider {provider} for semantic relation contract {contract_id} has no semantic target provenance"
            ),
            Self::InconsistentSemanticRelationTarget {
                provider,
                contract_id,
            } => write!(
                f,
                "resolved provider {provider} for semantic relation contract {contract_id} does not match the declared semantic target"
            ),
            Self::DuplicateSemanticProviderModule { module_id } => write!(
                f,
                "semantic provider module {module_id} maps to more than one semantic occurrence"
            ),
        }
    }
}

impl Error for FabricBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentInstanceBinding(error) => Some(error),
            Self::Composition(error) => Some(error),
            Self::MissingSemanticRelationBinding { .. }
            | Self::SemanticRelationRequirementMismatch { .. }
            | Self::UnmappedSemanticRelationProvider { .. }
            | Self::InconsistentSemanticRelationTarget { .. }
            | Self::DuplicateSemanticProviderModule { .. } => None,
        }
    }
}

impl From<fabric_component::ComponentError> for FabricBuildError {
    fn from(value: fabric_component::ComponentError) -> Self {
        Self::ComponentInstanceBinding(value)
    }
}

impl From<CompositionError> for FabricBuildError {
    fn from(value: CompositionError) -> Self {
        Self::Composition(value)
    }
}

/// Identity-less reusable organization of Fabric authoring.
///
/// A contribution is not a Composition, participant, package, lifecycle owner,
/// or runtime object. It only records authoring operations that can be merged
/// into a [`Fabric`] before the single final [`Fabric::build`] boundary.
pub struct FabricContribution {
    authoring: FabricAuthoring,
}

impl Default for FabricContribution {
    fn default() -> Self {
        Self::new()
    }
}

impl FabricContribution {
    pub fn new() -> Self {
        Self {
            authoring: FabricAuthoring::new(),
        }
    }

    pub fn with(mut self, contribution: impl IntoFabricContribution) -> Self {
        self.authoring
            .merge(contribution.into_fabric_contribution().authoring);
        self
    }

    pub fn resource(mut self, resource: impl IntoFabricResource) -> Self {
        self.authoring.add_resource(resource);
        self
    }

    /// Adds externally owned semantic meaning to one selected Resource occurrence.
    pub fn resource_augmentation(
        mut self,
        augmentation: impl IntoFabricResourceAugmentation,
    ) -> Self {
        self.authoring.add_resource_augmentation(augmentation);
        self
    }

    pub fn system(mut self, system: impl IntoFabricSystem) -> Self {
        self.authoring.add_system(system);
        self
    }

    /// Adds externally owned semantic meaning to one selected System occurrence.
    pub fn system_augmentation(mut self, augmentation: impl IntoFabricSystemAugmentation) -> Self {
        self.authoring.add_system_augmentation(augmentation);
        self
    }

    pub fn component(mut self, component: impl IntoFabricComponent) -> Self {
        self.authoring.add_component(component);
        self
    }

    pub fn with_block(mut self, block: Block) -> Self {
        self.authoring.add_block(block);
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
        self.authoring.add_provider_selection(selection);
        self
    }
}

/// Converts reusable authoring into an identity-less Fabric contribution.
pub trait IntoFabricContribution {
    fn into_fabric_contribution(self) -> FabricContribution;
}

impl IntoFabricContribution for FabricContribution {
    fn into_fabric_contribution(self) -> FabricContribution {
        self
    }
}

struct FabricAuthoring {
    blocks: Vec<Block>,
    raw_block_ids: Vec<BlockId>,
    provider_selections: Vec<ContractProviderSelection>,
    component_resource_provider_selections:
        Vec<crate::composition::ComponentResourceBindingManifestEntry>,
    component_system_provider_selections:
        Vec<crate::composition::ComponentSystemBindingManifestEntry>,
    resources: Vec<ResourceManifestEntry>,
    resource_augmentations: Vec<ResourceAugmentationManifestEntry>,
    systems: Vec<SystemManifestEntry>,
    system_augmentations: Vec<SystemAugmentationManifestEntry>,
    component_augmentations: Vec<ComponentAugmentationManifestEntry>,
    relation_declarations: Vec<RelationDeclarationProvenance>,
    components: Vec<ComponentDeclaration>,
    component_realizations: BTreeMap<ComponentId, RealizationProvenance>,
    component_initial_participation: BTreeMap<ComponentId, ComponentDesiredState>,
    module_declarations: Vec<ModuleDeclaration>,
    typed_modules: Vec<Box<dyn Module>>,
    component_declarations: Vec<ComponentDeclaration>,
    component_self_realizations: Vec<ComponentParticipationRealization>,
    component_augmentation_preparations:
        Vec<fabric_component::ComponentAugmentationParticipationRealization>,
}

impl FabricAuthoring {
    fn new() -> Self {
        Self {
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
            relation_declarations: Vec::new(),
            components: Vec::new(),
            component_realizations: BTreeMap::new(),
            component_initial_participation: BTreeMap::new(),
            module_declarations: Vec::new(),
            typed_modules: Vec::new(),
            component_declarations: Vec::new(),
            component_self_realizations: Vec::new(),
            component_augmentation_preparations: Vec::new(),
        }
    }

    fn merge(&mut self, other: Self) {
        self.blocks.extend(other.blocks);
        self.raw_block_ids.extend(other.raw_block_ids);
        self.provider_selections.extend(other.provider_selections);
        self.component_resource_provider_selections
            .extend(other.component_resource_provider_selections);
        self.component_system_provider_selections
            .extend(other.component_system_provider_selections);
        self.resources.extend(other.resources);
        self.resource_augmentations
            .extend(other.resource_augmentations);
        self.systems.extend(other.systems);
        self.system_augmentations.extend(other.system_augmentations);
        self.component_augmentations
            .extend(other.component_augmentations);
        self.relation_declarations
            .extend(other.relation_declarations);
        self.components.extend(other.components);
        self.component_realizations
            .extend(other.component_realizations);
        self.component_initial_participation
            .extend(other.component_initial_participation);
        self.module_declarations.extend(other.module_declarations);
        self.typed_modules.extend(other.typed_modules);
        self.component_declarations
            .extend(other.component_declarations);
        self.component_self_realizations
            .extend(other.component_self_realizations);
        self.component_augmentation_preparations
            .extend(other.component_augmentation_preparations);
    }

    fn add_resource(&mut self, resource: impl IntoFabricResource) {
        let contribution = resource.into_fabric_resource();
        self.resources.push(contribution.entry().clone());
        self.module_declarations
            .extend(contribution.declarations().iter().cloned());
        self.provider_selections
            .extend(contribution.provider_selections().iter().cloned());
        self.relation_declarations
            .extend(contribution.relation_declarations().iter().cloned());
        self.typed_modules.extend(contribution.modules());
    }

    fn add_resource_augmentation(&mut self, augmentation: impl IntoFabricResourceAugmentation) {
        let contribution = augmentation.into_fabric_resource_augmentation();
        self.resource_augmentations
            .push(contribution.entry().clone());
        let (modules, declarations, selections) = contribution.into_parts();
        self.typed_modules.extend(modules);
        self.module_declarations.extend(declarations);
        self.provider_selections.extend(selections);
    }

    fn add_system(&mut self, system: impl IntoFabricSystem) {
        let contribution = system.into_fabric_system();
        self.systems.push(contribution.entry().clone());
        self.module_declarations
            .extend(contribution.declarations().iter().cloned());
        self.provider_selections
            .extend(contribution.provider_selections().iter().cloned());
        self.relation_declarations
            .extend(contribution.relation_declarations().iter().cloned());
        self.typed_modules.extend(contribution.modules());
    }

    fn add_system_augmentation(&mut self, augmentation: impl IntoFabricSystemAugmentation) {
        let contribution = augmentation.into_fabric_system_augmentation();
        self.system_augmentations.push(contribution.entry().clone());
        let (modules, declarations, selections) = contribution.into_parts();
        self.typed_modules.extend(modules);
        self.module_declarations.extend(declarations);
        self.provider_selections.extend(selections);
    }

    fn add_component(&mut self, component: impl IntoFabricComponent) {
        let (parts, modules, selections, augmentations, realization) =
            component.into_fabric_component();
        self.component_augmentations.extend(augmentations);
        let component_id = parts.declaration.component_id().clone();
        let initial_participation = parts
            .initial_participation
            .unwrap_or_else(|| default_initial_participation(&realization));
        self.components.push(parts.declaration.clone());
        self.component_realizations
            .insert(component_id.clone(), realization);
        self.component_initial_participation
            .insert(component_id, initial_participation);
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
        self.relation_declarations
            .extend(parts.relation_declarations);
        if let Some(self_realization) = parts.self_realization {
            self.component_self_realizations.push(self_realization);
        }
    }

    fn add_block(&mut self, block: Block) {
        self.raw_block_ids.push(block.id().clone());
        self.blocks.push(block);
    }

    fn add_provider_selection(&mut self, selection: ContractProviderSelection) {
        self.provider_selections.push(selection);
    }
}

fn default_initial_participation(realization: &RealizationProvenance) -> ComponentDesiredState {
    match realization {
        RealizationProvenance::DeclarationOnly => ComponentDesiredState::Disabled,
        RealizationProvenance::SelfRealization { .. } | RealizationProvenance::Adapter(_) => {
            ComponentDesiredState::Enabled
        }
    }
}

pub struct Fabric {
    composition_id: CompositionId,
    authoring: FabricAuthoring,
}

impl Fabric {
    pub fn new(composition_id: impl IntoCompositionId) -> Result<Self, CompositionError> {
        Ok(Self::from_id(composition_id.into_composition_id()?))
    }

    pub fn from_id(composition_id: CompositionId) -> Self {
        Self {
            composition_id,
            authoring: FabricAuthoring::new(),
        }
    }

    pub fn with(mut self, contribution: impl IntoFabricContribution) -> Self {
        self.authoring
            .merge(contribution.into_fabric_contribution().authoring);
        self
    }

    pub fn resource(mut self, resource: impl IntoFabricResource) -> Self {
        self.authoring.add_resource(resource);
        self
    }

    /// Adds externally owned semantic meaning to one selected Resource occurrence.
    pub fn resource_augmentation(
        mut self,
        augmentation: impl IntoFabricResourceAugmentation,
    ) -> Self {
        self.authoring.add_resource_augmentation(augmentation);
        self
    }

    pub fn system(mut self, system: impl IntoFabricSystem) -> Self {
        self.authoring.add_system(system);
        self
    }

    /// Adds externally owned semantic meaning to one selected System occurrence.
    pub fn system_augmentation(mut self, augmentation: impl IntoFabricSystemAugmentation) -> Self {
        self.authoring.add_system_augmentation(augmentation);
        self
    }

    pub fn component(mut self, component: impl IntoFabricComponent) -> Self {
        self.authoring.add_component(component);
        self
    }

    pub fn with_block(mut self, block: Block) -> Self {
        self.authoring.add_block(block);
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
        self.authoring.add_provider_selection(selection);
        self
    }

    pub fn build(self) -> Result<Composition, FabricBuildError> {
        let FabricAuthoring {
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
            relation_declarations,
            components,
            mut module_declarations,
            component_realizations,
            component_initial_participation,
            typed_modules,
            component_declarations,
            component_self_realizations,
            component_augmentation_preparations,
        } = self.authoring;
        let composition_id = self.composition_id;

        let mut default_modules = typed_modules;
        // The native host carries every composed Component declaration, even
        // when no local self realization exists. Declaration-only Components are
        // host-known without any fake runtime behavior.
        let component_host_export = if !component_declarations.is_empty() {
            let initial_controls = component_initial_participation
                .iter()
                .map(|(component_id, desired)| (component_id.clone(), *desired))
                .collect::<Vec<_>>();
            let native_module =
                ComponentHostModule::with_components_augmentations_and_initial_controls(
                    component_declarations,
                    component_self_realizations,
                    component_augmentation_preparations,
                    initial_controls,
                )?;
            module_declarations.push(native_module.declaration());
            default_modules.push(Box::new(native_module));
            Some(CompositionExport::new(
                ContractId::new(COMPONENT_RUNTIME_EXPORT_ID).expect("static export id"),
                fabric_core::ContractRequirement::provisional(
                    component_host_handle_contract_key().id().clone(),
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
        if let Some(export) = &component_host_export {
            builder = builder.export(export.clone());
        }
        let composition = builder.build()?;
        let relation_bindings = resolved_semantic_relation_bindings(
            &composition,
            &resources,
            &systems,
            &components,
            &component_realizations,
            &relation_declarations,
        )?;

        let manifest = FabricManifest::new(
            resources,
            resource_augmentations,
            systems,
            system_augmentations,
            component_augmentations,
            components,
            component_realizations,
            component_initial_participation,
            module_declarations,
            provider_selections,
            component_resource_provider_selections,
            component_system_provider_selections,
            relation_declarations,
            relation_bindings,
            raw_block_ids,
            composition.exports().to_vec(),
        );
        Ok(Composition::new(
            composition,
            manifest,
            component_host_export,
        ))
    }
}

pub(crate) fn default_block_id() -> fabric_core::BlockId {
    fabric_core::BlockId::new(DEFAULT_BLOCK_ID).expect("static high-level fabric default block id")
}

#[cfg(test)]
mod realization_provenance_tests {
    use super::*;
    use crate::authoring::{
        RelationTarget, RelationTargetDescriptor, ResourceDefinition, SystemDefinition,
    };
    use crate::composition::RelationDeclarationOwner;
    use fabric_component::ComponentRelationName;

    crate::component! {
        ProvenanceComponent {
            id: "fabric.test.provenance.component";
        }
    }

    crate::component! {
        ProvenanceSelfComponent {
            id: "fabric.test.provenance.self-component";
            runtime { prepare { Ok(()) } }
        }
    }

    crate::component! {
        ProvenanceAdapterComponent {
            id: "fabric.test.provenance.adapter-component";
            api { fn read(&self) -> u64; }
        }
    }

    crate::adapter! {
        ProvenanceComponentAdapter for ProvenanceAdapterComponent {
            id: "test.provenance.component-adapter";
            runtime { fn read(&self) -> u64 { 1 } }
        }
    }

    #[test]
    fn component_lowering_retains_declaration_self_and_adapter_truth() {
        let declaration = Fabric::new("fabric.test.provenance.component-declaration")
            .expect("fabric")
            .component(ProvenanceComponent::define())
            .build()
            .expect("build");
        assert!(matches!(
            declaration
                .manifest()
                .component_realization(&ProvenanceComponent::component_id()),
            Some(RealizationProvenance::DeclarationOnly)
        ));

        let self_realized = Fabric::new("fabric.test.provenance.component-self")
            .expect("fabric")
            .component(ProvenanceSelfComponent::define())
            .build()
            .expect("build");
        assert!(matches!(
            self_realized
                .manifest()
                .component_realization(&ProvenanceSelfComponent::component_id()),
            Some(RealizationProvenance::SelfRealization {
                runtime_module_id: None
            })
        ));

        let adapter = ProvenanceComponentAdapter::new();
        let adapter_id = adapter.adapter_definition_id();
        let adapted = Fabric::new("fabric.test.provenance.component-adapter")
            .expect("fabric")
            .component(
                ProvenanceAdapterComponent::define()
                    .using(adapter)
                    .expect("adapter compatibility"),
            )
            .build()
            .expect("build");
        match adapted
            .manifest()
            .component_realization(&ProvenanceAdapterComponent::component_id())
        {
            Some(RealizationProvenance::Adapter(provenance)) => {
                assert_eq!(provenance.definition_id, adapter_id);
                assert_eq!(provenance.mode, AdapterRealizationMode::Direct);
                assert!(provenance.semantic_owner_module_id.is_none());
            }
            other => panic!("expected Component Adapter provenance, got {other:?}"),
        }
    }

    crate::resource! {
        ProvenanceTargetStore {
            id: "fabric.test.relation-provenance.target-store";
            api { fn get(&self) -> u64; }
        }
    }

    crate::system! {
        ProvenanceTargetSignal {
            id: "fabric.test.relation-provenance.target-signal";
            api { fn now(&self) -> u64; }
        }
    }

    crate::system! {
        ProvenanceAdapterSignal {
            id: "fabric.test.relation-provenance.adapter-signal";
            api { fn now(&self) -> u64; }
        }
    }

    crate::resource! {
        ProvenanceOwnerResource {
            id: "fabric.test.relation-provenance.owner-resource";
            relations {
                requires {
                    store: ProvenanceTargetStore;
                    signal: ProvenanceTargetSignal;
                }
            }
            api { fn read(&self) -> u64; }
            runtime { fn read(&self) -> u64 { self.store.get() + self.signal.now() } }
        }
    }

    crate::resource! {
        ProvenanceMediatedResource {
            id: "fabric.test.relation-provenance.mediated-resource";
            relations { requires { signal: ProvenanceTargetSignal; } }
            api { fn read(&self) -> u64; }
            realization {
                mediate read;
                fn raw_read(&self) -> u64;
            }
            runtime { fn read(&self) -> u64 { self.realization.raw_read() + self.signal.now() } }
        }
    }

    crate::system! {
        ProvenanceOwnerSystem {
            id: "fabric.test.relation-provenance.owner-system";
            relations {
                requires {
                    store: ProvenanceTargetStore;
                    signal: ProvenanceTargetSignal;
                }
            }
            api { fn read(&self) -> u64; }
            runtime { fn read(&self) -> u64 { self.store.get() + self.signal.now() } }
        }
    }

    crate::adapter! {
        ProvenanceResourceAdapter for ProvenanceOwnerResource {
            id: "test.relation-provenance.resource-adapter";
            relations { requires { adapter_signal: ProvenanceAdapterSignal; } }
            runtime { fn read(&self) -> u64 { self.adapter_signal.now() } }
        }
    }

    crate::adapter! {
        ProvenanceMediatedResourceAdapter for ProvenanceMediatedResource {
            id: "test.relation-provenance.mediated-resource-adapter";
            runtime { fn raw_read(&self) -> u64 { 1 } }
        }
    }

    crate::component! {
        ProvenanceRelationComponent {
            id: "fabric.test.relation-provenance.component";
            relations {
                requires {
                    primary_store: ProvenanceTargetStore;
                    primary_signal: ProvenanceTargetSignal;
                    fallback_signal: ProvenanceTargetSignal;
                }
            }
        }
    }

    fn relation<'a>(
        manifest: &'a FabricManifest,
        role: &str,
        owner: impl Fn(&RelationDeclarationOwner) -> bool,
    ) -> &'a RelationDeclarationProvenance {
        let role = ComponentRelationName::new(role).expect("role");
        manifest
            .relation_declarations()
            .iter()
            .find(|relation| relation.role() == &role && owner(relation.owner()))
            .expect("relation provenance")
    }

    #[test]
    fn semantic_relation_declaration_provenance_is_retained_without_materialization() {
        struct PanicSelfResource;
        impl crate::authoring::ResourceDefinition for PanicSelfResource {
            type Config = ();

            fn resource_id() -> crate::resource::ResourceId {
                crate::resource::ResourceId::new("fabric.test.relation-provenance.panic")
                    .expect("id")
            }

            fn schema() -> crate::resource::ResourceSchemaDescriptor {
                crate::resource::ResourceSchemaDescriptor::provisional(Self::resource_id())
            }

            fn declaration(
                selection: &crate::authoring::ResourceSelection<Self>,
            ) -> fabric_core::ModuleDeclaration {
                fabric_core::ModuleDeclaration::new(selection.module_id().clone())
            }

            fn has_self_realization() -> bool {
                true
            }

            fn materialize(
                _selection: &crate::authoring::ResourceSelection<Self>,
            ) -> Option<Box<dyn fabric_core::ModuleRuntime>> {
                panic!("composition build must not materialize relation provenance")
            }
        }

        let target_store = ProvenanceTargetStore::select("primary").expect("store");
        let target_signal = ProvenanceTargetSignal::select().expect("signal");
        let adapter_signal_target = ProvenanceAdapterSignal::select().expect("adapter signal");
        let self_resource = ProvenanceOwnerResource::select("self").expect("self resource");
        let self_resource_module = self_resource.module_id().clone();
        let direct_resource = ProvenanceOwnerResource::select("direct")
            .expect("direct resource")
            .using(ProvenanceResourceAdapter::new())
            .expect("direct adapter");
        let direct_provider = direct_resource.adapter().provider_module_id().clone();
        let mediated_resource = ProvenanceMediatedResource::select("mediated")
            .expect("mediated resource")
            .using(ProvenanceMediatedResourceAdapter::new())
            .expect("mediated adapter");
        let mediated_owner = mediated_resource.resource().module_id().clone();
        let owner_system = ProvenanceOwnerSystem::select().expect("system");
        let owner_system_module = owner_system.module_id().clone();
        let component = ProvenanceRelationComponent::define();
        let panic_resource =
            <PanicSelfResource as crate::authoring::ResourceDefinition>::select("panic", ())
                .expect("panic resource");

        let composition = Fabric::new("fabric.test.relation-provenance")
            .expect("fabric")
            .resource(target_store.clone())
            .system(target_signal.clone())
            .system(adapter_signal_target)
            .resource(self_resource)
            .resource(direct_resource)
            .resource(mediated_resource)
            .system(owner_system)
            .resource(panic_resource)
            .component(component)
            .build()
            .expect("build");
        let manifest = composition.manifest();

        let self_store = relation(manifest, "store", |owner| {
            matches!(
                owner,
                RelationDeclarationOwner::Resource { resource_name, .. }
                    if resource_name.as_str() == "self"
            )
        });
        assert_eq!(self_store.consumer_module_id(), &self_resource_module);
        assert_eq!(
            self_store.target(),
            &RelationTargetDescriptor::Resource(ProvenanceTargetStore::resource_id())
        );
        assert_eq!(
            self_store.requirement().id(),
            ProvenanceTargetStore::relation_requirement().id()
        );

        let self_signal = relation(manifest, "signal", |owner| {
            matches!(
                owner,
                RelationDeclarationOwner::Resource { resource_name, .. }
                    if resource_name.as_str() == "self"
            )
        });
        assert_eq!(
            self_signal.target(),
            &RelationTargetDescriptor::System(ProvenanceTargetSignal::system_id())
        );

        let direct_store = relation(manifest, "store", |owner| {
            matches!(
                owner,
                RelationDeclarationOwner::Resource { resource_name, .. }
                    if resource_name.as_str() == "direct"
            )
        });
        assert_eq!(direct_store.consumer_module_id(), &direct_provider);

        let mediated_signal = relation(manifest, "signal", |owner| {
            matches!(
                owner,
                RelationDeclarationOwner::Resource { resource_name, .. }
                    if resource_name.as_str() == "mediated"
            )
        });
        assert_eq!(mediated_signal.consumer_module_id(), &mediated_owner);

        let system_store = relation(manifest, "store", |owner| {
            matches!(owner, RelationDeclarationOwner::System { system_id }
                if system_id == &ProvenanceOwnerSystem::system_id())
        });
        assert_eq!(system_store.consumer_module_id(), &owner_system_module);
        assert_eq!(
            system_store.target(),
            &RelationTargetDescriptor::Resource(ProvenanceTargetStore::resource_id())
        );

        let adapter_signal = relation(manifest, "adapter_signal", |owner| {
            matches!(
                owner,
                RelationDeclarationOwner::AdapterRealizationUse {
                    adapter_definition_id,
                    provider_module_id,
                } if adapter_definition_id.as_str() == "test.relation-provenance.resource-adapter"
                    && provider_module_id == &direct_provider
            )
        });
        assert_eq!(adapter_signal.consumer_module_id(), &direct_provider);
        assert_eq!(
            adapter_signal.target(),
            &RelationTargetDescriptor::System(ProvenanceAdapterSignal::system_id())
        );

        let component_store = relation(manifest, "primary_store", |owner| {
            matches!(owner, RelationDeclarationOwner::Component { component_id }
                if component_id == &ProvenanceRelationComponent::component_id())
        });
        assert_eq!(
            component_store.target(),
            &RelationTargetDescriptor::Resource(ProvenanceTargetStore::resource_id())
        );

        let primary_signal = relation(manifest, "primary_signal", |owner| {
            matches!(owner, RelationDeclarationOwner::Component { component_id }
                if component_id == &ProvenanceRelationComponent::component_id())
        });
        let fallback_signal = relation(manifest, "fallback_signal", |owner| {
            matches!(owner, RelationDeclarationOwner::Component { component_id }
                if component_id == &ProvenanceRelationComponent::component_id())
        });
        assert_eq!(
            primary_signal.target(),
            &RelationTargetDescriptor::System(ProvenanceTargetSignal::system_id())
        );
        assert_eq!(
            fallback_signal.target(),
            &RelationTargetDescriptor::System(ProvenanceTargetSignal::system_id())
        );
        assert_ne!(
            primary_signal.consumer_module_id(),
            fallback_signal.consumer_module_id()
        );

        assert!(manifest.resources().iter().any(|entry| {
            entry.resource_id() == &ProvenanceTargetStore::resource_id()
                && entry.name() == target_store.name()
                && entry.semantic_provider_module_id() == target_store.module_id()
        }));
        assert!(manifest.systems().iter().any(|entry| {
            entry.system_id() == &ProvenanceTargetSignal::system_id()
                && entry.semantic_provider_module_id() == target_signal.module_id()
        }));
    }
}
