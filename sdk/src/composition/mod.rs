mod manifest;

use std::collections::BTreeMap;
use std::sync::Arc;

use fabric_component::{ComponentDeclaration, ComponentHostHandle, ComponentId};
use fabric_core::{Composition as CoreComposition, CompositionExport, CompositionId};

use crate::authoring::{FabricBuildError, RelationTargetDescriptor};

pub(crate) use manifest::{
    AdapterRealizationMode, AdapterRealizationProvenance, RealizationProvenance,
    RelationDeclarationOwner, RelationDeclarationProvenance, adapter_realization_provenance,
};
pub use manifest::{
    AdapterRealizedOwner, ComponentAugmentationInspection, ComponentAugmentationManifestEntry,
    ComponentInspection, ComponentResourceBindingManifestEntry,
    ComponentSystemBindingManifestEntry, FabricManifest, FabricManifestDiagnostics,
    RealizationInspection, ResourceAugmentationInspection, ResourceAugmentationManifestEntry,
    ResourceInspection, ResourceManifestEntry, SemanticApiEndpoint, SemanticApiMetadata,
    SemanticRealizationKind, SemanticRelationBindingManifestEntry, SemanticRelationOwner,
    SemanticRelationTargetDefinition, SemanticRelationTargetOccurrence,
    SystemAugmentationInspection, SystemAugmentationManifestEntry, SystemInspection,
    SystemManifestEntry,
};

pub struct Composition {
    core: CoreComposition,
    manifest: Arc<FabricManifest>,
    component_host_export: Option<CompositionExport<ComponentHostHandle>>,
}

impl std::fmt::Debug for Composition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Composition")
            .field("id", self.id())
            .finish()
    }
}

impl Composition {
    pub(crate) fn new(
        core: CoreComposition,
        manifest: FabricManifest,
        component_host_export: Option<CompositionExport<ComponentHostHandle>>,
    ) -> Self {
        Self {
            core,
            manifest: Arc::new(manifest),
            component_host_export,
        }
    }

    pub fn id(&self) -> &CompositionId {
        self.core.id()
    }

    /// Returns the generic resolved Core Composition for deliberate advanced use.
    pub fn core(&self) -> &CoreComposition {
        &self.core
    }

    pub fn manifest(&self) -> &FabricManifest {
        &self.manifest
    }

    pub fn resources(&self) -> impl Iterator<Item = ResourceInspection<'_>> + '_ {
        self.manifest
            .resources()
            .iter()
            .map(|entry| ResourceInspection::new(entry, &self.manifest))
    }

    pub fn systems(&self) -> impl Iterator<Item = SystemInspection<'_>> + '_ {
        self.manifest
            .systems()
            .iter()
            .map(|entry| SystemInspection::new(entry, &self.manifest))
    }

    pub fn components(&self) -> impl Iterator<Item = ComponentInspection<'_>> + '_ {
        self.manifest
            .components()
            .iter()
            .map(|declaration| ComponentInspection::new(declaration, &self.manifest))
    }

    pub fn relations(&self) -> &[SemanticRelationBindingManifestEntry] {
        self.manifest.relations()
    }

    pub fn into_core(self) -> CoreComposition {
        self.core
    }

    pub(crate) fn component_host_export(&self) -> Option<&CompositionExport<ComponentHostHandle>> {
        self.component_host_export.as_ref()
    }

    pub(crate) fn semantic_context(&self) -> Arc<FabricManifest> {
        Arc::clone(&self.manifest)
    }
}

