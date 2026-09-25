mod augmentation_requirement;
mod block_author;
mod composition_ext;
mod definitions;
mod fabric;
mod fabric_builder;
mod instance;
mod runtime_authoring;
mod system;

pub(crate) use augmentation_requirement::requirement_for_key;
pub use block_author::BlockAuthor;
pub use composition_ext::CompositionExt;
pub use definitions::{
    AdaptableComponentDefinition, AdaptableResourceDefinition, AdapterBridgeMode,
    AdapterDefinition, AdapterDefinitionId, AdapterDefinitionIdError, AdapterProviderModule,
    CanonicalAdapterSupport, CanonicalComponentAdapterRuntime, ComponentAdapterCompatibility,
    ComponentAdapterTarget, ComponentAugmentation, ComponentAugmentationDefinition,
    ComponentAugmentationRealization, ComponentAugmentationRequirement, ComponentAugmentationSet,
    ComponentAugmentationSetAdapterRealization, ComponentAugmentationSetAttachment,
    ComponentAugmentationSetRealization, ComponentAugmentationSupportDefinition,
    ComponentAugmentedAdapterRealization, ComponentDefinition, ComponentRealization,
    ComponentRealizationContract, ComponentRelationCompatibility, ComponentRelationTarget,
    ComponentResourceRequirement, ComponentResourceScope, ComponentSpec,
    ComponentSystemRequirement, ComponentSystemScope, ContractDependency, IntoResourceName,
    PrimaryResourceContract, RelationTarget, RelationTargetDescriptor, Requires,
    ResourceAdapterCompatibility, ResourceAugmentation, ResourceAugmentationDefinition,
    ResourceAugmentationError, ResourceAugmentationRealization, ResourceAugmentationRequirement,
    ResourceAugmentationSupportDefinition, ResourceDefinition, ResourceRealization,
    ResourceSelection, SelfRealizingComponentDefinition,
};
pub use fabric::{
    AdapterRealizedOwner, ComponentAugmentationInspection, ComponentAugmentationManifestEntry,
    ComponentInspection, ComponentResourceBindingManifestEntry,
    ComponentSystemBindingManifestEntry, Composition, Fabric, FabricBuildError, FabricContribution,
    FabricManifest, FabricManifestDiagnostics, IntoFabricContribution, IntoFabricResource,
    IntoFabricResourceAugmentation, IntoFabricSystem, IntoFabricSystemAugmentation,
    RealizationInspection, ResourceAugmentationInspection, ResourceAugmentationManifestEntry,
    ResourceInspection, ResourceManifestEntry, SemanticApiEndpoint, SemanticApiMetadata,
    SemanticRealizationKind, SemanticRelationBindingManifestEntry, SemanticRelationOwner,
    SemanticRelationTargetDefinition, SemanticRelationTargetOccurrence,
    SystemAugmentationInspection, SystemAugmentationManifestEntry, SystemInspection,
    SystemManifestEntry,
};
pub(crate) use fabric::{RelationDeclarationOwner, RelationDeclarationProvenance};
pub use fabric_builder::FabricBuilder;
pub use instance::{
    BoundComponent, ComponentLiveObservation, ComponentObservedParticipation,
    ComponentReconciliationObservation, ComponentReconciliationOutcome,
    ComponentReconciliationResult, Instance, InstanceComponents, InstanceObservation,
    LocalRealizationObservation, LocalRealizationRole, ResourceLiveObservation,
    SemanticRealizationObservation, SystemLiveObservation,
};
pub use runtime_authoring::{
    AdapterRuntimeState, RuntimeContext, RuntimeState, StatefulAdapterDefinition,
    StatefulRuntimeAuthoring,
};
pub use system::{
    AdaptableSystemDefinition, PrimarySystemContract, SystemAugmentation,
    SystemAugmentationDefinition, SystemAugmentationError, SystemAugmentationRealization,
    SystemAugmentationRequirement, SystemAugmentationSupportDefinition, SystemDefinition,
    SystemRealization, SystemRequires, SystemSelection,
};

/// Advanced handwritten Component definition and realization APIs.
///
/// Macro users normally need only `component!` from the umbrella crate. This
/// module intentionally groups the manual seams so they do not define normal
/// Component authoring vocabulary.
pub mod component {
    pub use super::{
        AdaptableComponentDefinition, ComponentDefinition, ComponentRealization,
        ComponentRealizationContract, ComponentRelationCompatibility, ComponentRelationTarget,
        ComponentResourceRequirement, ComponentResourceScope, ComponentSpec, ComponentSystemScope,
        SelfRealizingComponentDefinition,
    };
    pub use crate::component::advanced::{
        ComponentParticipationPreparation, ComponentParticipationRealization,
        ComponentParticipationScope,
    };
}
