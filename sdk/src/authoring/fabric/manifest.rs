use crate::authoring::{AdapterDefinitionId, RelationTargetDescriptor};
use fabric_component::{ComponentId, ComponentRelationName};
use fabric_core::{
    BlockId, CompositionExportDeclaration, ContractId, ContractIdentity, ContractProviderSelection,
    ContractRequirementDeclaration, ModuleDeclaration, ModuleId,
};
use fabric_resource::{ResourceId, ResourceName, ResourceSchemaDescriptor};

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

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ResourceManifestEntry {
    resource_id: ResourceId,
    name: ResourceName,
    schema: ResourceSchemaDescriptor,
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
}

impl ResourceAugmentationManifestEntry {
    pub(crate) fn new(
        contract_id: ContractId,
        contract_identity: ContractIdentity,
        resource_id: ResourceId,
        resource_name: ResourceName,
    ) -> Self {
        Self {
            contract_id,
            contract_identity,
            resource_id,
            resource_name,
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
}

impl ResourceManifestEntry {
    pub(crate) fn new(
        resource_id: ResourceId,
        name: ResourceName,
        schema: ResourceSchemaDescriptor,
        realization: RealizationProvenance,
        semantic_provider_module_id: ModuleId,
    ) -> Self {
        Self {
            resource_id,
            name,
            schema,
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
    realization: RealizationProvenance,
    semantic_provider_module_id: ModuleId,
}

/// Bounded semantic inspection truth for an external System attachment.
#[derive(Clone, Debug)]
pub struct SystemAugmentationManifestEntry {
    contract_id: ContractId,
    contract_identity: ContractIdentity,
    system_id: fabric_system::SystemId,
}

#[derive(Clone, Debug)]
pub struct ComponentAugmentationManifestEntry {
    contract_id: ContractId,
    contract_identity: ContractIdentity,
    component_id: ComponentId,
}
impl ComponentAugmentationManifestEntry {
    pub(crate) fn new(
        contract_id: ContractId,
        contract_identity: ContractIdentity,
        component_id: ComponentId,
    ) -> Self {
        Self {
            contract_id,
            contract_identity,
            component_id,
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
}

impl SystemAugmentationManifestEntry {
    pub(crate) fn new(
        contract_id: ContractId,
        contract_identity: ContractIdentity,
        system_id: fabric_system::SystemId,
    ) -> Self {
        Self {
            contract_id,
            contract_identity,
            system_id,
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
        realization: RealizationProvenance,
        semantic_provider_module_id: ModuleId,
    ) -> Self {
        Self {
            system_id,
            schema,
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
    module_declarations: Vec<ModuleDeclaration>,
    provider_selections: Vec<ContractProviderSelection>,
    component_resource_bindings: Vec<ComponentResourceBindingManifestEntry>,
    component_system_bindings: Vec<ComponentSystemBindingManifestEntry>,
    relation_declarations: Vec<RelationDeclarationProvenance>,
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
        module_declarations: Vec<ModuleDeclaration>,
        provider_selections: Vec<ContractProviderSelection>,
        component_resource_bindings: Vec<ComponentResourceBindingManifestEntry>,
        component_system_bindings: Vec<ComponentSystemBindingManifestEntry>,
        relation_declarations: Vec<RelationDeclarationProvenance>,
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
            provider_selections,
            component_resource_bindings,
            component_system_bindings,
            relation_declarations,
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

    pub fn diagnostics(&self) -> FabricManifestDiagnostics<'_> {
        FabricManifestDiagnostics {
            module_declarations: &self.module_declarations,
            provider_selections: &self.provider_selections,
            raw_blocks: &self.raw_blocks,
            composition_exports: &self.composition_exports,
        }
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