pub(crate) fn resolved_semantic_relation_bindings(
    composition: &CoreComposition,
    resources: &[ResourceManifestEntry],
    systems: &[SystemManifestEntry],
    components: &[ComponentDeclaration],
    component_realizations: &BTreeMap<ComponentId, RealizationProvenance>,
    relation_declarations: &[RelationDeclarationProvenance],
) -> Result<Vec<SemanticRelationBindingManifestEntry>, FabricBuildError> {
    let mut resource_providers = BTreeMap::new();
    for resource in resources {
        let occurrence = SemanticRelationTargetOccurrence::Resource {
            resource_id: resource.resource_id().clone(),
            resource_name: resource.name().clone(),
        };
        if resource_providers
            .insert(resource.semantic_provider_module_id().clone(), occurrence)
            .is_some()
        {
            return Err(FabricBuildError::DuplicateSemanticProviderModule {
                module_id: resource.semantic_provider_module_id().clone(),
            });
        }
    }
    let mut system_providers = BTreeMap::new();
    for system in systems {
        let occurrence = SemanticRelationTargetOccurrence::System {
            system_id: system.system_id().clone(),
        };
        if system_providers
            .insert(system.semantic_provider_module_id().clone(), occurrence)
            .is_some()
        {
            return Err(FabricBuildError::DuplicateSemanticProviderModule {
                module_id: system.semantic_provider_module_id().clone(),
            });
        }
    }
    let mut adapter_owners = BTreeMap::new();
    for resource in resources {
        if let RealizationProvenance::Adapter(provenance) = resource.realization() {
            adapter_owners.insert(
                provenance.provider_module_id.clone(),
                AdapterRealizedOwner::Resource {
                    resource_id: resource.resource_id().clone(),
                    resource_name: resource.name().clone(),
                },
            );
        }
    }
    for system in systems {
        if let RealizationProvenance::Adapter(provenance) = system.realization() {
            adapter_owners.insert(
                provenance.provider_module_id.clone(),
                AdapterRealizedOwner::System {
                    system_id: system.system_id().clone(),
                },
            );
        }
    }
    for component in components {
        if let Some(RealizationProvenance::Adapter(provenance)) =
            component_realizations.get(component.component_id())
        {
            adapter_owners.insert(
                provenance.provider_module_id.clone(),
                AdapterRealizedOwner::Component {
                    component_id: component.component_id().clone(),
                },
            );
        }
    }

    relation_declarations
        .iter()
        .map(|declaration| {
            let binding = composition
                .resolved_binding(
                    declaration.consumer_module_id(),
                    declaration.requirement().id(),
                )
                .ok_or_else(|| FabricBuildError::MissingSemanticRelationBinding {
                    consumer: declaration.consumer_module_id().clone(),
                    contract_id: declaration.requirement().id().clone(),
                })?;
            if binding.requirement() != declaration.requirement() {
                return Err(FabricBuildError::SemanticRelationRequirementMismatch {
                    consumer: declaration.consumer_module_id().clone(),
                    contract_id: declaration.requirement().id().clone(),
                });
            }
            let (declared_target, resolved_target) = match declaration.target() {
                RelationTargetDescriptor::Resource(resource_id) => {
                    let resolved = resource_providers.get(binding.provider()).ok_or_else(|| {
                        FabricBuildError::UnmappedSemanticRelationProvider {
                            provider: binding.provider().clone(),
                            contract_id: declaration.requirement().id().clone(),
                        }
                    })?;
                    match resolved {
                        SemanticRelationTargetOccurrence::Resource {
                            resource_id: resolved_id,
                            ..
                        } if resolved_id == resource_id => (
                            SemanticRelationTargetDefinition::Resource {
                                resource_id: resource_id.clone(),
                            },
                            resolved.clone(),
                        ),
                        _ => {
                            return Err(FabricBuildError::InconsistentSemanticRelationTarget {
                                provider: binding.provider().clone(),
                                contract_id: declaration.requirement().id().clone(),
                            });
                        }
                    }
                }
                RelationTargetDescriptor::System(system_id) => {
                    let resolved = system_providers.get(binding.provider()).ok_or_else(|| {
                        FabricBuildError::UnmappedSemanticRelationProvider {
                            provider: binding.provider().clone(),
                            contract_id: declaration.requirement().id().clone(),
                        }
                    })?;
                    match resolved {
                        SemanticRelationTargetOccurrence::System {
                            system_id: resolved_id,
                        } if resolved_id == system_id => (
                            SemanticRelationTargetDefinition::System {
                                system_id: system_id.clone(),
                            },
                            resolved.clone(),
                        ),
                        _ => {
                            return Err(FabricBuildError::InconsistentSemanticRelationTarget {
                                provider: binding.provider().clone(),
                                contract_id: declaration.requirement().id().clone(),
                            });
                        }
                    }
                }
            };
            let owner = match declaration.owner() {
                RelationDeclarationOwner::Resource {
                    resource_id,
                    resource_name,
                } => SemanticRelationOwner::Resource {
                    resource_id: resource_id.clone(),
                    resource_name: resource_name.clone(),
                },
                RelationDeclarationOwner::System { system_id } => SemanticRelationOwner::System {
                    system_id: system_id.clone(),
                },
                RelationDeclarationOwner::Component { component_id } => {
                    SemanticRelationOwner::Component {
                        component_id: component_id.clone(),
                    }
                }
                RelationDeclarationOwner::AdapterRealizationUse {
                    adapter_definition_id,
                    provider_module_id,
                } => SemanticRelationOwner::AdapterRealization {
                    adapter_definition_id: adapter_definition_id.clone(),
                    realized_owner: adapter_owners.get(provider_module_id).cloned(),
                },
            };
            Ok(SemanticRelationBindingManifestEntry::new(
                owner,
                declaration.role().clone(),
                declared_target,
                resolved_target,
                declaration.requirement().clone(),
            ))
        })
        .collect()
}
