mod augmentation;
mod builder;
mod resource;
mod sealed;
mod system;
mod system_augmentation;

pub use crate::composition::SemanticApiMetadata;
pub use augmentation::IntoFabricResourceAugmentation;
pub use builder::{Fabric, FabricBuildError, FabricContribution, IntoFabricContribution};
pub use resource::IntoFabricResource;
pub use system::IntoFabricSystem;
pub use system_augmentation::IntoFabricSystemAugmentation;
