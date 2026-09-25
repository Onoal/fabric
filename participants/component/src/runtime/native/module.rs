use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

mod communication_runtime;
mod control_runtime;
mod invocation_runtime;
mod operation_runtime;
mod registry_runtime;
mod requirement_runtime;
mod state;
mod surface_runtime;

use fabric_core::{
    ContractRequirement, Health, InstanceId, InstanceRuntimeContext, Module, ModuleContract,
    ModuleError, ModuleId, ModuleRuntime,
};

use crate::control::ComponentControlSnapshot;
use crate::control::{
    ComponentReconstructionOutcome, ComponentReconstructionRail, ComponentReconstructionReport,
    ComponentReconstructionResult, ComponentReconstructionRuntimeState,
    ComponentReconstructionService, component_reconstruction_contract_key,
};
use crate::invocation::{
    ComponentCommunication, ComponentCommunicationService, component_communication_contract_key,
};
use crate::invocation::{
    OperationFuture, OperationRail, OperationRailService, OperationRegistrar,
    OperationRegistrarService, operation_rail_contract_key, operation_registrar_contract_key,
};
use crate::invocation::{
    Surface, SurfaceId, SurfaceRegistry, SurfaceRegistryService, surface_contract_key,
};
use crate::lifecycle::{ComponentHostLifecycle, ComponentHostStatus};
use crate::participation::ComponentScopeService;
use crate::readiness::{
    ComponentAggregateBlocker, ComponentAggregateReadiness, ComponentReadinessPolicy,
    ComponentReadinessRail, ComponentReadinessService, component_readiness_contract_key,
};
use crate::registry::{
    ComponentRegistry, ComponentRegistryService, ComponentStatus, ParticipationState,
    component_registry_contract_key,
};
use crate::requirement::{
    ComponentDependencyAvailabilityBlocker, ComponentDependencyHealthBlocker,
    ComponentEffectiveAvailability, ComponentEffectiveHealth, ComponentRequirementKind,
    ComponentRequirementRail, ComponentRequirementService, DegradedComponentHealth,
    ResolvedComponentRequirement, component_requirement_contract_key,
};
use crate::runtime::{
    ComponentAugmentationParticipationRealization, ComponentMaterializer,
    ComponentMaterializerService, ComponentParticipationCleanup,
    ComponentParticipationContribution, ComponentParticipationRealization,
    ComponentParticipationScope, ComponentResourceDependency, component_materializer_contract_key,
    component_named_resource_dependency_contract_key,
    component_named_system_dependency_contract_key, component_system_dependency_contract_key,
};
use crate::runtime::{ComponentHost, ComponentHostService, component_host_contract_key};
use crate::{
    ComponentControl, ComponentControlRail, ComponentControlService, ComponentDesiredState,
    component_control_contract_key,
};
use crate::{
    ComponentDeclaration, ComponentError, ComponentId, ComponentInstanceBinding,
    ComponentParticipation, ComponentParticipationId, OperationDefinition, OperationDescriptor,
    OperationId,
};
use state::{
    RegisteredOperation, bound_component, current_active_participation, current_participation,
    current_preparing_participation, effective_availability_from_health, effective_health,
    participation_is_available, project_aggregate_readiness, refresh_operational_status,
    validate_context_runtime, validated_catalog, validated_control_snapshot,
};

type ErasedOperationInput = Box<dyn std::any::Any + Send + Sync>;
type ErasedOperationOutput = Box<dyn std::any::Any + Send + Sync>;
type RegisteredOperationHandler = Arc<
    dyn Fn(crate::InvocationContext, ErasedOperationInput) -> OperationFuture<ErasedOperationOutput>
        + Send
        + Sync,
>;

pub struct ComponentHostModule {
    module_id: ModuleId,
    shared: Arc<SharedComponentState>,
}

struct SharedComponentState {
    inner: Mutex<ComponentModuleState>,
}

struct ComponentMaterializerAdapter {
    shared: Arc<SharedComponentState>,
}

struct ComponentReconstructionAdapter {
    shared: Arc<SharedComponentState>,
}

struct ComponentModuleState {
    status: ComponentHostStatus,
    readiness_policy: ComponentReadinessPolicy,
    component_declarations: BTreeMap<crate::ComponentId, ComponentDeclaration>,
    runtime_attachments: BTreeMap<crate::ComponentId, ComponentParticipationRealization>,
    augmentation_preparations:
        BTreeMap<crate::ComponentId, Vec<crate::ComponentAugmentationParticipationRealization>>,
    prepared_contributions: BTreeMap<crate::ComponentId, PreparedParticipation>,
    resource_dependencies: Arc<BTreeMap<fabric_core::ContractId, Arc<ComponentResourceDependency>>>,
    components: BTreeMap<crate::ComponentId, ComponentStatus>,
    control_snapshot: Option<ComponentControlSnapshot>,
    component_controls: BTreeMap<crate::ComponentId, ComponentControl>,
    operations: BTreeMap<OperationId, RegisteredOperation>,
    surfaces: BTreeMap<SurfaceId, Surface>,
    requirements: Vec<ResolvedComponentRequirement>,
    next_invocation_id: u64,
    next_participation_id: u64,
}

