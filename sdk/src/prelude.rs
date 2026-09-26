//! Optional convenience imports for Fabric's normal high-level authoring API.
//!
//! `use fabric::*` is the canonical root-first style. This prelude mirrors
//! that surface; handwritten authoring is intentionally available only through
//! [`crate::authoring`], and raw orchestration only through [`crate::core`].

pub use crate::{
    AdapterRealizedOwner, BoundComponent, ComponentAugmentationInspection, ComponentDesiredState,
    ComponentError, ComponentId, ComponentInspection, ComponentLiveObservation,
    ComponentObservedParticipation, ComponentReconciliationObservation,
    ComponentReconciliationOutcome, ComponentReconciliationResult, Composition, CompositionError,
    CompositionId, Fabric, FabricBuildError, FabricContribution, Health, HostArchitecture,
    HostCompatibilityError, HostDescriptor, HostFacilityId, HostOperatingSystem, HostRequirement,
    Instance, InstanceError, InstanceFacility, InstanceFacilityContext, InstanceFacilityError,
    InstanceFacilityName, InstanceFacilityObservation, InstanceGeneration, InstanceId,
    InstanceObservation, IntoFabricContribution, IntoFabricResource,
    IntoFabricResourceAugmentation, IntoFabricSystem, IntoFabricSystemAugmentation, LifecycleState,
    LocalRealizationObservation, LocalRealizationRole, MaterializationPlan,
    MaterializationPlanProvenance, MaterializationProfile, MaterializationProfileName,
    RealizationInspection, ResourceAugmentationInspection, ResourceCompatibilityError, ResourceId,
    ResourceInspection, ResourceLiveObservation, ResourceName, ResourceSelection,
    RuntimeCleanupError, SdkAuthoringError, SemanticApiEndpoint, SemanticApiMetadata,
    SemanticRealizationKind, SemanticRealizationObservation, SemanticRelationBindingManifestEntry,
    SemanticRelationOwner, SemanticRelationTargetDefinition, SemanticRelationTargetOccurrence,
    SystemAugmentationInspection, SystemCompatibilityError, SystemId, SystemInspection,
    SystemLiveObservation, SystemSelection, adapter, component, resource, system,
};
