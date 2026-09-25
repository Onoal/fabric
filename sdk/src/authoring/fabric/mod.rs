mod augmentation;
mod builder;
mod manifest;
mod resource;
mod sealed;
mod system;
mod system_augmentation;

pub use augmentation::IntoFabricResourceAugmentation;
pub use builder::{
    Composition, Fabric, FabricBuildError, FabricContribution, IntoFabricContribution,
};
pub(crate) use manifest::{
    AdapterRealizationMode, RealizationProvenance, RelationDeclarationOwner,
    RelationDeclarationProvenance,
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
pub use resource::IntoFabricResource;
pub use system::IntoFabricSystem;
pub use system_augmentation::IntoFabricSystemAugmentation;