struct PreparedComponentContribution {
    kind: ComponentParticipationContribution,
    teardown: Option<ComponentParticipationCleanup>,
}

struct PreparedParticipation {
    participation: ComponentParticipation,
    contributions: Vec<PreparedComponentContribution>,
}

fn unbound_instance_id() -> InstanceId {
    InstanceId::new("fabric.component.runtime.unbound")
        .expect("static unbound component runtime instance id")
}

impl Default for ComponentHostModule {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentHostModule {
    pub fn new() -> Self {
        Self::with_readiness_policy(ComponentReadinessPolicy::empty())
    }

    pub fn with_readiness_policy(readiness_policy: ComponentReadinessPolicy) -> Self {
        Self::with_configuration(readiness_policy, Vec::new(), Vec::new())
            .expect("empty component runtime configuration must remain valid")
    }

    pub fn with_control_snapshot(
        control_snapshot: ComponentControlSnapshot,
    ) -> Result<Self, ComponentError> {
        Self::with_configuration_and_control_snapshot(
            ComponentReadinessPolicy::empty(),
            Vec::new(),
            Vec::new(),
            control_snapshot,
        )
    }

    pub fn with_components(
        declarations: impl IntoIterator<Item = ComponentDeclaration>,
        attachments: impl IntoIterator<Item = ComponentParticipationRealization>,
    ) -> Result<Self, ComponentError> {
        Self::with_components_and_augmentations(declarations, attachments, Vec::new())
    }

    /// Constructs the native host with one base attachment and zero or more
    /// additive preparation contributions per declared ComponentInstanceBinding.
    pub fn with_components_and_augmentations(
        declarations: impl IntoIterator<Item = ComponentDeclaration>,
        attachments: impl IntoIterator<Item = ComponentParticipationRealization>,
        augmentations: impl IntoIterator<Item = ComponentAugmentationParticipationRealization>,
    ) -> Result<Self, ComponentError> {
        Self::with_components_augmentations_and_initial_controls(
            declarations,
            attachments,
            augmentations,
            Vec::new(),
        )
    }

    /// Constructs the native host with declarative initial desired controls.
    ///
    /// These controls are instance-neutral until Core binds a fresh
    /// [`InstanceId`]. Explicit [`ComponentControlSnapshot`] restoration uses a
    /// separate constructor and remains Instance-specific live state.
    pub fn with_components_augmentations_and_initial_controls(
        declarations: impl IntoIterator<Item = ComponentDeclaration>,
        attachments: impl IntoIterator<Item = ComponentParticipationRealization>,
        augmentations: impl IntoIterator<Item = ComponentAugmentationParticipationRealization>,
        initial_controls: impl IntoIterator<Item = (ComponentId, ComponentDesiredState)>,
    ) -> Result<Self, ComponentError> {
        let augmentations = augmentations
            .into_iter()
            .fold(BTreeMap::new(), |mut values, value| {
                values
                    .entry(value.component_id().clone())
                    .or_insert_with(Vec::new)
                    .push(value);
                values
            });
        let module = Self::with_optional_control_snapshot(
            ComponentReadinessPolicy::empty(),
            declarations,
            attachments,
            None,
        )?;
        {
            let mut state = module
                .shared
                .inner
                .lock()
                .expect("component runtime state lock");
            for component_id in augmentations.keys() {
                if !state.component_declarations.contains_key(component_id) {
                    return Err(ComponentError::UnknownComponent(component_id.clone()));
                }
            }
            for (component_id, desired) in initial_controls {
                if !state.component_declarations.contains_key(&component_id) {
                    return Err(ComponentError::UnknownComponent(component_id));
                }
                if state
                    .component_controls
                    .insert(
                        component_id.clone(),
                        ComponentControl::new(
                            ComponentInstanceBinding::for_instance(
                                component_id.clone(),
                                unbound_instance_id(),
                            ),
                            desired,
                        ),
                    )
                    .is_some()
                {
                    return Err(ComponentError::DuplicateComponentControlSnapshotEntry(
                        component_id,
                    ));
                }
            }
            state.augmentation_preparations = augmentations;
        }
        Ok(module)
    }

    pub fn with_configuration(
        readiness_policy: ComponentReadinessPolicy,
        declarations: impl IntoIterator<Item = ComponentDeclaration>,
        attachments: impl IntoIterator<Item = ComponentParticipationRealization>,
    ) -> Result<Self, ComponentError> {
        Self::with_optional_control_snapshot(readiness_policy, declarations, attachments, None)
    }

