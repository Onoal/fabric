mod augmentation_requirement;
mod block_author;
mod composition_ext;
mod definitions;
mod fabric;
mod fabric_builder;
mod fabric_instance;
mod runtime_authoring;
mod system;

pub(crate) use augmentation_requirement::requirement_for_key;
pub use block_author::BlockAuthor;
pub use composition_ext::CompositionExt;
pub use definitions::{
    AdaptableComponentDefinition, AdaptableResourceDefinition, AdapterBridgeMode,
    AdapterDefinition, AdapterProviderModule, CanonicalAdapterSupport,
    CanonicalComponentAdapterRuntime, ComponentAdapterCompatibility, ComponentAdapterTarget,
    ComponentAugmentation, ComponentAugmentationDefinition, ComponentAugmentationRealization,
    ComponentAugmentationRequirement, ComponentAugmentationSet,
    ComponentAugmentationSetAdapterRealization, ComponentAugmentationSetAttachment,
    ComponentAugmentationSetRealization, ComponentAugmentationSupportDefinition,
    ComponentAugmentedAdapterRealization, ComponentDefinition, ComponentRealization,
    ComponentRealizationContract, ComponentRelationCompatibility, ComponentRelationTarget,
    ComponentResourceRequirement, ComponentResourceScope, ComponentSpec, ComponentSystemScope,
    ContractDependency, IntoResourceName, PrimaryResourceContract, RelationTarget, Requires,
    ResourceAdapterCompatibility, ResourceAugmentation, ResourceAugmentationDefinition,
    ResourceAugmentationError, ResourceAugmentationRealization, ResourceAugmentationRequirement,
    ResourceAugmentationSupportDefinition, ResourceDefinition, ResourceRealization,
    ResourceSelection, SelfRealizingComponentDefinition,
};
pub use fabric::{
    ComponentAugmentationManifestEntry, ComponentResourceBindingManifestEntry,
    ComponentSystemBindingManifestEntry, Composition, Fabric, FabricBuildError, FabricManifest,
    FabricManifestDiagnostics, IntoFabricResource, IntoFabricResourceAugmentation,
    IntoFabricSystem, IntoFabricSystemAugmentation, ResourceAugmentationManifestEntry,
    ResourceManifestEntry, SystemAugmentationManifestEntry, SystemManifestEntry,
};
pub use fabric_builder::FabricBuilder;
pub use fabric_instance::{FabricComponents, FabricInstance};
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
