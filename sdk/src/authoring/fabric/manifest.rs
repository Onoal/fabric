use crate::authoring::{AdapterDefinitionId, RelationTargetDescriptor};
use fabric_component::{ComponentDesiredState, ComponentId, ComponentRelationName};
use fabric_core::{
    BlockId, CompositionExportDeclaration, ContractId, ContractIdentity, ContractProviderSelection,
    ContractRequirementDeclaration, ModuleDeclaration, ModuleId,
};
use fabric_host::HostRequirement;
use fabric_resource::{ResourceId, ResourceName, ResourceSchemaDescriptor};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticApiMetadata {
    contract_id: Option<ContractId>,
    contract_identity: Option<ContractIdentity>,
    endpoints: Vec<SemanticApiEndpoint>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticApiEndpoint {
    name: String,
}

impl SemanticApiMetadata {
    pub fn new(
        contract_id: ContractId,
        contract_identity: ContractIdentity,
        endpoints: Vec<SemanticApiEndpoint>,
    ) -> Self {
        Self {
            contract_id: Some(contract_id),
            contract_identity: Some(contract_identity),
            endpoints,
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn contract_id(&self) -> Option<&ContractId> {
        self.contract_id.as_ref()
    }

    pub fn contract_identity(&self) -> Option<&ContractIdentity> {
        self.contract_identity.as_ref()
    }

    pub fn endpoints(&self) -> &[SemanticApiEndpoint] {
        &self.endpoints
    }
}

impl SemanticApiEndpoint {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Private semantic realization truth retained while SDK authoring still knows
/// how a participant was lowered. It contains declarative definition and Core
/// module provenance only; live runtime state remains in an Instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RealizationProvenance {
    DeclarationOnly,
    SelfRealization { runtime_module_id: Option<ModuleId> },
    Adapter(AdapterRealizationProvenance),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AdapterRealizationMode {
    Direct,
    Mediated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AdapterRealizationProvenance {
    pub(crate) definition_id: AdapterDefinitionId,
    pub(crate) mode: AdapterRealizationMode,
    pub(crate) provider_module_id: ModuleId,
    pub(crate) semantic_owner_module_id: Option<ModuleId>,
}

pub(crate) fn adapter_realization_provenance(
    bridge_mode: crate::authoring::AdapterBridgeMode,
    definition_id: AdapterDefinitionId,
    provider_module_id: ModuleId,
    semantic_owner_module_id: ModuleId,
) -> RealizationProvenance {
    let (mode, semantic_owner_module_id) = match bridge_mode {
        crate::authoring::AdapterBridgeMode::SemanticApi => (AdapterRealizationMode::Direct, None),
        crate::authoring::AdapterBridgeMode::ExplicitContract
        | crate::authoring::AdapterBridgeMode::DifferentialSemanticApi => (
            AdapterRealizationMode::Mediated,
            Some(semantic_owner_module_id),
        ),
    };
    RealizationProvenance::Adapter(AdapterRealizationProvenance {
        definition_id,
        mode,
        provider_module_id,
        semantic_owner_module_id,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RelationDeclarationOwner {
    Resource {
        resource_id: ResourceId,
        resource_name: ResourceName,
    },
    System {
        system_id: fabric_system::SystemId,
    },
    Component {
        component_id: ComponentId,
    },
    AdapterRealizationUse {
        adapter_definition_id: AdapterDefinitionId,
        provider_module_id: ModuleId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RelationDeclarationProvenance {
    owner: RelationDeclarationOwner,
    role: ComponentRelationName,
    target: RelationTargetDescriptor,
    consumer_module_id: ModuleId,
    requirement: ContractRequirementDeclaration,
}

#[allow(dead_code)]
impl RelationDeclarationProvenance {
    pub(crate) fn new(
        owner: RelationDeclarationOwner,
        role: ComponentRelationName,
        target: RelationTargetDescriptor,
        consumer_module_id: ModuleId,
        requirement: ContractRequirementDeclaration,
    ) -> Self {
        Self {
            owner,
            role,
            target,
            consumer_module_id,
            requirement,
        }
    }

    pub(crate) fn owner(&self) -> &RelationDeclarationOwner {
        &self.owner
    }

    pub(crate) fn role(&self) -> &ComponentRelationName {
        &self.role
    }

    pub(crate) fn target(&self) -> &RelationTargetDescriptor {
        &self.target
    }

    pub(crate) fn consumer_module_id(&self) -> &ModuleId {
        &self.consumer_module_id
    }

    pub(crate) fn requirement(&self) -> &ContractRequirementDeclaration {
        &self.requirement
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticRelationOwner {
    Resource {
        resource_id: ResourceId,
        resource_name: ResourceName,
    },
    System {
        system_id: fabric_system::SystemId,
    },
    Component {
        component_id: ComponentId,
    },
    AdapterRealization {
        adapter_definition_id: AdapterDefinitionId,
        realized_owner: Option<AdapterRealizedOwner>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdapterRealizedOwner {
    Resource {
        resource_id: ResourceId,
        resource_name: ResourceName,
    },
    System {
        system_id: fabric_system::SystemId,
    },
    Component {
        component_id: ComponentId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticRelationTargetDefinition {
    Resource { resource_id: ResourceId },
    System { system_id: fabric_system::SystemId },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticRelationTargetOccurrence {
    Resource {
        resource_id: ResourceId,
        resource_name: ResourceName,
    },
    System {
        system_id: fabric_system::SystemId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticRelationBindingManifestEntry {
    owner: SemanticRelationOwner,
    role: ComponentRelationName,
    declared_target: SemanticRelationTargetDefinition,
    resolved_target: SemanticRelationTargetOccurrence,
    requirement: ContractRequirementDeclaration,
}

impl SemanticRelationBindingManifestEntry {
    pub(crate) fn new(
        owner: SemanticRelationOwner,
        role: ComponentRelationName,
        declared_target: SemanticRelationTargetDefinition,
        resolved_target: SemanticRelationTargetOccurrence,
        requirement: ContractRequirementDeclaration,
    ) -> Self {
        Self {
            owner,
            role,
            declared_target,
            resolved_target,
            requirement,
        }
    }

    pub fn owner(&self) -> &SemanticRelationOwner {
        &self.owner
    }

    pub fn role(&self) -> &ComponentRelationName {
        &self.role
    }

    pub fn declared_target(&self) -> &SemanticRelationTargetDefinition {
        &self.declared_target
    }

    pub fn resolved_target(&self) -> &SemanticRelationTargetOccurrence {
        &self.resolved_target
    }

    pub fn requirement(&self) -> &ContractRequirementDeclaration {
        &self.requirement
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ResourceManifestEntry {
    resource_id: ResourceId,
    name: ResourceName,
    schema: ResourceSchemaDescriptor,
    api: SemanticApiMetadata,
    realization: RealizationProvenance,
    semantic_provider_module_id: ModuleId,
}

/// Bounded semantic inspection truth for an external Resource attachment.
#[derive(Clone, Debug)]
pub struct ResourceAugmentationManifestEntry {
    contract_id: ContractId,
    contract_identity: ContractIdentity,
    resource_id: ResourceId,
    resource_name: ResourceName,
    support_provider_module_id: Option<ModuleId>,
}

impl ResourceAugmentationManifestEntry {
    pub(crate) fn new(
        contract_id: ContractId,
        contract_identity: ContractIdentity,
        resource_id: ResourceId,
        resource_name: ResourceName,
        support_provider_module_id: Option<ModuleId>,
    ) -> Self {
        Self {
            contract_id,
            contract_identity,
            resource_id,
            resource_name,
            support_provider_module_id,
        }
    }

    pub fn contract_id(&self) -> &ContractId {
        &self.contract_id
    }
    pub fn contract_identity(&self) -> &ContractIdentity {
        &self.contract_identity
    }
    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    pub fn resource_name(&self) -> &ResourceName {
        &self.resource_name
    }
    pub fn has_support_realization(&self) -> bool {
        self.support_provider_module_id.is_some()
    }
    pub(crate) fn support_provider_module_id(&self) -> Option<&ModuleId> {
        self.support_provider_module_id.as_ref()
    }
}

impl ResourceManifestEntry {
    pub(crate) fn new(
        resource_id: ResourceId,
        name: ResourceName,
        schema: ResourceSchemaDescriptor,
        api: SemanticApiMetadata,
        realization: RealizationProvenance,
        semantic_provider_module_id: ModuleId,
    ) -> Self {
        Self {
            resource_id,
            name,
            schema,
            api,
            realization,
            semantic_provider_module_id,
        }
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn name(&self) -> &ResourceName {
        &self.name
    }

    pub fn schema(&self) -> &ResourceSchemaDescriptor {
        &self.schema
    }

    pub fn api(&self) -> &SemanticApiMetadata {
        &self.api
    }

    #[allow(dead_code)]
    pub(crate) fn realization(&self) -> &RealizationProvenance {
        &self.realization
    }

    #[allow(dead_code)]
    pub(crate) fn semantic_provider_module_id(&self) -> &ModuleId {
        &self.semantic_provider_module_id
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SystemManifestEntry {
    system_id: fabric_system::SystemId,
    schema: fabric_system::SystemSchemaDescriptor,
    api: SemanticApiMetadata,
    realization: RealizationProvenance,
    semantic_provider_module_id: ModuleId,
}

/// Bounded semantic inspection truth for an external System attachment.
#[derive(Clone, Debug)]
pub struct SystemAugmentationManifestEntry {
    contract_id: ContractId,
    contract_identity: ContractIdentity,
    system_id: fabric_system::SystemId,
    support_provider_module_id: Option<ModuleId>,
}

#[derive(Clone, Debug)]
pub struct ComponentAugmentationManifestEntry {
    contract_id: ContractId,
    contract_identity: ContractIdentity,
    component_id: ComponentId,
    support_provider_module_id: Option<ModuleId>,
}
impl ComponentAugmentationManifestEntry {
    pub(crate) fn new(
        contract_id: ContractId,
        contract_identity: ContractIdentity,
        component_id: ComponentId,
        support_provider_module_id: Option<ModuleId>,
    ) -> Self {
        Self {
            contract_id,
            contract_identity,
            component_id,
            support_provider_module_id,
        }
    }
    pub fn contract_id(&self) -> &ContractId {
        &self.contract_id
    }
    pub fn contract_identity(&self) -> &ContractIdentity {
        &self.contract_identity
    }
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    pub fn has_support_realization(&self) -> bool {
        self.support_provider_module_id.is_some()
    }
    pub(crate) fn support_provider_module_id(&self) -> Option<&ModuleId> {
        self.support_provider_module_id.as_ref()
    }
}

impl SystemAugmentationManifestEntry {
    pub(crate) fn new(
        contract_id: ContractId,
        contract_identity: ContractIdentity,
        system_id: fabric_system::SystemId,
        support_provider_module_id: Option<ModuleId>,
    ) -> Self {
        Self {
            contract_id,
            contract_identity,
            system_id,
            support_provider_module_id,
        }
    }

    pub fn contract_id(&self) -> &ContractId {
        &self.contract_id
    }
    pub fn contract_identity(&self) -> &ContractIdentity {
        &self.contract_identity
    }
    pub fn system_id(&self) -> &fabric_system::SystemId {
        &self.system_id
    }
    pub fn has_support_realization(&self) -> bool {
        self.support_provider_module_id.is_some()
    }
    pub(crate) fn support_provider_module_id(&self) -> Option<&ModuleId> {
        self.support_provider_module_id.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct ComponentResourceBindingManifestEntry {
    component_id: ComponentId,
    requirement_name: ComponentRelationName,
    resource_id: ResourceId,
    resource_name: ResourceName,
}

impl ComponentResourceBindingManifestEntry {
    pub(crate) fn new(
        component_id: ComponentId,
        requirement_name: ComponentRelationName,
        resource_id: ResourceId,
        resource_name: ResourceName,
    ) -> Self {
        Self {
            component_id,
            requirement_name,
            resource_id,
            resource_name,
        }
    }
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    pub fn requirement_name(&self) -> &ComponentRelationName {
        &self.requirement_name
    }
    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    pub fn resource_name(&self) -> &ResourceName {
        &self.resource_name
    }
}

#[derive(Clone, Debug)]
pub struct ComponentSystemBindingManifestEntry {
    component_id: ComponentId,
    system_id: fabric_system::SystemId,
}
impl ComponentSystemBindingManifestEntry {
    pub(crate) fn new(component_id: ComponentId, system_id: fabric_system::SystemId) -> Self {
        Self {
            component_id,
            system_id,
        }
    }
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    pub fn system_id(&self) -> &fabric_system::SystemId {
        &self.system_id
    }
}

impl SystemManifestEntry {
    pub(crate) fn new(
        system_id: fabric_system::SystemId,
        schema: fabric_system::SystemSchemaDescriptor,
        api: SemanticApiMetadata,
        realization: RealizationProvenance,
        semantic_provider_module_id: ModuleId,
    ) -> Self {
        Self {
            system_id,
            schema,
            api,
            realization,
            semantic_provider_module_id,
        }
    }

    pub fn system_id(&self) -> &fabric_system::SystemId {
        &self.system_id
    }

    pub fn schema(&self) -> &fabric_system::SystemSchemaDescriptor {
        &self.schema
    }

    pub fn api(&self) -> &SemanticApiMetadata {
        &self.api
    }

    #[allow(dead_code)]
    pub(crate) fn realization(&self) -> &RealizationProvenance {
        &self.realization
    }

    #[allow(dead_code)]
    pub(crate) fn semantic_provider_module_id(&self) -> &ModuleId {
        &self.semantic_provider_module_id
    }
}

pub struct FabricManifestDiagnostics<'a> {
    module_declarations: &'a [ModuleDeclaration],
    provider_selections: &'a [ContractProviderSelection],
    raw_blocks: &'a [BlockId],
    composition_exports: &'a [CompositionExportDeclaration],
}

impl<'a> FabricManifestDiagnostics<'a> {
    pub fn module_declarations(&self) -> &'a [ModuleDeclaration] {
        self.module_declarations
    }
    pub fn provider_selections(&self) -> &'a [ContractProviderSelection] {
        self.provider_selections
    }
    pub fn raw_blocks(&self) -> &'a [BlockId] {
        self.raw_blocks
    }
    pub fn composition_exports(&self) -> &'a [CompositionExportDeclaration] {
        self.composition_exports
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct FabricManifest {
    resources: Vec<ResourceManifestEntry>,
    resource_augmentations: Vec<ResourceAugmentationManifestEntry>,
    systems: Vec<SystemManifestEntry>,
    system_augmentations: Vec<SystemAugmentationManifestEntry>,
    component_augmentations: Vec<ComponentAugmentationManifestEntry>,
    components: Vec<fabric_component::ComponentDeclaration>,
    component_realizations: std::collections::BTreeMap<ComponentId, RealizationProvenance>,
    component_initial_participation: std::collections::BTreeMap<ComponentId, ComponentDesiredState>,
    module_declarations: Vec<ModuleDeclaration>,
    provider_selections: Vec<ContractProviderSelection>,
    component_resource_bindings: Vec<ComponentResourceBindingManifestEntry>,
    component_system_bindings: Vec<ComponentSystemBindingManifestEntry>,
    relation_declarations: Vec<RelationDeclarationProvenance>,
    relation_bindings: Vec<SemanticRelationBindingManifestEntry>,
    raw_blocks: Vec<BlockId>,
    composition_exports: Vec<CompositionExportDeclaration>,
}

impl FabricManifest {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        resources: Vec<ResourceManifestEntry>,
        resource_augmentations: Vec<ResourceAugmentationManifestEntry>,
        systems: Vec<SystemManifestEntry>,
        system_augmentations: Vec<SystemAugmentationManifestEntry>,
        component_augmentations: Vec<ComponentAugmentationManifestEntry>,
        components: Vec<fabric_component::ComponentDeclaration>,
        component_realizations: std::collections::BTreeMap<ComponentId, RealizationProvenance>,
        component_initial_participation: std::collections::BTreeMap<
            ComponentId,
            ComponentDesiredState,
        >,
        module_declarations: Vec<ModuleDeclaration>,
        provider_selections: Vec<ContractProviderSelection>,
        component_resource_bindings: Vec<ComponentResourceBindingManifestEntry>,
        component_system_bindings: Vec<ComponentSystemBindingManifestEntry>,
        relation_declarations: Vec<RelationDeclarationProvenance>,
        relation_bindings: Vec<SemanticRelationBindingManifestEntry>,
        raw_blocks: Vec<BlockId>,
        composition_exports: Vec<CompositionExportDeclaration>,
    ) -> Self {
        Self {
            resources,
            resource_augmentations,
            systems,
            system_augmentations,
            component_augmentations,
            components,
            module_declarations,
            component_realizations,
            component_initial_participation,
            provider_selections,
            component_resource_bindings,
            component_system_bindings,
            relation_declarations,
            relation_bindings,
            raw_blocks,
            composition_exports,
        }
    }

    pub fn resources(&self) -> &[ResourceManifestEntry] {
        &self.resources
    }

    pub fn resource_augmentations(&self) -> &[ResourceAugmentationManifestEntry] {
        &self.resource_augmentations
    }

    pub fn systems(&self) -> &[SystemManifestEntry] {
        &self.systems
    }

    pub fn system_augmentations(&self) -> &[SystemAugmentationManifestEntry] {
        &self.system_augmentations
    }
    pub fn component_augmentations(&self) -> &[ComponentAugmentationManifestEntry] {
        &self.component_augmentations
    }

    pub fn components(&self) -> &[fabric_component::ComponentDeclaration] {
        &self.components
    }

    #[allow(dead_code)]
    pub(crate) fn component_realization(
        &self,
        component_id: &ComponentId,
    ) -> Option<&RealizationProvenance> {
        self.component_realizations.get(component_id)
    }

    pub(crate) fn component_initial_participation(
        &self,
        component_id: &ComponentId,
    ) -> Option<ComponentDesiredState> {
        self.component_initial_participation
            .get(component_id)
            .copied()
    }

    pub fn component_resource_bindings(&self) -> &[ComponentResourceBindingManifestEntry] {
        &self.component_resource_bindings
    }
    pub fn component_system_bindings(&self) -> &[ComponentSystemBindingManifestEntry] {
        &self.component_system_bindings
    }

    #[allow(dead_code)]
    pub(crate) fn relation_declarations(&self) -> &[RelationDeclarationProvenance] {
        &self.relation_declarations
    }

    pub fn relations(&self) -> &[SemanticRelationBindingManifestEntry] {
        &self.relation_bindings
    }

    pub fn diagnostics(&self) -> FabricManifestDiagnostics<'_> {
        FabricManifestDiagnostics {
            module_declarations: &self.module_declarations,
            provider_selections: &self.provider_selections,
            raw_blocks: &self.raw_blocks,
            composition_exports: &self.composition_exports,
        }
    }

    fn host_requirement_for_module(&self, module_id: &ModuleId) -> Option<&HostRequirement> {
        self.module_declarations
            .iter()
            .find(|declaration| declaration.module_id() == module_id)
            .and_then(|declaration| declaration.host_requirement())
            .map(|requirement| requirement.requirement())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticRealizationKind {
    DeclarationOnly,
    SelfRealization,
    AdapterDirect,
    AdapterMediated,
}

#[derive(Clone, Copy)]
pub struct RealizationInspection<'a> {
    provenance: &'a RealizationProvenance,
    manifest: &'a FabricManifest,
}

impl<'a> RealizationInspection<'a> {
    pub(crate) fn new(provenance: &'a RealizationProvenance, manifest: &'a FabricManifest) -> Self {
        Self {
            provenance,
            manifest,
        }
    }

    pub fn kind(&self) -> SemanticRealizationKind {
        match self.provenance {
            RealizationProvenance::DeclarationOnly => SemanticRealizationKind::DeclarationOnly,
            RealizationProvenance::SelfRealization { .. } => {
                SemanticRealizationKind::SelfRealization
            }
            RealizationProvenance::Adapter(provenance) => match provenance.mode {
                AdapterRealizationMode::Direct => SemanticRealizationKind::AdapterDirect,
                AdapterRealizationMode::Mediated => SemanticRealizationKind::AdapterMediated,
            },
        }
    }

    pub fn adapter_definition_id(&self) -> Option<&AdapterDefinitionId> {
        match self.provenance {
            RealizationProvenance::Adapter(provenance) => Some(&provenance.definition_id),
            _ => None,
        }
    }

    pub fn host_requirement(&self) -> Option<&HostRequirement> {
        match self.provenance {
            RealizationProvenance::DeclarationOnly => None,
            RealizationProvenance::SelfRealization {
                runtime_module_id: Some(module_id),
            } => self.manifest.host_requirement_for_module(module_id),
            RealizationProvenance::SelfRealization {
                runtime_module_id: None,
            } => None,
            RealizationProvenance::Adapter(provenance) => self
                .manifest
                .host_requirement_for_module(&provenance.provider_module_id),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ResourceInspection<'a> {
    entry: &'a ResourceManifestEntry,
    manifest: &'a FabricManifest,
}

impl<'a> ResourceInspection<'a> {
    pub(crate) fn new(entry: &'a ResourceManifestEntry, manifest: &'a FabricManifest) -> Self {
        Self { entry, manifest }
    }

    pub fn resource_id(&self) -> &ResourceId {
        self.entry.resource_id()
    }
    pub fn name(&self) -> &ResourceName {
        self.entry.name()
    }
    pub fn schema(&self) -> &ResourceSchemaDescriptor {
        self.entry.schema()
    }
    pub fn api(&self) -> &SemanticApiMetadata {
        self.entry.api()
    }
    pub fn realization(&self) -> RealizationInspection<'a> {
        RealizationInspection::new(self.entry.realization(), self.manifest)
    }
    pub fn relations(&self) -> impl Iterator<Item = &'a SemanticRelationBindingManifestEntry> + 'a {
        let resource_id = self.entry.resource_id();
        let resource_name = self.entry.name();
        self.manifest.relations().iter().filter(move |relation| {
            matches!(
                relation.owner(),
                SemanticRelationOwner::Resource {
                    resource_id: owner_id,
                    resource_name: owner_name,
                } if owner_id == resource_id && owner_name == resource_name
            )
        })
    }
    pub fn required_by(
        &self,
    ) -> impl Iterator<Item = &'a SemanticRelationBindingManifestEntry> + 'a {
        let target = SemanticRelationTargetOccurrence::Resource {
            resource_id: self.entry.resource_id().clone(),
            resource_name: self.entry.name().clone(),
        };
        self.manifest
            .relations()
            .iter()
            .filter(move |relation| relation.resolved_target() == &target)
    }
    pub fn augmentations(&self) -> impl Iterator<Item = ResourceAugmentationInspection<'a>> + 'a {
        let resource_id = self.entry.resource_id();
        let resource_name = self.entry.name();
        let manifest = self.manifest;
        self.manifest
            .resource_augmentations()
            .iter()
            .filter(move |augmentation| {
                augmentation.resource_id() == resource_id
                    && augmentation.resource_name() == resource_name
            })
            .map(move |entry| ResourceAugmentationInspection { entry, manifest })
    }
}

#[derive(Clone, Copy)]
pub struct SystemInspection<'a> {
    entry: &'a SystemManifestEntry,
    manifest: &'a FabricManifest,
}

impl<'a> SystemInspection<'a> {
    pub(crate) fn new(entry: &'a SystemManifestEntry, manifest: &'a FabricManifest) -> Self {
        Self { entry, manifest }
    }

    pub fn system_id(&self) -> &fabric_system::SystemId {
        self.entry.system_id()
    }
    pub fn schema(&self) -> &fabric_system::SystemSchemaDescriptor {
        self.entry.schema()
    }
    pub fn api(&self) -> &SemanticApiMetadata {
        self.entry.api()
    }
    pub fn realization(&self) -> RealizationInspection<'a> {
        RealizationInspection::new(self.entry.realization(), self.manifest)
    }
    pub fn relations(&self) -> impl Iterator<Item = &'a SemanticRelationBindingManifestEntry> + 'a {
        let system_id = self.entry.system_id();
        self.manifest.relations().iter().filter(move |relation| {
            matches!(
                relation.owner(),
                SemanticRelationOwner::System { system_id: owner_id } if owner_id == system_id
            )
        })
    }
    pub fn required_by(
        &self,
    ) -> impl Iterator<Item = &'a SemanticRelationBindingManifestEntry> + 'a {
        let target = SemanticRelationTargetOccurrence::System {
            system_id: self.entry.system_id().clone(),
        };
        self.manifest
            .relations()
            .iter()
            .filter(move |relation| relation.resolved_target() == &target)
    }
    pub fn augmentations(&self) -> impl Iterator<Item = SystemAugmentationInspection<'a>> + 'a {
        let system_id = self.entry.system_id();
        let manifest = self.manifest;
        self.manifest
            .system_augmentations()
            .iter()
            .filter(move |augmentation| augmentation.system_id() == system_id)
            .map(move |entry| SystemAugmentationInspection { entry, manifest })
    }
}

#[derive(Clone, Copy)]
pub struct ComponentInspection<'a> {
    declaration: &'a fabric_component::ComponentDeclaration,
    manifest: &'a FabricManifest,
}

impl<'a> ComponentInspection<'a> {
    pub(crate) fn new(
        declaration: &'a fabric_component::ComponentDeclaration,
        manifest: &'a FabricManifest,
    ) -> Self {
        Self {
            declaration,
            manifest,
        }
    }

    pub fn component_id(&self) -> &ComponentId {
        self.declaration.component_id()
    }
    pub fn api(&self) -> &fabric_component::ComponentApiMetadata {
        self.declaration.api()
    }
    pub fn declaration(&self) -> &fabric_component::ComponentDeclaration {
        self.declaration
    }
    pub fn realization(&self) -> Option<RealizationInspection<'a>> {
        self.manifest
            .component_realization(self.declaration.component_id())
            .map(|provenance| RealizationInspection::new(provenance, self.manifest))
    }
    pub fn initial_participation(&self) -> ComponentDesiredState {
        self.manifest
            .component_initial_participation(self.declaration.component_id())
            .expect("Component inspection must have retained initial participation intent")
    }
    pub fn relations(&self) -> impl Iterator<Item = &'a SemanticRelationBindingManifestEntry> + 'a {
        let component_id = self.declaration.component_id();
        self.manifest.relations().iter().filter(move |relation| {
            matches!(
                relation.owner(),
                SemanticRelationOwner::Component { component_id: owner_id } if owner_id == component_id
            )
        })
    }
    pub fn augmentations(&self) -> impl Iterator<Item = ComponentAugmentationInspection<'a>> + 'a {
        let component_id = self.declaration.component_id();
        let manifest = self.manifest;
        self.manifest
            .component_augmentations()
            .iter()
            .filter(move |augmentation| augmentation.component_id() == component_id)
            .map(move |entry| ComponentAugmentationInspection { entry, manifest })
    }
}

#[derive(Clone, Copy)]
pub struct ResourceAugmentationInspection<'a> {
    entry: &'a ResourceAugmentationManifestEntry,
    manifest: &'a FabricManifest,
}

impl<'a> ResourceAugmentationInspection<'a> {
    pub fn contract_id(&self) -> &ContractId {
        self.entry.contract_id()
    }
    pub fn contract_identity(&self) -> &ContractIdentity {
        self.entry.contract_identity()
    }
    pub fn has_support_realization(&self) -> bool {
        self.entry.has_support_realization()
    }
    pub fn host_requirement(&self) -> Option<&HostRequirement> {
        self.entry
            .support_provider_module_id()
            .and_then(|module_id| self.manifest.host_requirement_for_module(module_id))
    }
}

#[derive(Clone, Copy)]
pub struct SystemAugmentationInspection<'a> {
    entry: &'a SystemAugmentationManifestEntry,
    manifest: &'a FabricManifest,
}

impl<'a> SystemAugmentationInspection<'a> {
    pub fn contract_id(&self) -> &ContractId {
        self.entry.contract_id()
    }
    pub fn contract_identity(&self) -> &ContractIdentity {
        self.entry.contract_identity()
    }
    pub fn has_support_realization(&self) -> bool {
        self.entry.has_support_realization()
    }
    pub fn host_requirement(&self) -> Option<&HostRequirement> {
        self.entry
            .support_provider_module_id()
            .and_then(|module_id| self.manifest.host_requirement_for_module(module_id))
    }
}

#[derive(Clone, Copy)]
pub struct ComponentAugmentationInspection<'a> {
    entry: &'a ComponentAugmentationManifestEntry,
    manifest: &'a FabricManifest,
}

impl<'a> ComponentAugmentationInspection<'a> {
    pub fn contract_id(&self) -> &ContractId {
        self.entry.contract_id()
    }
    pub fn contract_identity(&self) -> &ContractIdentity {
        self.entry.contract_identity()
    }
    pub fn has_support_realization(&self) -> bool {
        self.entry.has_support_realization()
    }
    pub fn host_requirement(&self) -> Option<&HostRequirement> {
        self.entry
            .support_provider_module_id()
            .and_then(|module_id| self.manifest.host_requirement_for_module(module_id))
    }
}

#[cfg(test)]
mod realization_provenance_tests {
    use super::*;

    #[test]
    fn bridge_lowering_maps_only_semantic_direct_and_mediated_modes() {
        let definition_id =
            AdapterDefinitionId::new("test.provenance.adapter").expect("definition identity");
        let provider =
            ModuleId::new("fabric.test.provenance.provider").expect("provider module id");
        let owner =
            ModuleId::new("fabric.test.provenance.owner").expect("semantic owner module id");

        let direct = adapter_realization_provenance(
            crate::authoring::AdapterBridgeMode::SemanticApi,
            definition_id.clone(),
            provider.clone(),
            owner.clone(),
        );
        assert!(matches!(
            direct,
            RealizationProvenance::Adapter(AdapterRealizationProvenance {
                mode: AdapterRealizationMode::Direct,
                semantic_owner_module_id: None,
                ..
            })
        ));

        for bridge_mode in [
            crate::authoring::AdapterBridgeMode::ExplicitContract,
            crate::authoring::AdapterBridgeMode::DifferentialSemanticApi,
        ] {
            match adapter_realization_provenance(
                bridge_mode,
                definition_id.clone(),
                provider.clone(),
                owner.clone(),
            ) {
                RealizationProvenance::Adapter(provenance) => {
                    assert_eq!(provenance.mode, AdapterRealizationMode::Mediated);
                    assert_eq!(provenance.definition_id, definition_id);
                    assert_eq!(provenance.provider_module_id, provider);
                    assert_eq!(provenance.semantic_owner_module_id, Some(owner.clone()));
                }
                other => panic!("expected mediated Adapter provenance, got {other:?}"),
            }
        }
    }
}
