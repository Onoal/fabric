mod augmentation;
mod builder;
mod manifest;
mod resource;
mod sealed;
mod system;
mod system_augmentation;

pub use augmentation::IntoFabricResourceAugmentation;
pub use builder::{Composition, Fabric, FabricBuildError};
pub use manifest::{
    ComponentAugmentationManifestEntry, ComponentResourceBindingManifestEntry,
    ComponentSystemBindingManifestEntry, FabricManifest, FabricManifestDiagnostics,
    ResourceAugmentationManifestEntry, ResourceManifestEntry, SystemAugmentationManifestEntry,
    SystemManifestEntry,
};
pub use resource::IntoFabricResource;
pub use system::IntoFabricSystem;
pub use system_augmentation::IntoFabricSystemAugmentation;
