//! Optional convenience imports for Fabric's normal high-level authoring API.
//!
//! `use fabric::*` is the canonical root-first style. This prelude mirrors
//! that surface; handwritten authoring is intentionally available only through
//! [`crate::authoring`], and raw orchestration only through [`crate::core`].

pub use crate::{
    BuiltFabric, ComponentDefinition, ComponentError, ComponentId, ComponentRequirementKind,
    ComponentRuntimeContribution, ComponentRuntimePreparation, ComponentRuntimeTeardownError,
    ComponentRuntimeTeardownFailure, ComponentStatus, CompositionError, CompositionId, Fabric,
    FabricBuildError, FabricComponents, FabricInstance, Health, HostArchitecture,
    HostCompatibilityError, HostDescriptor, HostFacilityId, HostOperatingSystem, HostRequirement,
    InstanceError, InstanceGeneration, InstanceId, IntoFabricResource,
    IntoFabricResourceAugmentation, IntoFabricSystem, IntoFabricSystemAugmentation,
    InvocationContext, InvocationOrigin, LifecycleState, OperationId, OperationKey,
    OperationTypeId, ParticipationState, ResourceCompatibilityError, ResourceId, ResourceName,
    ResourceSelection, RuntimeCleanupError, SdkAuthoringError, SystemCompatibilityError, SystemId,
    SystemSelection, adapter, component, resource, system,
};