    pub fn with_configuration_and_control_snapshot(
        readiness_policy: ComponentReadinessPolicy,
        declarations: impl IntoIterator<Item = ComponentDeclaration>,
        attachments: impl IntoIterator<Item = ComponentParticipationRealization>,
        control_snapshot: ComponentControlSnapshot,
    ) -> Result<Self, ComponentError> {
        Self::with_optional_control_snapshot(
            readiness_policy,
            declarations,
            attachments,
            Some(control_snapshot),
        )
    }

    fn with_optional_control_snapshot(
        readiness_policy: ComponentReadinessPolicy,
        declarations: impl IntoIterator<Item = ComponentDeclaration>,
        attachments: impl IntoIterator<Item = ComponentParticipationRealization>,
        control_snapshot: Option<ComponentControlSnapshot>,
    ) -> Result<Self, ComponentError> {
        let (component_declarations, runtime_attachments) =
            validated_catalog(declarations, attachments)?;
        let control_snapshot = validated_control_snapshot(control_snapshot)?;
        Ok(Self {
            module_id: ModuleId::new("fabric.component.runtime.module")
                .expect("static component runtime module id"),
            shared: Arc::new(SharedComponentState {
                inner: Mutex::new(ComponentModuleState {
                    status: ComponentHostStatus::new(
                        unbound_instance_id(),
                        None,
                        ComponentHostLifecycle::Stopped,
                        Health::Unavailable,
                    ),
                    readiness_policy,
                    component_declarations,
                    runtime_attachments,
                    augmentation_preparations: BTreeMap::new(),
                    prepared_contributions: BTreeMap::new(),
                    resource_dependencies: Arc::new(BTreeMap::new()),
                    components: BTreeMap::new(),
                    control_snapshot,
                    component_controls: BTreeMap::new(),
                    operations: BTreeMap::new(),
                    surfaces: BTreeMap::new(),
                    requirements: Vec::new(),
                    next_invocation_id: 1,
                    next_participation_id: 1,
                }),
            }),
        })
    }
}

impl SharedComponentState {
    fn record_prepared_contribution(
        &self,
        participation: &ComponentParticipation,
        kind: ComponentParticipationContribution,
        teardown: Option<ComponentParticipationCleanup>,
    ) -> Result<(), ComponentError> {
        let mut state = self.inner.lock().expect("component runtime state lock");
        if !current_preparing_participation(&state, participation) {
            return Err(ComponentError::StaleComponentParticipation(
                participation.clone(),
            ));
        }
        let entry = state
            .prepared_contributions
            .entry(participation.component().component_id().clone())
            .or_insert_with(|| PreparedParticipation {
                participation: participation.clone(),
                contributions: Vec::new(),
            });
        if entry.participation != *participation {
            return Err(ComponentError::StaleComponentParticipation(
                participation.clone(),
            ));
        }
        entry
            .contributions
            .push(PreparedComponentContribution { kind, teardown });
        Ok(())
    }

