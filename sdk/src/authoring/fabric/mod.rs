mod augmentation;
mod builder;
mod manifest;
mod resource;
mod sealed;
mod system;

pub use augmentation::IntoFabricResourceAugmentation;
pub use builder::{BuiltFabric, Fabric, FabricBuildError};
pub use manifest::{
    ComponentResourceBindingManifestEntry, ComponentSystemBindingManifestEntry, FabricManifest,
    FabricManifestDiagnostics, ResourceAugmentationManifestEntry, ResourceManifestEntry,
    SystemManifestEntry,
};
pub use resource::IntoFabricResource;
pub use system::IntoFabricSystem;
