//! Optional convenience imports for Fabric's normal high-level authoring API.
//!
//! `use fabric::*` is the canonical root-first style. This prelude mirrors
//! that surface; handwritten authoring is intentionally available only through
//! [`crate::authoring`], and raw orchestration only through [`crate::core`].

pub use crate::{
    ComponentError, ComponentId, Composition, CompositionError, CompositionId, Fabric,
    FabricBuildError, FabricComponents, FabricInstance, Health, HostArchitecture,
    HostCompatibilityError, HostDescriptor, HostFacilityId, HostOperatingSystem, HostRequirement,
    InstanceError, InstanceGeneration, InstanceId, IntoFabricResource,
    IntoFabricResourceAugmentation, IntoFabricSystem, IntoFabricSystemAugmentation, LifecycleState,
    ResourceCompatibilityError, ResourceId, ResourceName, ResourceSelection, RuntimeCleanupError,
    SdkAuthoringError, SystemCompatibilityError, SystemId, SystemSelection, adapter, component,
    resource, system,
};
