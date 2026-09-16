#![forbid(unsafe_code)]

pub mod adapter;
pub mod authoring;
pub mod component;
pub mod contracts;
pub mod core;
pub mod host;
pub mod ids;
pub mod prelude;
pub mod resource;
pub mod system;
pub mod versions;

mod error;

#[cfg(test)]
mod source_guards;
#[cfg(test)]
mod tests;

pub use fabric_sdk_macros::{adapter, component, resource, system};

pub use authoring::{
    AdaptableComponentDefinition, AdaptableResourceDefinition, AdaptableSystemDefinition,
    AdapterDefinition, AdapterProviderModule, BuiltFabric, ComponentDefinition,
    ComponentRealization, ComponentResourceBindingManifestEntry, ComponentResourceRequirement,
    ComponentResourceScope, ComponentSpec, ComponentSystemBindingManifestEntry,
    ComponentSystemScope, ContractDependency, Fabric, FabricBuildError, FabricComponents,
    FabricInstance, FabricManifest, IntoFabricResource, IntoFabricResourceAugmentation,
    IntoFabricSystem, IntoFabricSystemAugmentation, IntoResourceName, PrimaryResourceContract,
    PrimarySystemContract, Requires, ResourceAugmentation, ResourceAugmentationDefinition,
    ResourceAugmentationError, ResourceAugmentationManifestEntry, ResourceAugmentationRealization,
    ResourceAugmentationRequirement, ResourceAugmentationSupportDefinition, ResourceDefinition,
    ResourceManifestEntry, ResourceRealization, ResourceSelection,
    SelfRealizingComponentDefinition, SystemAugmentation, SystemAugmentationDefinition,
    SystemAugmentationError, SystemAugmentationManifestEntry, SystemAugmentationRealization,
    SystemAugmentationRequirement, SystemAugmentationSupportDefinition, SystemDefinition,
    SystemManifestEntry, SystemRealization, SystemRequires, SystemSelection,
};
pub use error::SdkAuthoringError;
