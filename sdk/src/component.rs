//! Component APIs grouped by architectural ownership.
//!
//! Normal applications use [`crate::component!`] plus the umbrella root. The
//! families below are deliberate escape hatches for reading declarations,
//! inspecting a live participation, invoking endpoints, operating a host, or
//! writing handwritten integrations.

/// Runtime-free semantic Component declaration metadata.
pub mod declaration {
    pub use fabric_component::{
        ComponentDeclaration, ComponentId, ComponentRelationDeclaration, ComponentRelationName,
    };
}

/// Generation-scoped live Component occurrence identity and diagnostics.
pub mod participation {
    pub use fabric_component::{
        ComponentError, ComponentInstanceBinding, ComponentParticipation,
        ComponentParticipationCleanupError, ComponentParticipationCleanupFailure,
        ComponentParticipationId, ComponentParticipationPreparationPhase,
    };
}

/// Typed API endpoint invocation and optional provenance APIs.
pub mod invocation {
    pub use fabric_component::{
        InvocationContext, InvocationId, InvocationOrigin, InvocationRail, InvocationService,
        OperationDefinition, OperationDescriptor, OperationFuture, OperationId, OperationKey,
        OperationRail, OperationRailService, OperationTypeId,
    };
}

/// Instance-local host, control, readiness, reconstruction, and observation APIs.
///
/// Requirement and health values here are live operator projections, not
/// semantic Component relation declarations.
pub mod operator {
    pub use fabric_component::operator::*;
}

/// Handwritten realization, communication, surface, and low-level rail APIs.
///
/// Normal macro users do not need this module.
pub mod advanced {
    pub use fabric_component::advanced::*;
    pub use fabric_component::{
        ComponentResourceRequirementDeclaration, ComponentResourceRequirementName,
        ComponentSystemRequirementDeclaration, OperationRegistrar, OperationRegistrarService,
    };
}

// Macro expansions from downstream crates require these stable paths. They are
// intentionally hidden: normal Component vocabulary is exposed by the named
// families above and by the umbrella crate root.
#[doc(hidden)]
pub use fabric_component::{
    ComponentAggregateBlocker, ComponentAggregateReadiness,
    ComponentAugmentationParticipationPreparation, ComponentCommunication,
    ComponentCommunicationService, ComponentContract, ComponentControl, ComponentControlRail,
    ComponentControlService, ComponentControlSnapshot, ComponentControlSnapshotEntry,
    ComponentDeclaration, ComponentDependencyAvailabilityBlocker, ComponentDependencyHealthBlocker,
    ComponentDesiredState, ComponentEffectiveAvailability, ComponentEffectiveHealth,
    ComponentError, ComponentHost, ComponentHostLifecycle, ComponentHostModule,
    ComponentHostService, ComponentHostStatus, ComponentId, ComponentInstanceBinding,
    ComponentInvocation, ComponentMaterializer, ComponentMaterializerService,
    ComponentParticipation, ComponentParticipationCleanup, ComponentParticipationCleanupError,
    ComponentParticipationCleanupFailure, ComponentParticipationContribution,
    ComponentParticipationId, ComponentParticipationPreparation,
    ComponentParticipationPreparationPhase, ComponentParticipationRealization,
    ComponentParticipationScope, ComponentReadinessPolicy, ComponentReadinessRail,
    ComponentReadinessService, ComponentReconstructionOutcome, ComponentReconstructionRail,
    ComponentReconstructionReport, ComponentReconstructionResult,
    ComponentReconstructionRuntimeState, ComponentReconstructionService, ComponentRegistry,
    ComponentRegistryService, ComponentRelationDeclaration, ComponentRelationName,
    ComponentRequirementKind, ComponentRequirementRail, ComponentRequirementService,
    ComponentResourceRequirementName, ComponentScope, ComponentStatus, DegradedComponentHealth,
    InvocationContext, InvocationId, InvocationOrigin, InvocationRail, InvocationService,
    OperationDefinition, OperationDescriptor, OperationFuture, OperationId, OperationKey,
    OperationRail, OperationRailService, OperationRegistrar, OperationRegistrarService,
    OperationTypeId, ParticipationState, ProvidedComponentContract, ResolvedComponentRequirement,
    Surface, SurfaceId, SurfaceRegistry, SurfaceRegistryService,
    component_communication_contract_id, component_communication_contract_key,
    component_control_contract_id, component_control_contract_key, component_host_contract_id,
    component_host_contract_key, component_materializer_contract_id,
    component_materializer_contract_key, component_readiness_contract_id,
    component_readiness_contract_key, component_reconstruction_contract_id,
    component_reconstruction_contract_key, component_registry_contract_id,
    component_registry_contract_key, component_requirement_contract_id,
    component_requirement_contract_key, invocation_contract_id, invocation_contract_key,
    operation_rail_contract_id, operation_rail_contract_key, operation_registrar_contract_id,
    operation_registrar_contract_key, surface_contract_id, surface_contract_key,
};