    /// Removes all participation-owned authority before invoking its
    /// occurrence-local teardown actions. Teardown then runs outside the host
    /// mutex, in reverse successful preparation order.
    fn unregister_with_teardown(
        &self,
        participation: &ComponentParticipation,
    ) -> Result<ComponentStatus, ComponentError> {
        let (status, contributions) = {
            let mut state = self.inner.lock().expect("component runtime state lock");
            let status = state
                .components
                .get(participation.component().component_id())
                .cloned()
                .ok_or_else(|| {
                    ComponentError::UnknownComponent(
                        participation.component().component_id().clone(),
                    )
                })?;
            if status.participation() != participation {
                return Err(ComponentError::StaleComponentParticipation(
                    participation.clone(),
                ));
            }
            state
                .components
                .remove(participation.component().component_id());
            state
                .operations
                .retain(|_, operation| operation.owner != *participation);
            state.surfaces.retain(|_, surface| {
                surface.owner().component_id() != participation.component().component_id()
            });
            let contributions = state
                .prepared_contributions
                .remove(participation.component().component_id())
                .filter(|prepared| prepared.participation == *participation)
                .map(|prepared| prepared.contributions)
                .unwrap_or_default();
            refresh_operational_status(&mut state);
            (status, contributions)
        };

        let mut failures = Vec::new();
        for contribution in contributions.into_iter().rev() {
            let Some(teardown) = contribution.teardown else {
                continue;
            };
            if let Err(source) = teardown() {
                failures.push(crate::ComponentParticipationCleanupFailure::new(
                    participation.clone(),
                    contribution.kind,
                    source,
                ));
            }
        }
        match crate::ComponentParticipationCleanupError::from_failures(failures) {
            Some(cleanup) => Err(ComponentError::ComponentParticipationCleanupFailed(cleanup)),
            None => Ok(status),
        }
    }
}

impl ModuleRuntime for ComponentHostModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![
            crate::component_host_handle_contract_key().declaration(),
            component_host_contract_key().declaration(),
            crate::invocation_contract_key().declaration(),
            component_communication_contract_key().declaration(),
            component_control_contract_key().declaration(),
            component_registry_contract_key().declaration(),
            component_materializer_contract_key().declaration(),
            component_reconstruction_contract_key().declaration(),
            operation_rail_contract_key().declaration(),
            operation_registrar_contract_key().declaration(),
            surface_contract_key().declaration(),
            component_readiness_contract_key().declaration(),
            component_requirement_contract_key().declaration(),
        ]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, fabric_core::ModuleError> {
        let service: Arc<dyn ComponentHostService> = self.shared.clone();
        let invocation: Arc<dyn crate::InvocationService> = self.shared.clone();
        let component_communication: Arc<dyn ComponentCommunicationService> = self.shared.clone();
        let component_control: Arc<dyn ComponentControlService> = self.shared.clone();
        let component_registry: Arc<dyn ComponentRegistryService> = self.shared.clone();
        let runtime_host: Arc<dyn ComponentMaterializerService> =
            Arc::new(ComponentMaterializerAdapter {
                shared: self.shared.clone(),
            });
        let reconstruction: Arc<dyn ComponentReconstructionService> =
            Arc::new(ComponentReconstructionAdapter {
                shared: self.shared.clone(),
            });
        let operation_rail: Arc<dyn OperationRailService> = self.shared.clone();
        let operation_registrar: Arc<dyn OperationRegistrarService> = self.shared.clone();
        let surface_registry: Arc<dyn SurfaceRegistryService> = self.shared.clone();
        let readiness: Arc<dyn ComponentReadinessService> = self.shared.clone();
        let requirements: Arc<dyn ComponentRequirementService> = self.shared.clone();
        let materializer = Arc::new(ComponentMaterializer::new(runtime_host));
        let control = Arc::new(ComponentControlRail::new(component_control));
        let registry = Arc::new(ComponentRegistry::new(component_registry));
        let reconstruction = Arc::new(ComponentReconstructionRail::new(reconstruction));
        let invocation_rail = Arc::new(crate::InvocationRail::new(invocation));
        let operations = Arc::new(OperationRail::new(operation_rail));
        let handle = crate::ComponentHostHandle::new(
            materializer.clone(),
            control.clone(),
            reconstruction.clone(),
            registry.clone(),
            invocation_rail.clone(),
            operations.clone(),
        );
        Ok(vec![
            ModuleContract::new(
                &crate::component_host_handle_contract_key(),
                Arc::new(handle),
            ),
            ModuleContract::new(
                &component_host_contract_key(),
                Arc::new(ComponentHost::new(service)),
            ),
            ModuleContract::new(&crate::invocation_contract_key(), invocation_rail),
            ModuleContract::new(
                &component_communication_contract_key(),
                Arc::new(ComponentCommunication::new(component_communication)),
            ),
            ModuleContract::new(&component_control_contract_key(), control),
            ModuleContract::new(&component_registry_contract_key(), registry),
            ModuleContract::new(&component_materializer_contract_key(), materializer),
            ModuleContract::new(&component_reconstruction_contract_key(), reconstruction),
            ModuleContract::new(&operation_rail_contract_key(), operations),
            ModuleContract::new(
                &operation_registrar_contract_key(),
                Arc::new(OperationRegistrar::new(operation_registrar)),
            ),
            ModuleContract::new(
                &surface_contract_key(),
                Arc::new(SurfaceRegistry::new(surface_registry)),
            ),
            ModuleContract::new(
                &component_readiness_contract_key(),
                Arc::new(ComponentReadinessRail::new(readiness)),
            ),
            ModuleContract::new(
                &component_requirement_contract_key(),
                Arc::new(ComponentRequirementRail::new(requirements)),
            ),
        ])
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        let state = self
            .shared
            .inner
            .lock()
            .expect("component runtime state lock");
        state
            .component_declarations
            .values()
            .flat_map(|declaration| {
                declaration
                    .resource_requirements()
                    .iter()
                    .map(move |requirement| {
                        ContractRequirement::<ComponentResourceDependency>::provisional(
                            component_named_resource_dependency_contract_key(
                                declaration.component_id(),
                                requirement.name(),
                                requirement.requirement(),
                            )
                            .id()
                            .clone(),
                        )
                        .declaration()
                        .clone()
                    })
            })
            .chain(
                state
                    .component_declarations
                    .values()
                    .flat_map(|declaration| {
                        declaration
                            .system_requirements()
                            .iter()
                            .map(move |requirement| {
                                ContractRequirement::<ComponentResourceDependency>::provisional(
                                    component_named_system_dependency_contract_key(
                                        declaration.component_id(),
                                        requirement.name(),
                                        requirement.requirement(),
                                    )
                                    .id()
                                    .clone(),
                                )
                                .declaration()
                                .clone()
                            })
                    }),
            )
            .collect()
    }

