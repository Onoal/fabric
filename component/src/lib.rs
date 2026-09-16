mod communication;
mod component;
mod component_scope;
mod contract;
mod control;
mod control_snapshot;
mod declaration;
mod error;
mod invocation;
mod lifecycle;
mod native;
mod operation;
mod operational;
mod operations;
mod participation;
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

pub use communication::{
    ComponentCommunication, ComponentCommunicationService, ComponentContract,
    ProvidedComponentContract, component_communication_contract_id,
    component_communication_contract_key,
};
pub use component::{Component, ComponentId};
pub use component_scope::{ComponentInvocation, ComponentScope};
pub use contract::{
    ComponentRuntime, ComponentRuntimeService, component_runtime_contract_id,
    component_runtime_contract_key,
};
pub use control::{
    ComponentControl, ComponentControlRail, ComponentControlService, ComponentDesiredState,
    component_control_contract_id, component_control_contract_key,
};
pub use control_snapshot::{ComponentControlSnapshot, ComponentControlSnapshotEntry};
pub use declaration::{
    ComponentDeclaration, ComponentResourceRequirementDeclaration,
    ComponentResourceRequirementName, ComponentSystemRequirementDeclaration,
};
pub use error::{ComponentError, ComponentRuntimeFailurePhase};
pub use fabric_core::{InstanceGeneration, InstanceId};
pub use invocation::{
    InvocationContext, InvocationId, InvocationOrigin, InvocationRail, InvocationService,
    invocation_contract_id, invocation_contract_key,
};
pub use lifecycle::{ComponentRuntimeLifecycle, ComponentRuntimeStatus};
pub use native::ComponentRuntimeModule;
pub use operation::{
    OperationDefinition, OperationDescriptor, OperationId, OperationKey, OperationTypeId,
};
pub use operational::{
    ComponentRuntimeHandle, component_runtime_handle_contract_id,
    component_runtime_handle_contract_key,
};
pub use operations::{
    OperationFuture, OperationRail, OperationRailService, OperationRegistrar,
    OperationRegistrarService, operation_rail_contract_id, operation_rail_contract_key,
    operation_registrar_contract_id, operation_registrar_contract_key,
};
pub use participation::{ComponentParticipation, ComponentParticipationId};
pub use readiness::{
    ComponentAggregateBlocker, ComponentAggregateReadiness, ComponentReadinessPolicy,
    ComponentReadinessRail, ComponentReadinessService, component_readiness_contract_id,
    component_readiness_contract_key,
};
pub use reconstruction::{
    ComponentReconstructionOutcome, ComponentReconstructionRail, ComponentReconstructionReport,
    ComponentReconstructionResult, ComponentReconstructionRuntimeState,
    ComponentReconstructionService, component_reconstruction_contract_id,
    component_reconstruction_contract_key,
};
pub use registry::{
    ComponentRegistry, ComponentRegistryService, ComponentStatus, ParticipationState,
    component_registry_contract_id, component_registry_contract_key,
};
pub use requirement::{
    ComponentDependencyAvailabilityBlocker, ComponentDependencyHealthBlocker,
    ComponentEffectiveAvailability, ComponentEffectiveHealth, ComponentRequirementKind,
    ComponentRequirementRail, ComponentRequirementService, DegradedComponentHealth,
    ResolvedComponentRequirement, component_requirement_contract_id,
    component_requirement_contract_key,
};
pub use runtime::{
    ComponentAugmentationRuntimeDefinition, ComponentMaterializer, ComponentMaterializerService,
    ComponentResourceDependency, ComponentRuntimeDefinition, ComponentRuntimeScope,
    component_materializer_contract_id, component_materializer_contract_key,
    component_named_resource_dependency_contract_key, component_resource_dependency_contract_id,
    component_resource_dependency_contract_key, component_system_dependency_contract_key,
};
pub use surface::{
    Surface, SurfaceId, SurfaceRegistry, SurfaceRegistryService, surface_contract_id,
    surface_contract_key,
};
