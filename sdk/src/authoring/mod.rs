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
    AdaptableComponentDefinition, AdaptableResourceDefinition, AdapterDefinition,
    AdapterProviderModule, ComponentAugmentation, ComponentAugmentationDefinition,
    ComponentAugmentationRealization, ComponentAugmentationRequirement, ComponentAugmentationSet,
    ComponentAugmentationSetAdapterRealization, ComponentAugmentationSetAttachment,
    ComponentAugmentationSetRealization, ComponentAugmentationSupportDefinition,
    ComponentAugmentedAdapterRealization, ComponentDefinition, ComponentRealization,
    ComponentRealizationContract, ComponentResourceRequirement, ComponentResourceScope,
    ComponentSpec, ComponentSystemScope, ContractDependency, IntoResourceName,
    PrimaryResourceContract, RelationTarget, Requires, ResourceAugmentation,
    ResourceAugmentationDefinition, ResourceAugmentationError, ResourceAugmentationRealization,
    ResourceAugmentationRequirement, ResourceAugmentationSupportDefinition, ResourceDefinition,
    ResourceRealization, ResourceSelection, SelfRealizingComponentDefinition,
};
pub use fabric::{
    BuiltFabric, ComponentAugmentationManifestEntry, ComponentResourceBindingManifestEntry,
    ComponentSystemBindingManifestEntry, Fabric, FabricBuildError, FabricManifest,
    FabricManifestDiagnostics, IntoFabricResource, IntoFabricResourceAugmentation,
    IntoFabricSystem, IntoFabricSystemAugmentation, ResourceAugmentationManifestEntry,
    ResourceManifestEntry, SystemAugmentationManifestEntry, SystemManifestEntry,
};
pub use fabric_builder::FabricBuilder;
pub use fabric_instance::{FabricComponents, FabricInstance};
pub use runtime_authoring::{
    RuntimeContext, RuntimeState, StatefulAdapterDefinition, StatefulRuntimeAuthoring,
};
pub use system::{
    AdaptableSystemDefinition, PrimarySystemContract, SystemAugmentation,
    SystemAugmentationDefinition, SystemAugmentationError, SystemAugmentationRealization,
    SystemAugmentationRequirement, SystemAugmentationSupportDefinition, SystemDefinition,
    SystemRealization, SystemRequires, SystemSelection,
};