    fn bind(
        &mut self,
        bindings: &fabric_core::ModuleBindings,
    ) -> Result<(), fabric_core::ModuleError> {
        let declarations = self
            .shared
            .inner
            .lock()
            .expect("component runtime state lock")
            .component_declarations
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let mut dependencies = BTreeMap::new();
        for declaration in declarations {
            for requirement in declaration.resource_requirements() {
                let key = component_named_resource_dependency_contract_key(
                    declaration.component_id(),
                    requirement.name(),
                    requirement.requirement(),
                );
                let resolved = bindings
                    .resolve(
                        &ContractRequirement::<ComponentResourceDependency>::provisional(
                            key.id().clone(),
                        ),
                    )
                    .map_err(|error| ModuleError::new(error.to_string()))?;
                dependencies.insert(key.id().clone(), resolved);
            }
            for requirement in declaration.system_requirements() {
                let named_key = component_named_system_dependency_contract_key(
                    declaration.component_id(),
                    requirement.name(),
                    requirement.requirement(),
                );
                let resolved = bindings
                    .resolve(
                        &ContractRequirement::<ComponentResourceDependency>::provisional(
                            named_key.id().clone(),
                        ),
                    )
                    .map_err(|error| ModuleError::new(error.to_string()))?;
                if requirement.name().as_str() == requirement.requirement().id().as_str() {
                    let legacy_key = component_system_dependency_contract_key(
                        declaration.component_id(),
                        requirement.requirement(),
                    );
                    dependencies.insert(legacy_key.id().clone(), Arc::clone(&resolved));
                }
                dependencies.insert(named_key.id().clone(), resolved);
            }
        }
        self.shared
            .inner
            .lock()
            .expect("component runtime state lock")
            .resource_dependencies = Arc::new(dependencies);
        Ok(())
    }

    fn bind_instance_context(
        &mut self,
        context: &InstanceRuntimeContext,
    ) -> Result<(), ModuleError> {
        let mut state = self
            .shared
            .inner
            .lock()
            .expect("component runtime state lock");
        state
            .rebind_instance_context(context.instance_id().clone(), context.generation())
            .map_err(|error| ModuleError::new(error.to_string()))?;
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), fabric_core::ModuleError> {
        let mut state = self
            .shared
            .inner
            .lock()
            .expect("component runtime state lock");
        state
            .current_generation()
            .map_err(|error| ModuleError::new(error.to_string()))?;
        state
            .transition_to(ComponentHostLifecycle::Starting)
            .map_err(|error| fabric_core::ModuleError::new(error.to_string()))?;
        state.set_health(Health::Degraded);
        Ok(())
    }

    fn start(&mut self) -> Result<(), fabric_core::ModuleError> {
        let mut state = self
            .shared
            .inner
            .lock()
            .expect("component runtime state lock");
        state
            .transition_to(ComponentHostLifecycle::Ready)
            .map_err(|error| fabric_core::ModuleError::new(error.to_string()))?;
        let projected = project_aggregate_readiness(&state);
        state.set_health(projected.status().health());
        Ok(())
    }

    fn stop(&mut self) -> Result<(), fabric_core::ModuleError> {
        let participations = {
            let mut state = self
                .shared
                .inner
                .lock()
                .expect("component runtime state lock");
            if state.current_status().lifecycle() == ComponentHostLifecycle::Stopped {
                return Ok(());
            }
            state
                .transition_to(ComponentHostLifecycle::Stopping)
                .map_err(|error| fabric_core::ModuleError::new(error.to_string()))?;
            state.set_health(Health::Unavailable);
            state
                .components
                .values()
                .map(|status| status.participation().clone())
                .collect::<Vec<_>>()
        };

        let registry =
            ComponentRegistry::new(self.shared.clone() as Arc<dyn ComponentRegistryService>);
        let mut failures = Vec::new();
        for participation in participations {
            if let Err(error) = registry.unregister(&participation) {
                failures.push(error);
            }
        }

        let mut state = self
            .shared
            .inner
            .lock()
            .expect("component runtime state lock");
        state.operations.clear();
        state.surfaces.clear();
        state.components.clear();
        state.prepared_contributions.clear();
        state
            .transition_to(ComponentHostLifecycle::Stopped)
            .map_err(|error| fabric_core::ModuleError::new(error.to_string()))?;
        state.set_health(Health::Unavailable);
        if failures.is_empty() {
            Ok(())
        } else {
            Err(fabric_core::ModuleError::new(
                failures
                    .into_iter()
                    .map(|failure| failure.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            ))
        }
    }

    fn health(&self) -> Health {
        self.shared
            .inner
            .lock()
            .expect("component runtime state lock")
            .current_status()
            .health()
    }
}

impl Module for ComponentHostModule {
    fn declaration(&self) -> fabric_core::ModuleDeclaration {
        fabric_core::ModuleDeclaration::new(self.module_id.clone())
            .with_provided_contracts(self.provided_contract_declarations())
            .with_required_contracts(self.required_contract_declarations())
            .with_optional_contracts(self.optional_contract_declarations())
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        let state = self
            .shared
            .inner
            .lock()
            .expect("component module state lock");
        let readiness_policy = state.readiness_policy.clone();
        let declarations = state
            .component_declarations
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let attachments = state
            .runtime_attachments
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let augmentations = state
            .augmentation_preparations
            .values()
            .flat_map(|values| values.iter().cloned())
            .collect::<Vec<_>>();
        let control_snapshot = state.control_snapshot.clone();
        let component_controls = state.component_controls.clone();
        drop(state);
        let module = Self::with_optional_control_snapshot(
            readiness_policy,
            declarations,
            attachments,
            control_snapshot,
        )
        .expect("stored component runtime module definition must remain valid");
        module
            .shared
            .inner
            .lock()
            .expect("component runtime state lock")
            .augmentation_preparations =
            augmentations
                .into_iter()
                .fold(BTreeMap::new(), |mut values, value| {
                    values
                        .entry(value.component_id().clone())
                        .or_insert_with(Vec::new)
                        .push(value);
                    values
                });
        module
            .shared
            .inner
            .lock()
            .expect("component runtime state lock")
            .component_controls = component_controls;
        Some(Box::new(module))
    }
}

impl ComponentHostService for SharedComponentState {
    fn instance_id(&self) -> InstanceId {
        self.inner
            .lock()
            .expect("component runtime state lock")
            .instance_id()
    }

