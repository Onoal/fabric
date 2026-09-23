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

mod communication;
mod component;
mod component_scope;
mod contract;
mod control;
mod control_snapshot;
pub mod declaration;
mod error;
pub mod invocation;
mod lifecycle;
mod native;
mod operation;
mod operational;
mod operations;
pub mod participation;
mod readiness;
mod reconstruction;
mod registry;
mod requirement;
mod runtime;
mod surface;

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
pub use communication::{
    ComponentCommunication, ComponentCommunicationService, ComponentContract,
    ProvidedComponentContract, component_communication_contract_id,
    component_communication_contract_key,
};
pub use component::ComponentId;
#[doc(hidden)]
pub use component::ComponentInstanceBinding;
#[doc(hidden)]
pub use component_scope::{ComponentInvocation, ComponentScope};
#[doc(hidden)]
pub use contract::{
    ComponentHost, ComponentHostService, component_host_contract_id, component_host_contract_key,
};
#[doc(hidden)]
pub use control::{
    ComponentControl, ComponentControlRail, ComponentControlService, ComponentDesiredState,
    component_control_contract_id, component_control_contract_key,
};
#[doc(hidden)]
pub use control_snapshot::{ComponentControlSnapshot, ComponentControlSnapshotEntry};
#[doc(hidden)]
pub use declaration::{
    ComponentDeclaration, ComponentRelationDeclaration, ComponentRelationName,
    ComponentResourceRequirementDeclaration, ComponentResourceRequirementName,
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
    InvocationContext, InvocationId, InvocationOrigin, InvocationRail, InvocationService,
    invocation_contract_id, invocation_contract_key,
};
#[doc(hidden)]
pub use lifecycle::{ComponentHostLifecycle, ComponentHostStatus};
#[doc(hidden)]
pub use native::ComponentHostModule;
#[doc(hidden)]
pub use operation::{
    OperationDefinition, OperationDescriptor, OperationId, OperationKey, OperationTypeId,
};
#[doc(hidden)]
pub use operational::{
    ComponentHostHandle, component_host_handle_contract_id, component_host_handle_contract_key,
};
#[doc(hidden)]
pub use operations::{
    OperationFuture, OperationRail, OperationRailService, OperationRegistrar,
    OperationRegistrarService, operation_rail_contract_id, operation_rail_contract_key,
    operation_registrar_contract_id, operation_registrar_contract_key,
};
#[doc(hidden)]
pub use participation::{ComponentParticipation, ComponentParticipationId};
#[doc(hidden)]
pub use readiness::{
    ComponentAggregateBlocker, ComponentAggregateReadiness, ComponentReadinessPolicy,
    ComponentReadinessRail, ComponentReadinessService, component_readiness_contract_id,
    component_readiness_contract_key,
};
#[doc(hidden)]
pub use reconstruction::{
    ComponentReconstructionOutcome, ComponentReconstructionRail, ComponentReconstructionReport,
    ComponentReconstructionResult, ComponentReconstructionRuntimeState,
    ComponentReconstructionService, component_reconstruction_contract_id,
    component_reconstruction_contract_key,
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
pub use runtime::{
    ComponentAugmentationParticipationPreparation, ComponentAugmentationParticipationRealization,
    ComponentMaterializer, ComponentMaterializerService, ComponentParticipationCleanup,
    ComponentParticipationContribution, ComponentParticipationPreparation,
    ComponentParticipationRealization, ComponentParticipationScope, ComponentResourceDependency,
    component_materializer_contract_id, component_materializer_contract_key,
    component_named_resource_dependency_contract_key, component_resource_dependency_contract_id,
    component_resource_dependency_contract_key, component_system_dependency_contract_key,
};
#[doc(hidden)]
pub use surface::{
    Surface, SurfaceId, SurfaceRegistry, SurfaceRegistryService, surface_contract_id,
    surface_contract_key,
};

/// Instance-local host, control, readiness, reconstruction, and observation APIs.
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
