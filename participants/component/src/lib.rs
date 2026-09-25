//! Runtime and integration primitives for Fabric Components.
//!
//! A Component is a semantic behavioral participant. Its declaration owns an
//! identity and may declare Config, Relations, and an API. A realization is
//! optional and produces a generation-scoped [`ComponentParticipation`]. This
//! crate owns that participation host, typed invocation machinery, and the
//! deliberate integration seams around them; it does not define a Component's
//! semantic API or choose Resource/System realizations.
//!
//! Normal authors should use [`onoal_fabric`](https://docs.rs/onoal-fabric)
//! and `fabric::component!`. The named modules below are for advanced
//! integrations: [`declaration`], [`participation`], [`invocation`],
//! [`operator`], and [`advanced`].

mod control;
pub mod declaration;
mod error;
pub mod invocation;
mod lifecycle;
mod operation;
mod operational;
pub mod participation;
mod readiness;
mod registry;
mod requirement;
mod runtime;

#[cfg(test)]
mod catalog_tests;
#[cfg(test)]
mod communication_tests;
#[cfg(test)]
mod participation_tests;
#[cfg(test)]
mod readiness_tests;
#[cfg(test)]
mod reconstruction_tests;
#[cfg(test)]
mod requirement_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod source_guards;

#[doc(hidden)]
pub use control::{
    ComponentControl, ComponentControlRail, ComponentControlService, ComponentDesiredState,
    component_control_contract_id, component_control_contract_key,
};
#[doc(hidden)]
pub use control::{ComponentControlSnapshot, ComponentControlSnapshotEntry};
#[doc(hidden)]
pub use control::{
    ComponentReconstructionOutcome, ComponentReconstructionRail, ComponentReconstructionReport,
    ComponentReconstructionResult, ComponentReconstructionRuntimeState,
    ComponentReconstructionService, component_reconstruction_contract_id,
    component_reconstruction_contract_key,
};
pub use declaration::ComponentId;
#[doc(hidden)]
pub use declaration::{
    ComponentApiEndpoint, ComponentApiMetadata, ComponentDeclaration, ComponentRelationDeclaration,
    ComponentRelationName, ComponentResourceRequirementDeclaration,
    ComponentSystemRequirementDeclaration,
};
pub use error::ComponentError;
#[doc(hidden)]
pub use error::{
    ComponentParticipationCleanupError, ComponentParticipationCleanupFailure,
    ComponentParticipationPreparationPhase,
};
#[doc(hidden)]
pub use fabric_core::{InstanceGeneration, InstanceId};
#[doc(hidden)]
pub use invocation::{
    ComponentCommunication, ComponentCommunicationService, ComponentContract,
    ProvidedComponentContract, component_communication_contract_id,
    component_communication_contract_key,
};
#[doc(hidden)]
pub use invocation::{
    InvocationContext, InvocationId, InvocationOrigin, InvocationRail, InvocationService,
    invocation_contract_id, invocation_contract_key,
};
#[doc(hidden)]
pub use invocation::{
    OperationFuture, OperationRail, OperationRailService, OperationRegistrar,
    OperationRegistrarService, operation_rail_contract_id, operation_rail_contract_key,
    operation_registrar_contract_id, operation_registrar_contract_key,
};
#[doc(hidden)]
pub use invocation::{
    Surface, SurfaceId, SurfaceRegistry, SurfaceRegistryService, surface_contract_id,
    surface_contract_key,
};
#[doc(hidden)]
pub use lifecycle::{ComponentHostLifecycle, ComponentHostStatus};
#[doc(hidden)]
pub use operation::{
    OperationDefinition, OperationDescriptor, OperationId, OperationKey, OperationTypeId,
};
#[doc(hidden)]
pub use operational::{
    ComponentHostHandle, component_host_handle_contract_id, component_host_handle_contract_key,
};
#[doc(hidden)]
pub use participation::ComponentInstanceBinding;
#[doc(hidden)]
pub use participation::{ComponentInvocation, ComponentScope};
#[doc(hidden)]
pub use participation::{ComponentParticipation, ComponentParticipationId};
#[doc(hidden)]
pub use readiness::{
    ComponentAggregateBlocker, ComponentAggregateReadiness, ComponentReadinessPolicy,
    ComponentReadinessRail, ComponentReadinessService, component_readiness_contract_id,
    component_readiness_contract_key,
};
#[doc(hidden)]
pub use registry::{
    ComponentRegistry, ComponentRegistryService, ComponentStatus, ParticipationState,
    component_registry_contract_id, component_registry_contract_key,
};
#[doc(hidden)]
pub use requirement::{
    ComponentDependencyAvailabilityBlocker, ComponentDependencyHealthBlocker,
    ComponentEffectiveAvailability, ComponentEffectiveHealth, ComponentRequirementKind,
    ComponentRequirementRail, ComponentRequirementService, DegradedComponentHealth,
    ResolvedComponentRequirement, component_requirement_contract_id,
    component_requirement_contract_key,
};
#[doc(hidden)]
pub use runtime::ComponentHostModule;
#[doc(hidden)]
pub use runtime::{
    ComponentAugmentationParticipationPreparation, ComponentAugmentationParticipationRealization,
    ComponentMaterializer, ComponentMaterializerService, ComponentParticipationCleanup,
    ComponentParticipationContribution, ComponentParticipationPreparation,
    ComponentParticipationRealization, ComponentParticipationScope, ComponentResourceDependency,
    component_materializer_contract_id, component_materializer_contract_key,
    component_named_resource_dependency_contract_key,
    component_named_system_dependency_contract_key, component_resource_dependency_contract_id,
    component_resource_dependency_contract_key, component_system_dependency_contract_key,
};
#[doc(hidden)]
pub use runtime::{
    ComponentHost, ComponentHostService, component_host_contract_id, component_host_contract_key,
};