    fn current_instance_id(&self) -> Result<InstanceId, ComponentError> {
        let state = self.inner.lock().expect("component runtime state lock");
        match state.current_status().lifecycle() {
            ComponentHostLifecycle::Ready => Ok(state.instance_id()),
            ComponentHostLifecycle::Starting
            | ComponentHostLifecycle::Stopping
            | ComponentHostLifecycle::Stopped => Err(ComponentError::Unavailable),
        }
    }

    fn current_status(&self) -> ComponentHostStatus {
        self.inner
            .lock()
            .expect("component runtime state lock")
            .current_status()
    }

    fn current_lifecycle(&self) -> ComponentHostLifecycle {
        self.inner
            .lock()
            .expect("component runtime state lock")
            .current_status()
            .lifecycle()
    }

    fn current_health(&self) -> Health {
        self.inner
            .lock()
            .expect("component runtime state lock")
            .current_status()
            .health()
    }
}

impl ComponentReadinessService for SharedComponentState {
    fn policy(&self) -> ComponentReadinessPolicy {
        self.inner
            .lock()
            .expect("component runtime state lock")
            .readiness_policy
            .clone()
    }

    fn aggregate_readiness(&self) -> ComponentAggregateReadiness {
        let state = self.inner.lock().expect("component runtime state lock");
        match state.status.lifecycle() {
            ComponentHostLifecycle::Ready => project_aggregate_readiness(&state),
            ComponentHostLifecycle::Starting
            | ComponentHostLifecycle::Stopping
            | ComponentHostLifecycle::Stopped => ComponentAggregateReadiness::new(
                state.status.clone(),
                state.readiness_policy.clone(),
                Vec::new(),
            ),
        }
    }
}

impl ComponentMaterializerAdapter {
    fn rollback_materialization(
        &self,
        participation: &ComponentParticipation,
    ) -> Option<crate::ComponentParticipationCleanupError> {
        let registry =
            ComponentRegistry::new(self.shared.clone() as Arc<dyn ComponentRegistryService>);
        match registry.unregister(participation) {
            Err(ComponentError::ComponentParticipationCleanupFailed(cleanup)) => Some(cleanup),
            _ => None,
        }
    }

    fn materialization_failure(
        &self,
        component_id: &ComponentId,
        participation: &ComponentParticipation,
        phase: crate::ComponentParticipationPreparationPhase,
    ) -> ComponentError {
        let primary = ComponentError::ComponentParticipationMaterializationFailed {
            component_id: component_id.clone(),
            phase,
        };
        match self.rollback_materialization(participation) {
            Some(cleanup) => ComponentError::ComponentParticipationMaterializationCleanupFailed {
                primary: Box::new(primary),
                cleanup,
            },
            None => primary,
        }
    }

    fn preparation_completes_declaration(
        &self,
        component_id: &ComponentId,
        participation: &ComponentParticipation,
    ) -> bool {
        let state = self
            .shared
            .inner
            .lock()
            .expect("component runtime state lock");
        let Some(declaration) = state.component_declarations.get(component_id) else {
            return false;
        };
        declaration.operations().iter().all(|operation| {
            state
                .operations
                .get(operation.id())
                .is_some_and(|registered| registered.owner == *participation)
        })
    }
}

impl ComponentMaterializerService for ComponentMaterializerAdapter {
    fn known_component_ids(&self) -> Vec<ComponentId> {
        self.shared
            .inner
            .lock()
            .expect("component runtime state lock")
            .component_declarations
            .keys()
            .cloned()
            .collect()
    }

