use std::error::Error;
use std::fmt;

use fabric_core::{InstanceGeneration, InstanceId};

use crate::{
    ComponentId, ComponentParticipation, ComponentRuntimeContribution, ComponentRuntimeLifecycle,
    OperationId, OperationTypeId, SurfaceId,
};

/// One best-effort cleanup failure from one Component participation
/// contribution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentRuntimeTeardownFailure {
    component_id: ComponentId,
    participation: ComponentParticipation,
    contribution: ComponentRuntimeContribution,
    source: Box<ComponentError>,
}

impl ComponentRuntimeTeardownFailure {
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    pub fn participation(&self) -> &ComponentParticipation {
        &self.participation
    }

    pub fn contribution(&self) -> ComponentRuntimeContribution {
        self.contribution
    }

    pub fn source(&self) -> &ComponentError {
        &self.source
    }

    pub(crate) fn new(
        participation: ComponentParticipation,
        contribution: ComponentRuntimeContribution,
        source: ComponentError,
    ) -> Self {
        Self {
            component_id: participation.component().component_id().clone(),
            participation,
            contribution,
            source: Box::new(source),
        }
    }
}

/// Aggregated best-effort cleanup evidence for one Component participation
/// operation. This remains Component-specific because preparation
/// contributions are not Core ModuleRuntime participants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentRuntimeTeardownError {
    failures: Vec<ComponentRuntimeTeardownFailure>,
}

impl ComponentRuntimeTeardownError {
    pub fn failures(&self) -> &[ComponentRuntimeTeardownFailure] {
        &self.failures
    }

    pub(crate) fn from_failures(failures: Vec<ComponentRuntimeTeardownFailure>) -> Option<Self> {
        (!failures.is_empty()).then_some(Self { failures })
    }
}

impl fmt::Display for ComponentRuntimeTeardownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Component participation teardown failed for {} contribution(s): ",
            self.failures.len()
        )?;
        for (index, failure) in self.failures.iter().enumerate() {
            if index > 0 {
                f.write_str("; ")?;
            }
            write!(
                f,
                "{} participation {} {:?}: {}",
                failure.component_id,
                failure.participation.participation_id(),
                failure.contribution,
                failure.source,
            )?;
        }
        Ok(())
    }
}

