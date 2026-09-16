mod block_author;
mod composition_ext;
mod definitions;
mod fabric;
mod fabric_builder;
mod fabric_instance;
mod system;

pub use block_author::BlockAuthor;
pub use composition_ext::CompositionExt;
pub use definitions::{
    AdaptableComponentDefinition, AdaptableResourceDefinition, AdapterDefinition,
    AdapterProviderModule, ComponentDefinition, ComponentRealization, ComponentRealizationContract,
    ComponentResourceRequirement, ComponentResourceScope, ComponentSpec, ComponentSystemScope,
    ContractDependency, IntoResourceName, PrimaryResourceContract, Requires, ResourceAugmentation,
    ResourceAugmentationDefinition, ResourceAugmentationError, ResourceAugmentationRealization,
    ResourceAugmentationRequirement, ResourceAugmentationSupportDefinition, ResourceDefinition,
    ResourceRealization, ResourceSelection, SelfRealizingComponentDefinition,
};
pub use fabric::{
    BuiltFabric, ComponentResourceBindingManifestEntry, ComponentSystemBindingManifestEntry,
    Fabric, FabricBuildError, FabricManifest, FabricManifestDiagnostics, IntoFabricResource,
    IntoFabricResourceAugmentation, IntoFabricSystem, IntoFabricSystemAugmentation,
    ResourceAugmentationManifestEntry, ResourceManifestEntry, SystemAugmentationManifestEntry,
    SystemManifestEntry,
};
pub use fabric_builder::FabricBuilder;
pub use fabric_instance::{FabricComponents, FabricInstance};
pub use system::{
    AdaptableSystemDefinition, PrimarySystemContract, SystemAugmentation,
    SystemAugmentationDefinition, SystemAugmentationError, SystemAugmentationRealization,
    SystemAugmentationRequirement, SystemAugmentationSupportDefinition, SystemDefinition,
    SystemRealization, SystemRequires, SystemSelection,
};
