//! Optional convenience imports for Fabric's normal high-level authoring API.
//!
//! `use fabric::*` is the canonical root-first style. This prelude mirrors
//! that surface; handwritten authoring is intentionally available only through
//! [`crate::authoring`], and raw orchestration only through [`crate::core`].

pub use crate::{
    AdapterRealizedOwner, ComponentAugmentationInspection, ComponentDesiredState, ComponentError,
    ComponentId, ComponentInspection, Composition, CompositionError, CompositionId, Fabric,
    FabricBuildError, FabricContribution, Health, HostArchitecture, HostCompatibilityError,
    HostDescriptor, HostFacilityId, HostOperatingSystem, HostRequirement, Instance, InstanceError,
    InstanceGeneration, InstanceId, IntoFabricContribution, IntoFabricResource,
    IntoFabricResourceAugmentation, IntoFabricSystem, IntoFabricSystemAugmentation, LifecycleState,
    RealizationInspection, ResourceAugmentationInspection, ResourceCompatibilityError, ResourceId,
    ResourceInspection, ResourceName, ResourceSelection, RuntimeCleanupError, SdkAuthoringError,
    SemanticApiEndpoint, SemanticApiMetadata, SemanticRealizationKind,
    SemanticRelationBindingManifestEntry, SemanticRelationOwner, SemanticRelationTargetDefinition,
    SemanticRelationTargetOccurrence, SystemAugmentationInspection, SystemCompatibilityError,
    SystemId, SystemInspection, SystemSelection, adapter, component, resource, system,
};
