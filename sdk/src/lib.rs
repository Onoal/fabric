//! The normal Rust entry point for Fabric.
//!
//! Import [`resource!`], [`system!`], [`adapter!`], and [`component!`] together
//! with the high-level authoring surface through `use fabric::*`. Fabric
//! separates semantic Resources/Systems/Components from concrete Adapter
//! realizations and from generation-scoped live runtime state.
//!
//! Start with the repository manual: <https://github.com/Onoal/fabric/tree/main/docs>.
//! Advanced Core and handwritten-authoring APIs stay available through named
//! modules rather than defining the normal learning path.

#![forbid(unsafe_code)]

pub mod adapter;
pub mod authoring;
pub mod component;
pub mod contracts;
pub mod core;
pub mod experimental;
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

// Root imports are deliberately the normal authoring surface. Handwritten
// definitions, provider machinery, runtime state, and contract internals live
// under `authoring`; raw orchestration remains under `core`.
pub use authoring::{
    BuiltFabric, Fabric, FabricBuildError, FabricComponents, FabricInstance, IntoFabricResource,
    IntoFabricResourceAugmentation, IntoFabricSystem, IntoFabricSystemAugmentation,
    ResourceSelection, SystemSelection,
};
pub use component::{ComponentError, ComponentId};
pub use core::{
    CompositionError, CompositionId, Health, InstanceError, InstanceGeneration, InstanceId,
    LifecycleState, RuntimeCleanupError,
};
pub use error::SdkAuthoringError;
pub use host::{
    HostArchitecture, HostCompatibilityError, HostDescriptor, HostFacilityId, HostOperatingSystem,
    HostRequirement,
};
pub use resource::{ResourceCompatibilityError, ResourceId, ResourceName};
pub use system::{SystemCompatibilityError, SystemId};