    fn materialize(&self, component_id: &ComponentId) -> Result<ComponentStatus, ComponentError> {
        let attachment = {
            let state = self
                .shared
                .inner
                .lock()
                .expect("component runtime state lock");
            match state.current_status().lifecycle() {
                ComponentHostLifecycle::Ready => {}
                ComponentHostLifecycle::Starting
                | ComponentHostLifecycle::Stopping
                | ComponentHostLifecycle::Stopped => return Err(ComponentError::Unavailable),
            }
            if !state.component_declarations.contains_key(component_id) {
                return Err(ComponentError::UnknownComponent(component_id.clone()));
            }
            let attachment = state
                .runtime_attachments
                .get(component_id)
                .cloned()
                .ok_or_else(|| {
                    ComponentError::MissingComponentParticipationRealization(component_id.clone())
                })?;
            if state.components.contains_key(component_id) {
                return Err(ComponentError::ComponentParticipationAlreadyMaterialized(
                    component_id.clone(),
                ));
            }
            attachment
        };

        let component = {
            let state = self
                .shared
                .inner
                .lock()
                .expect("component runtime state lock");
            bound_component(&state, component_id)
        };
        let registry =
            ComponentRegistry::new(self.shared.clone() as Arc<dyn ComponentRegistryService>);
        let operation_invocation = self.shared.clone() as Arc<dyn OperationRailService>;
        let operations = self.shared.clone() as Arc<dyn OperationRegistrarService>;
        let component_scope = self.shared.clone() as Arc<dyn ComponentScopeService>;
        let resource_dependencies = self
            .shared
            .inner
            .lock()
            .expect("component runtime state lock")
            .resource_dependencies
            .clone();
        let preparing = registry.register(component, Health::Unavailable)?;
        let participation = preparing.participation().clone();
        let scope = ComponentParticipationScope::new(
            participation.clone(),
            operation_invocation,
            operations,
            component_scope,
            resource_dependencies,
        );
        let preparation = match attachment.prepare_with_teardown(&scope) {
            Ok(preparation) => preparation,
            Err(_) => {
                return Err(self.materialization_failure(
                    component_id,
                    &participation,
                    crate::ComponentParticipationPreparationPhase::Prepare,
                ));
            }
        };
        let (health, teardown) = preparation.into_parts();
        if self
            .shared
            .record_prepared_contribution(
                &participation,
                ComponentParticipationContribution::Base,
                teardown,
            )
            .is_err()
        {
            return Err(self.materialization_failure(
                component_id,
                &participation,
                crate::ComponentParticipationPreparationPhase::Prepare,
            ));
        }
        let augmentations = self
            .shared
            .inner
            .lock()
            .expect("component runtime state lock")
            .augmentation_preparations
            .get(component_id)
            .cloned()
            .unwrap_or_default();
        for (index, augmentation) in augmentations.into_iter().enumerate() {
            let preparation = match augmentation.prepare_with_teardown(&scope) {
                Ok(preparation) => preparation,
                Err(_) => {
                    return Err(self.materialization_failure(
                        component_id,
                        &participation,
                        crate::ComponentParticipationPreparationPhase::Prepare,
                    ));
                }
            };
            if self
                .shared
                .record_prepared_contribution(
                    &participation,
                    ComponentParticipationContribution::Augmentation { index },
                    preparation.into_teardown(),
                )
                .is_err()
            {
                return Err(self.materialization_failure(
                    component_id,
                    &participation,
                    crate::ComponentParticipationPreparationPhase::Prepare,
                ));
            }
        }
        if !self.preparation_completes_declaration(component_id, &participation) {
            return Err(self.materialization_failure(
                component_id,
                &participation,
                crate::ComponentParticipationPreparationPhase::Prepare,
            ));
        }
        if registry.update_health(&participation, health).is_err() {
            return Err(self.materialization_failure(
                component_id,
                &participation,
                crate::ComponentParticipationPreparationPhase::UpdateHealth,
            ));
        }
        match registry.activate(&participation) {
            Ok(status) => Ok(status),
            Err(_) => Err(self.materialization_failure(
                component_id,
                &participation,
                crate::ComponentParticipationPreparationPhase::Activate,
            )),
        }
    }