/// Instance-local host, control, readiness, reconstruction, and observation APIs.
///
/// Requirement and effective-health values in this module are live operator
/// projections. They are not the semantic relations declared by a Component.
pub mod operator {
    pub use crate::{
        ComponentAggregateBlocker, ComponentAggregateReadiness, ComponentControl,
        ComponentControlRail, ComponentControlService, ComponentControlSnapshot,
        ComponentControlSnapshotEntry, ComponentDependencyAvailabilityBlocker,
        ComponentDependencyHealthBlocker, ComponentDesiredState, ComponentEffectiveAvailability,
        ComponentEffectiveHealth, ComponentHost, ComponentHostHandle, ComponentHostLifecycle,
        ComponentHostModule, ComponentHostService, ComponentHostStatus, ComponentMaterializer,
        ComponentMaterializerService, ComponentReadinessPolicy, ComponentReadinessRail,
        ComponentReadinessService, ComponentReconstructionOutcome, ComponentReconstructionRail,
        ComponentReconstructionReport, ComponentReconstructionResult,
        ComponentReconstructionRuntimeState, ComponentReconstructionService, ComponentRegistry,
        ComponentRegistryService, ComponentRequirementKind, ComponentRequirementRail,
        ComponentRequirementService, ComponentStatus, DegradedComponentHealth, ParticipationState,
        ResolvedComponentRequirement,
    };
}

/// Deliberate low-level realization, communication, surface, and rail APIs.
///
/// These APIs support manual integrations; they are not required for normal
/// `component!` declaration, realization, or invocation authoring.
pub mod advanced {
    pub use crate::{
        ComponentAugmentationParticipationPreparation,
        ComponentAugmentationParticipationRealization, ComponentCommunication,
        ComponentCommunicationService, ComponentContract, ComponentInvocation,
        ComponentParticipationCleanup, ComponentParticipationContribution,
        ComponentParticipationPreparation, ComponentParticipationPreparationPhase,
        ComponentParticipationRealization, ComponentParticipationScope,
        ComponentResourceDependency, ComponentScope, ComponentSystemRequirementDeclaration,
        ProvidedComponentContract, Surface, SurfaceId, SurfaceRegistry, SurfaceRegistryService,
    };
}