impl Error for ComponentRuntimeTeardownError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentRuntimeFailurePhase {
    Prepare,
    UpdateHealth,
    Activate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentError {
    InvalidComponentId(String),
    InvalidOperationId(String),
    InvalidOperationTypeId(String),
    InvalidSurfaceId(String),
    DuplicateOperationId(OperationId),
    DuplicateSurfaceId(SurfaceId),
    OperationOwnerInstanceMismatch {
        operation_id: OperationId,
        owner_instance_id: InstanceId,
        instance_id: InstanceId,
    },
    SurfaceOwnerInstanceMismatch {
        surface_id: SurfaceId,
        owner_instance_id: InstanceId,
        instance_id: InstanceId,
    },
    DuplicateComponentId(ComponentId),
    DuplicateComponentControlSnapshotEntry(ComponentId),
    ComponentControlSnapshotInstanceMismatch {
        snapshot_instance_id: InstanceId,
        instance_id: InstanceId,
    },
    ComponentRegistryInstanceMismatch {
        component_id: ComponentId,
        component_instance_id: InstanceId,
        instance_id: InstanceId,
    },
    ComponentControlInstanceMismatch {
        component_id: ComponentId,
        component_instance_id: InstanceId,
        instance_id: InstanceId,
    },
    OperationOwnerNotParticipating {
        operation_id: OperationId,
        component_id: ComponentId,
    },
    OperationOwnerNotPreparing {
        operation_id: OperationId,
        component_id: ComponentId,
    },
    ComponentContractProviderInstanceMismatch {
        component_id: ComponentId,
        component_instance_id: InstanceId,
        instance_id: InstanceId,
    },
    ComponentContractCallerInstanceMismatch {
        component_id: ComponentId,
        component_instance_id: InstanceId,
        instance_id: InstanceId,
    },
    ComponentContractProviderNotParticipating(ComponentId),
    ComponentContractCallerNotParticipating(ComponentId),
    ComponentContractProviderProvenanceMismatch {
        contract_id: fabric_core::ContractId,
        expected_provider_module: fabric_core::ModuleId,
        actual_provider_module: fabric_core::ModuleId,
    },
    ComponentContractConsumerMismatch {
        expected_component_id: ComponentId,
        actual_component_id: ComponentId,
    },
    InvocationContextInstanceMismatch {
        context_instance_id: InstanceId,
        instance_id: InstanceId,
    },
    InvocationContextGenerationMismatch {
        context_generation: InstanceGeneration,
        generation: InstanceGeneration,
    },
    InvocationOriginNotParticipating(ComponentId),
    InstanceGenerationUnbound,
    InvalidComponentRuntimeLifecycleTransition {
        from: ComponentRuntimeLifecycle,
        to: ComponentRuntimeLifecycle,
    },
    UnknownComponent(ComponentId),
    ComponentUnavailable(ComponentId),
    MissingComponentRuntimeAttachment(ComponentId),
    UndeclaredComponentOperation {
        component_id: ComponentId,
        operation_id: OperationId,
    },
    StaleComponentParticipation(ComponentParticipation),
    ComponentParticipationNotActive(ComponentParticipation),
    ComponentParticipationNotPreparing(ComponentParticipation),
    ComponentParticipationAlreadyActive(ComponentParticipation),
    DuplicateComponentRuntimeDefinition(ComponentId),
    ComponentRuntimeAlreadyMaterialized(ComponentId),
    ComponentRuntimeNotMaterialized(ComponentId),
    ComponentRuntimeMaterializationFailed {
        component_id: ComponentId,
        phase: ComponentRuntimeFailurePhase,
    },
    ComponentRuntimeMaterializationCleanupFailed {
        primary: Box<ComponentError>,
        cleanup: ComponentRuntimeTeardownError,
    },
    ComponentRuntimeTeardownFailed(ComponentRuntimeTeardownError),
    UnknownComponentControl(ComponentId),
    ComponentReconstructionUnavailableLifecycle(ComponentRuntimeLifecycle),
    UnknownOperation(OperationId),
    UnknownSurface(SurfaceId),
    OperationTypeMismatch(OperationId),
    OperationDefinitionTypeMismatch {
        operation_id: OperationId,
        expected_input_type: OperationTypeId,
        actual_input_type: OperationTypeId,
        expected_output_type: OperationTypeId,
        actual_output_type: OperationTypeId,
    },
    /// Component Adapter compatibility is derived from the target Component
    /// identity; schema-style `supports:` overrides are not meaningful.
    UnsupportedComponentAdapterSupportOverride,
    Unavailable,
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidComponentId(value) => {
                write!(f, "ComponentId `{value}` is invalid")
            }
            Self::InvalidOperationId(value) => {
                write!(f, "OperationId `{value}` is invalid")
            }
            Self::InvalidOperationTypeId(value) => {
                write!(f, "OperationTypeId `{value}` is invalid")
            }
            Self::InvalidSurfaceId(value) => {
                write!(f, "SurfaceId `{value}` is invalid")
            }
            Self::DuplicateOperationId(operation_id) => {
                write!(f, "OperationId `{operation_id}` is already registered")
            }
            Self::DuplicateSurfaceId(surface_id) => {
                write!(f, "SurfaceId `{surface_id}` is already registered")
            }
            Self::OperationOwnerInstanceMismatch {
                operation_id,
                owner_instance_id,
                instance_id,
            } => write!(
                f,
                "OperationId `{operation_id}` belongs to Instance `{owner_instance_id}` but this Component runtime owns `{instance_id}`"
            ),
            Self::SurfaceOwnerInstanceMismatch {
                surface_id,
                owner_instance_id,
                instance_id,
            } => write!(
                f,
                "SurfaceId `{surface_id}` belongs to Instance `{owner_instance_id}` but this Component runtime owns `{instance_id}`"
            ),
            Self::DuplicateComponentId(component_id) => {
                write!(f, "ComponentId `{component_id}` is already registered")
            }
            Self::DuplicateComponentControlSnapshotEntry(component_id) => write!(
                f,
                "ComponentControlSnapshot contains duplicate ComponentId `{component_id}`"
            ),
            Self::ComponentControlSnapshotInstanceMismatch {
                snapshot_instance_id,
                instance_id,
            } => write!(
                f,
                "ComponentControlSnapshot belongs to Instance `{snapshot_instance_id}` but this Component runtime owns `{instance_id}`"
            ),
            Self::ComponentRegistryInstanceMismatch {
                component_id,
                component_instance_id,
                instance_id,
            } => write!(
                f,
                "ComponentId `{component_id}` belongs to Instance `{component_instance_id}` but this Component runtime owns `{instance_id}`"
            ),
            Self::ComponentControlInstanceMismatch {
                component_id,
                component_instance_id,
                instance_id,
            } => write!(
                f,
                "ComponentId `{component_id}` belongs to Instance `{component_instance_id}` but this Component control rail owns `{instance_id}`"
            ),
            Self::OperationOwnerNotParticipating {
                operation_id,
                component_id,
            } => write!(
                f,
                "OperationId `{operation_id}` cannot be registered because ComponentId `{component_id}` is not participating"
            ),
            Self::OperationOwnerNotPreparing {
                operation_id,
                component_id,
            } => write!(
                f,
                "OperationId `{operation_id}` cannot be registered because ComponentId `{component_id}` is no longer preparing"
            ),
            Self::ComponentContractProviderInstanceMismatch {
                component_id,
                component_instance_id,
                instance_id,
            }
            | Self::ComponentContractCallerInstanceMismatch {
                component_id,
                component_instance_id,
                instance_id,
            } => write!(
                f,
                "ComponentId `{component_id}` belongs to Instance `{component_instance_id}` but this Component communication rail owns `{instance_id}`"
            ),
            Self::ComponentContractProviderNotParticipating(component_id) => write!(
                f,
                "Component contract provider `{component_id}` is not participating"
            ),
            Self::ComponentContractCallerNotParticipating(component_id) => write!(
                f,
                "Component contract caller `{component_id}` is not participating"
            ),
            Self::ComponentContractProviderProvenanceMismatch {
                contract_id,
                expected_provider_module,
                actual_provider_module,
            } => write!(
                f,
                "Component contract `{contract_id}` resolved from Module `{expected_provider_module}` but wrapper claimed Module `{actual_provider_module}`"
            ),
            Self::ComponentContractConsumerMismatch {
                expected_component_id,
                actual_component_id,
            } => write!(
                f,
                "Component contract handle is scoped to `{expected_component_id}` but caller `{actual_component_id}` attempted to use it"
            ),
            Self::InvocationContextInstanceMismatch {
                context_instance_id,
                instance_id,
            } => write!(
                f,
                "InvocationContext belongs to Instance `{context_instance_id}` but this Component runtime owns `{instance_id}`"
            ),
            Self::InvocationContextGenerationMismatch {
                context_generation,
                generation,
            } => write!(
                f,
                "InvocationContext belongs to InstanceGeneration `{context_generation}` but this Component runtime owns `{generation}`"
            ),
            Self::InvocationOriginNotParticipating(component_id) => write!(
                f,
                "Component invocation origin `{component_id}` is not participating"
            ),
            Self::InstanceGenerationUnbound => {
                write!(
                    f,
                    "Instance generation is not bound to a running Component runtime"
                )
            }
            Self::InvalidComponentRuntimeLifecycleTransition { from, to } => {
                write!(
                    f,
                    "Component runtime lifecycle transition from `{from:?}` to `{to:?}` is invalid"
                )
            }
            Self::UnknownComponent(component_id) => {
                write!(f, "ComponentId `{component_id}` is not registered")
            }
            Self::MissingComponentRuntimeAttachment(component_id) => {
                write!(
                    f,
                    "ComponentId `{component_id}` is declared but has no runtime attachment"
                )
            }
            Self::UndeclaredComponentOperation {
                component_id,
                operation_id,
            } => {
                write!(
                    f,
                    "OperationId `{operation_id}` is not declared by ComponentId `{component_id}`"
                )
            }
            Self::ComponentUnavailable(component_id) => {
                write!(f, "ComponentId `{component_id}` is unavailable")
            }
            Self::StaleComponentParticipation(participation) => write!(
                f,
                "ComponentParticipation `{}` for ComponentId `{}` is no longer current",
                participation.participation_id(),
                participation.component().component_id(),
            ),
            Self::ComponentParticipationNotActive(participation) => write!(
                f,
                "ComponentParticipation `{}` for ComponentId `{}` is not active",
                participation.participation_id(),
                participation.component().component_id(),
            ),
            Self::ComponentParticipationNotPreparing(participation) => write!(
                f,
                "ComponentParticipation `{}` for ComponentId `{}` is not preparing",
                participation.participation_id(),
                participation.component().component_id(),
            ),
            Self::ComponentParticipationAlreadyActive(participation) => write!(
                f,
                "ComponentParticipation `{}` for ComponentId `{}` is already active",
                participation.participation_id(),
                participation.component().component_id(),
            ),
            Self::DuplicateComponentRuntimeDefinition(component_id) => write!(
                f,
                "ComponentId `{component_id}` has multiple runtime definitions"
            ),
            Self::ComponentRuntimeAlreadyMaterialized(component_id) => write!(
                f,
                "ComponentId `{component_id}` already has a current runtime participation"
            ),
            Self::ComponentRuntimeNotMaterialized(component_id) => write!(
                f,
                "ComponentId `{component_id}` has no current runtime participation"
            ),
            Self::ComponentRuntimeMaterializationFailed {
                component_id,
                phase,
            } => write!(
                f,
                "ComponentId `{component_id}` runtime materialization failed during `{phase:?}`"
            ),
            Self::ComponentRuntimeMaterializationCleanupFailed { primary, cleanup } => write!(
                f,
                "Component runtime materialization failed: {primary}; teardown also failed: {cleanup}"
            ),
            Self::ComponentRuntimeTeardownFailed(cleanup) => write!(f, "{cleanup}"),
            Self::UnknownComponentControl(component_id) => {
                write!(
                    f,
                    "ComponentId `{component_id}` has no desired control state"
                )
            }
            Self::ComponentReconstructionUnavailableLifecycle(lifecycle) => {
                write!(
                    f,
                    "Component reconstruction cannot run while lifecycle is `{lifecycle:?}`"
                )
            }
            Self::UnknownOperation(operation_id) => {
                write!(f, "OperationId `{operation_id}` is not registered")
            }
            Self::UnknownSurface(surface_id) => {
                write!(f, "SurfaceId `{surface_id}` is not registered")
            }
            Self::OperationTypeMismatch(operation_id) => {
                write!(
                    f,
                    "OperationId `{operation_id}` was invoked with the wrong type"
                )
            }
            Self::OperationDefinitionTypeMismatch {
                operation_id,
                expected_input_type,
                actual_input_type,
                expected_output_type,
                actual_output_type,
            } => write!(
                f,
                "OperationId `{operation_id}` was registered with semantic types `{actual_input_type}` -> `{actual_output_type}` but runtime already owns `{expected_input_type}` -> `{expected_output_type}`"
            ),
            Self::UnsupportedComponentAdapterSupportOverride => f.write_str(
                "`supports:` is not valid for an Adapter targeting a Component; Component Adapter compatibility is derived from its target identity",
            ),
            Self::Unavailable => f.write_str("component runtime contract is unavailable"),
        }
    }
}

impl Error for ComponentError {}