    fn dematerialize(&self, component_id: &ComponentId) -> Result<ComponentStatus, ComponentError> {
        {
            let state = self
                .shared
                .inner
                .lock()
                .expect("component runtime state lock");
            match state.current_status().lifecycle() {
                ComponentHostLifecycle::Ready => {}
                ComponentHostLifecycle::Starting
                | ComponentHostLifecycle::Stopping
                | ComponentHostLifecycle::Stopped => return Err(ComponentError::Unavailable),
            }
            if !state.component_declarations.contains_key(component_id) {
                return Err(ComponentError::UnknownComponent(component_id.clone()));
            }
        }
        let registry =
            ComponentRegistry::new(self.shared.clone() as Arc<dyn ComponentRegistryService>);
        let participation = registry
            .component(component_id)
            .map_err(|error| match error {
                ComponentError::UnknownComponent(_) => {
                    ComponentError::ComponentParticipationNotMaterialized(component_id.clone())
                }
                other => other,
            })?
            .participation()
            .clone();
        registry.unregister(&participation)
    }
}

impl ComponentReconstructionService for ComponentReconstructionAdapter {
    fn reconstruct(&self) -> Result<ComponentReconstructionReport, ComponentError> {
        let (lifecycle, controls, components, declared_components, attached_components) = {
            let state = self
                .shared
                .inner
                .lock()
                .expect("component runtime state lock");
            (
                state.current_status().lifecycle(),
                state
                    .component_controls
                    .values()
                    .cloned()
                    .collect::<Vec<_>>(),
                state.components.clone(),
                state
                    .component_declarations
                    .keys()
                    .cloned()
                    .collect::<std::collections::BTreeSet<_>>(),
                state
                    .runtime_attachments
                    .keys()
                    .cloned()
                    .collect::<std::collections::BTreeSet<_>>(),
            )
        };
        if !matches!(lifecycle, ComponentHostLifecycle::Ready) {
            return Err(ComponentError::ComponentReconstructionUnavailableLifecycle(
                lifecycle,
            ));
        }

        let runtime_host = ComponentMaterializerAdapter {
            shared: Arc::clone(&self.shared),
        };
        let mut outcomes = Vec::with_capacity(controls.len());
        for control in controls {
            let component_id = control.component().component_id().clone();
            let observed_runtime = components
                .get(&component_id)
                .cloned()
                .map(ComponentReconstructionRuntimeState::from_status)
                .unwrap_or(ComponentReconstructionRuntimeState::Absent);
            let declared = declared_components.contains(&component_id);
            let attached = attached_components.contains(&component_id);
            let result = match (control.desired(), &observed_runtime, declared, attached) {
                (
                    ComponentDesiredState::Enabled,
                    ComponentReconstructionRuntimeState::Absent,
                    false,
                    _,
                ) => ComponentReconstructionResult::UndeclaredComponent,
                (
                    ComponentDesiredState::Enabled,
                    ComponentReconstructionRuntimeState::Absent,
                    true,
                    false,
                ) => ComponentReconstructionResult::MissingRuntimeAttachment,
                (
                    ComponentDesiredState::Enabled,
                    ComponentReconstructionRuntimeState::Absent,
                    true,
                    true,
                ) => match runtime_host.materialize(&component_id) {
                    Ok(status) => ComponentReconstructionResult::Materialized(status),
                    Err(error) => ComponentReconstructionResult::MaterializationFailed(error),
                },
                (
                    ComponentDesiredState::Enabled,
                    ComponentReconstructionRuntimeState::Preparing(status),
                    true,
                    _,
                ) => {
                    ComponentReconstructionResult::BlockedPreparing(status.participation().clone())
                }
                (
                    ComponentDesiredState::Enabled,
                    ComponentReconstructionRuntimeState::Active(_),
                    true,
                    _,
                )
                | (
                    ComponentDesiredState::Disabled,
                    ComponentReconstructionRuntimeState::Absent,
                    _,
                    _,
                ) => ComponentReconstructionResult::AlreadyConverged,
                (
                    ComponentDesiredState::Disabled,
                    ComponentReconstructionRuntimeState::Preparing(_),
                    true,
                    _,
                )
                | (
                    ComponentDesiredState::Disabled,
                    ComponentReconstructionRuntimeState::Active(_),
                    true,
                    _,
                ) => match runtime_host.dematerialize(&component_id) {
                    Ok(status) => ComponentReconstructionResult::Dematerialized(status),
                    Err(error) => ComponentReconstructionResult::DematerializationFailed(error),
                },
                (
                    ComponentDesiredState::Enabled,
                    ComponentReconstructionRuntimeState::Preparing(_),
                    false,
                    _,
                )
                | (
                    ComponentDesiredState::Enabled,
                    ComponentReconstructionRuntimeState::Active(_),
                    false,
                    _,
                )
                | (
                    ComponentDesiredState::Disabled,
                    ComponentReconstructionRuntimeState::Preparing(_),
                    false,
                    _,
                )
                | (
                    ComponentDesiredState::Disabled,
                    ComponentReconstructionRuntimeState::Active(_),
                    false,
                    _,
                ) => ComponentReconstructionResult::PresentButUnmanaged,
            };
            outcomes.push(ComponentReconstructionOutcome::new(
                control,
                observed_runtime,
                result,
            ));
        }
        Ok(ComponentReconstructionReport::new(outcomes))
    }
}
