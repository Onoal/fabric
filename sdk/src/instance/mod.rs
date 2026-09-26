mod facility;

use fabric_component::{
    ComponentDesiredState, ComponentError, ComponentHostHandle, ComponentId,
    ComponentInstanceBinding, ComponentReconstructionReport, ComponentReconstructionResult,
    ComponentReconstructionRuntimeState, ComponentStatus, OperationFuture, OperationKey,
    ParticipationState,
};
use fabric_core::{
    CompositionId, Health, Instance as CoreInstance, InstanceGeneration, InstanceId,
    LifecycleState, ModuleId,
};
use fabric_resource::{ResourceId, ResourceName};
use fabric_system::SystemId;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::authoring::{AdapterDefinitionId, ComponentDefinition};
use crate::composition::{
    AdapterRealizationMode, FabricManifest, RealizationProvenance, SemanticRealizationKind,
};
use crate::materialization::{MaterializationPlanProvenance, MaterializationProfile};

pub use facility::{
    InstanceFacility, InstanceFacilityContext, InstanceFacilityError, InstanceFacilityName,
    InstanceFacilityObservation,
};

/// One high-level live materialization of a semantic Fabric [`crate::Composition`].
///
/// `Instance` owns generation-scoped live Core runtime state while retaining
/// the immutable semantic Composition context it was materialized from.
pub struct Instance {
    core: CoreInstance,
    semantic_context: Arc<FabricManifest>,
    materialization_plan: MaterializationPlanProvenance,
    components: Option<InstanceComponents>,
    facilities: Vec<Box<dyn InstanceFacility>>,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("composition_id", self.composition_id())
            .field("instance_id", self.instance_id())
            .field("materialization_plan", self.materialization_plan())
            .field("generation", &self.generation())
            .field("lifecycle", &self.lifecycle())
            .finish()
    }
}

#[derive(Clone)]
pub struct InstanceComponents {
    pub(crate) handle: std::sync::Arc<ComponentHostHandle>,
}

/// A current semantic observation of one live Fabric Instance.
///
/// This is the high-level Fabric projection over immutable Composition
/// semantics plus current live Core/Component observations. It is not a
/// history, reconciliation log, or raw Core [`fabric_core::InstanceReport`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceObservation {
    composition_id: CompositionId,
    instance_id: InstanceId,
    materialization_plan: MaterializationPlanProvenance,
    generation: InstanceGeneration,
    lifecycle: LifecycleState,
    health: Health,
    resources: Vec<ResourceLiveObservation>,
    systems: Vec<SystemLiveObservation>,
    components: Vec<ComponentLiveObservation>,
    facilities: Vec<InstanceFacilityObservation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceLiveObservation {
    resource_id: ResourceId,
    name: ResourceName,
    realization: SemanticRealizationObservation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemLiveObservation {
    system_id: SystemId,
    realization: SemanticRealizationObservation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentLiveObservation {
    component_id: ComponentId,
    initial_participation: ComponentDesiredState,
    desired_participation: ComponentDesiredState,
    observed_participation: ComponentObservedParticipation,
    health: Option<Health>,
    realization: Option<SemanticRealizationObservation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentObservedParticipation {
    Absent,
    Preparing,
    Active,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticRealizationObservation {
    kind: SemanticRealizationKind,
    adapter_definition_id: Option<AdapterDefinitionId>,
    local: Vec<LocalRealizationObservation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalRealizationObservation {
    role: LocalRealizationRole,
    module_id: ModuleId,
    lifecycle: LifecycleState,
    health: Health,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalRealizationRole {
    SemanticOwner,
    AdapterProvider,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentReconciliationObservation {
    outcomes: Vec<ComponentReconciliationOutcome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentReconciliationOutcome {
    component_id: ComponentId,
    desired: ComponentDesiredState,
    observed_before: ComponentObservedParticipation,
    result: ComponentReconciliationResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentReconciliationResult {
    AlreadyConverged,
    Materialized(ComponentObservedParticipation),
    Dematerialized(ComponentObservedParticipation),
    BlockedPreparing,
    MissingRuntimeAttachment,
    UndeclaredComponent,
    MaterializationFailed(ComponentError),
    DematerializationFailed(ComponentError),
    PresentButUnmanaged,
}

/// Instance-bound typed Component handle.
///
/// Declaration lookup can succeed before the Component is participating. Calls
/// and reconciliation remain bound to this Instance generation through the
/// Component host rails.
pub struct BoundComponent<'a, C>
where
    C: ComponentDefinition,
{
    instance: &'a Instance,
    _component: PhantomData<C>,
}

impl Instance {
    pub(crate) fn from_materialization(
        core: CoreInstance,
        semantic_context: Arc<FabricManifest>,
        materialization_plan: MaterializationPlanProvenance,
        components: Option<InstanceComponents>,
    ) -> Self {
        Self {
            core,
            semantic_context,
            materialization_plan,
            components,
            facilities: Vec::new(),
        }
    }

    /// Returns the immutable raw Core runtime occurrence for deliberate
    /// advanced diagnostics.
    pub fn core(&self) -> &CoreInstance {
        &self.core
    }

    pub fn composition_id(&self) -> &CompositionId {
        self.core.composition_id()
    }

    pub fn instance_id(&self) -> &InstanceId {
        self.core.instance_id()
    }

    pub fn materialization_profile(&self) -> &MaterializationProfile {
        self.materialization_plan.materialization_profile()
    }

    pub fn materialization_plan(&self) -> &MaterializationPlanProvenance {
        &self.materialization_plan
    }

    pub fn generation(&self) -> InstanceGeneration {
        self.core.generation()
    }

    pub fn lifecycle(&self) -> LifecycleState {
        self.core.lifecycle()
    }

    pub fn observe(&self) -> InstanceObservation {
        let report = self.core.report();
        let module_observations = module_observations(&report);
        let resources = self
            .semantic_context
            .resources()
            .iter()
            .map(|entry| ResourceLiveObservation {
                resource_id: entry.resource_id().clone(),
                name: entry.name().clone(),
                realization: observe_realization(entry.realization(), &module_observations),
            })
            .collect();
        let systems = self
            .semantic_context
            .systems()
            .iter()
            .map(|entry| SystemLiveObservation {
                system_id: entry.system_id().clone(),
                realization: observe_realization(entry.realization(), &module_observations),
            })
            .collect();
        let components = self
            .semantic_context
            .components()
            .iter()
            .map(|declaration| {
                let component_id = declaration.component_id();
                let initial = self
                    .semantic_context
                    .component_initial_participation(component_id)
                    .expect("Component manifest must retain initial participation intent");
                let desired = self
                    .components
                    .as_ref()
                    .and_then(|components| components.handle.control(component_id).ok())
                    .map(|control| control.desired())
                    .unwrap_or(initial);
                let status = self
                    .components
                    .as_ref()
                    .and_then(|components| components.handle.status(component_id).ok());
                let observed_participation = status
                    .as_ref()
                    .map(component_observed_participation)
                    .unwrap_or(ComponentObservedParticipation::Absent);
                let health = status.as_ref().map(ComponentStatus::health);
                let realization = self
                    .semantic_context
                    .component_realization(component_id)
                    .map(|provenance| observe_realization(provenance, &module_observations));
                ComponentLiveObservation {
                    component_id: component_id.clone(),
                    initial_participation: initial,
                    desired_participation: desired,
                    observed_participation,
                    health,
                    realization,
                }
            })
            .collect();

        InstanceObservation {
            composition_id: report.composition_id,
            instance_id: report.instance_id,
            materialization_plan: self.materialization_plan.clone(),
            generation: report.generation,
            lifecycle: report.lifecycle,
            health: report.health,
            resources,
            systems,
            components,
            facilities: self
                .facilities
                .iter()
                .map(|facility| InstanceFacilityObservation::new(facility.name().clone()))
                .collect(),
        }
    }

    pub fn start(&mut self) -> Result<(), fabric_core::InstanceError> {
        let starting = self.facility_context();
        for facility in &mut self.facilities {
            facility.starting(&starting);
        }
        let result = self.core.start();
        if result.is_ok() {
            let started = self.facility_context();
            for facility in &mut self.facilities {
                facility.started(&started);
            }
        }
        result
    }

    pub fn stop(&mut self) -> Result<(), fabric_core::RuntimeCleanupError> {
        if self.lifecycle() == LifecycleState::Stopped {
            return self.core.stop();
        }
        let stopping = self.facility_context();
        for facility in &mut self.facilities {
            facility.stopping(&stopping);
        }
        let result = self.core.stop();
        if self.lifecycle() == LifecycleState::Stopped {
            let stopped = self.facility_context();
            for facility in &mut self.facilities {
                facility.stopped(&stopped);
            }
        }
        result
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn semantic_context(&self) -> &Arc<FabricManifest> {
        &self.semantic_context
    }

    /// Returns the bounded Component live/operator surface, if this
    /// Composition has a Component host. Resource/System-only Compositions
    /// intentionally have no fake Component operator.
    pub fn components(&self) -> Option<&InstanceComponents> {
        self.components.as_ref()
    }

    pub fn attach_facility(
        &mut self,
        mut facility: impl InstanceFacility + 'static,
    ) -> Result<(), InstanceFacilityError> {
        let name = facility.name().clone();
        if self.lifecycle() == LifecycleState::Stopped {
            return Err(InstanceFacilityError::InstanceStopped { name });
        }
        if self
            .facilities
            .iter()
            .any(|attached| attached.name() == &name)
        {
            return Err(InstanceFacilityError::DuplicateFacility { name });
        }
        let context = self.facility_context();
        facility.attached(&context);
        self.facilities.push(Box::new(facility));
        Ok(())
    }

    pub fn detach_facility(
        &mut self,
        name: &InstanceFacilityName,
    ) -> Result<(), InstanceFacilityError> {
        let position = self
            .facilities
            .iter()
            .position(|facility| facility.name() == name)
            .ok_or_else(|| InstanceFacilityError::FacilityNotFound { name: name.clone() })?;
        let mut facility = self.facilities.remove(position);
        let context = self.facility_context();
        facility.detached(&context);
        Ok(())
    }

    fn facility_context(&self) -> InstanceFacilityContext {
        InstanceFacilityContext::new(
            self.composition_id().clone(),
            self.instance_id().clone(),
            self.generation(),
            self.materialization_plan.clone(),
            self.lifecycle(),
        )
    }

    pub fn reconcile_components(
        &self,
    ) -> Result<ComponentReconciliationObservation, ComponentError> {
        let report = self.component_host()?.handle.reconstruct()?;
        Ok(ComponentReconciliationObservation::from_report(report))
    }

    pub fn component<C>(&self) -> Result<BoundComponent<'_, C>, ComponentError>
    where
        C: ComponentDefinition,
    {
        let component_id = C::component_id();
        if !self
            .semantic_context
            .components()
            .iter()
            .any(|declaration| declaration.component_id() == &component_id)
        {
            return Err(ComponentError::UnknownComponent(component_id));
        }
        Ok(BoundComponent {
            instance: self,
            _component: PhantomData,
        })
    }

    fn component_host(&self) -> Result<&InstanceComponents, ComponentError> {
        self.components.as_ref().ok_or(ComponentError::Unavailable)
    }
}

impl InstanceComponents {
    pub fn materialize<C>(&self) -> Result<ComponentStatus, ComponentError>
    where
        C: ComponentDefinition,
    {
        self.handle.materialize(&C::component_id())
    }

    pub fn dematerialize<C>(&self) -> Result<ComponentStatus, ComponentError>
    where
        C: ComponentDefinition,
    {
        self.handle.dematerialize(&C::component_id())
    }

    pub fn materialize_id(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentStatus, ComponentError> {
        self.handle.materialize(component_id)
    }

    pub fn dematerialize_id(
        &self,
        component_id: &ComponentId,
    ) -> Result<ComponentStatus, ComponentError> {
        self.handle.dematerialize(component_id)
    }

    pub fn invoke_external<I, O>(
        &self,
        operation: &OperationKey<I, O>,
        input: I,
    ) -> OperationFuture<O>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
    {
        self.handle.invoke_external(operation, input)
    }

    pub fn status<C>(&self) -> Result<ComponentStatus, ComponentError>
    where
        C: ComponentDefinition,
    {
        self.handle.status(&C::component_id())
    }

    pub fn status_id(&self, component_id: &ComponentId) -> Result<ComponentStatus, ComponentError> {
        self.handle.status(component_id)
    }

    pub fn set_desired<C>(
        &self,
        desired: ComponentDesiredState,
        instance_id: &InstanceId,
    ) -> Result<ComponentDesiredState, ComponentError>
    where
        C: ComponentDefinition,
    {
        let component =
            ComponentInstanceBinding::for_instance(C::component_id(), instance_id.clone());
        self.handle
            .set_desired(component, desired)
            .map(|control| control.desired())
    }

    pub fn desired<C>(&self) -> Result<ComponentDesiredState, ComponentError>
    where
        C: ComponentDefinition,
    {
        self.handle
            .control(&C::component_id())
            .map(|control| control.desired())
    }

    pub fn reconcile(&self) -> Result<ComponentReconciliationObservation, ComponentError> {
        let report = self.handle.reconstruct()?;
        Ok(ComponentReconciliationObservation::from_report(report))
    }
}

impl<'a, C> BoundComponent<'a, C>
where
    C: ComponentDefinition,
{
    pub fn component_id(&self) -> ComponentId {
        C::component_id()
    }

    pub fn instance_id(&self) -> &InstanceId {
        self.instance.instance_id()
    }

    pub fn generation(&self) -> InstanceGeneration {
        self.instance.generation()
    }

    pub fn observe(&self) -> ComponentLiveObservation {
        let component_id = C::component_id();
        self.instance
            .observe()
            .components
            .into_iter()
            .find(|component| component.component_id == component_id)
            .expect("BoundComponent must refer to a declared semantic component")
    }

    pub fn desired(&self) -> Result<ComponentDesiredState, ComponentError> {
        self.instance
            .component_host()?
            .handle
            .control(&C::component_id())
            .map(|control| control.desired())
    }

    pub fn set_desired(
        &self,
        desired: ComponentDesiredState,
    ) -> Result<ComponentDesiredState, ComponentError> {
        let component = ComponentInstanceBinding::for_instance(
            C::component_id(),
            self.instance.instance_id().clone(),
        );
        self.instance
            .component_host()?
            .handle
            .set_desired(component, desired)
            .map(|control| control.desired())
    }

    pub fn enable(&self) -> Result<ComponentDesiredState, ComponentError> {
        self.set_desired(ComponentDesiredState::Enabled)
    }

    pub fn disable(&self) -> Result<ComponentDesiredState, ComponentError> {
        self.set_desired(ComponentDesiredState::Disabled)
    }

    pub fn reconcile(&self) -> Result<ComponentReconciliationOutcome, ComponentError> {
        let component_id = C::component_id();
        self.instance
            .reconcile_components()?
            .outcomes
            .into_iter()
            .find(|outcome| outcome.component_id == component_id)
            .ok_or(ComponentError::UnknownComponent(component_id))
    }

    pub fn call<I, O>(&self, operation: &OperationKey<I, O>, input: I) -> OperationFuture<O>
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
    {
        let host = self.instance.component_host().cloned();
        let component_id = C::component_id();
        let operation = OperationKey::new(
            operation.id().clone(),
            operation.input_type().clone(),
            operation.output_type().clone(),
        );
        Box::pin(async move {
            let host = host?;
            let status = host.handle.status(&component_id)?;
            if !status.is_active() {
                return Err(ComponentError::ComponentParticipationNotActive(
                    status.participation().clone(),
                ));
            }
            host.handle.invoke_external(&operation, input).await
        })
    }
}

impl InstanceObservation {
    pub fn composition_id(&self) -> &CompositionId {
        &self.composition_id
    }
    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }
    pub fn materialization_profile(&self) -> &MaterializationProfile {
        self.materialization_plan.materialization_profile()
    }
    pub fn materialization_plan(&self) -> &MaterializationPlanProvenance {
        &self.materialization_plan
    }
    pub fn generation(&self) -> InstanceGeneration {
        self.generation
    }
    pub fn lifecycle(&self) -> LifecycleState {
        self.lifecycle
    }
    pub fn health(&self) -> Health {
        self.health
    }
    pub fn resources(&self) -> &[ResourceLiveObservation] {
        &self.resources
    }
    pub fn systems(&self) -> &[SystemLiveObservation] {
        &self.systems
    }
    pub fn components(&self) -> &[ComponentLiveObservation] {
        &self.components
    }
    pub fn facilities(&self) -> &[InstanceFacilityObservation] {
        &self.facilities
    }
}

impl ResourceLiveObservation {
    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    pub fn name(&self) -> &ResourceName {
        &self.name
    }
    pub fn realization(&self) -> &SemanticRealizationObservation {
        &self.realization
    }
}

impl SystemLiveObservation {
    pub fn system_id(&self) -> &SystemId {
        &self.system_id
    }
    pub fn realization(&self) -> &SemanticRealizationObservation {
        &self.realization
    }
}

impl ComponentLiveObservation {
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    pub fn initial_participation(&self) -> ComponentDesiredState {
        self.initial_participation
    }
    pub fn desired_participation(&self) -> ComponentDesiredState {
        self.desired_participation
    }
    pub fn observed_participation(&self) -> ComponentObservedParticipation {
        self.observed_participation
    }
    pub fn health(&self) -> Option<Health> {
        self.health
    }
    pub fn realization(&self) -> Option<&SemanticRealizationObservation> {
        self.realization.as_ref()
    }
}

impl SemanticRealizationObservation {
    pub fn kind(&self) -> SemanticRealizationKind {
        self.kind
    }
    pub fn adapter_definition_id(&self) -> Option<&AdapterDefinitionId> {
        self.adapter_definition_id.as_ref()
    }
    pub fn local(&self) -> &[LocalRealizationObservation] {
        &self.local
    }
}

impl LocalRealizationObservation {
    pub fn role(&self) -> LocalRealizationRole {
        self.role
    }
    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }
    pub fn lifecycle(&self) -> LifecycleState {
        self.lifecycle
    }
    pub fn health(&self) -> Health {
        self.health
    }
}

impl ComponentReconciliationObservation {
    pub fn outcomes(&self) -> &[ComponentReconciliationOutcome] {
        &self.outcomes
    }

    fn from_report(report: ComponentReconstructionReport) -> Self {
        Self {
            outcomes: report
                .outcomes()
                .iter()
                .map(|outcome| ComponentReconciliationOutcome {
                    component_id: outcome.component_id().clone(),
                    desired: outcome.desired(),
                    observed_before: observed_from_reconstruction(outcome.observed_runtime()),
                    result: match outcome.result() {
                        ComponentReconstructionResult::AlreadyConverged => {
                            ComponentReconciliationResult::AlreadyConverged
                        }
                        ComponentReconstructionResult::Materialized(status) => {
                            ComponentReconciliationResult::Materialized(
                                component_observed_participation(status),
                            )
                        }
                        ComponentReconstructionResult::Dematerialized(status) => {
                            ComponentReconciliationResult::Dematerialized(
                                component_observed_participation(status),
                            )
                        }
                        ComponentReconstructionResult::BlockedPreparing(_) => {
                            ComponentReconciliationResult::BlockedPreparing
                        }
                        ComponentReconstructionResult::MissingRuntimeAttachment => {
                            ComponentReconciliationResult::MissingRuntimeAttachment
                        }
                        ComponentReconstructionResult::UndeclaredComponent => {
                            ComponentReconciliationResult::UndeclaredComponent
                        }
                        ComponentReconstructionResult::MaterializationFailed(error) => {
                            ComponentReconciliationResult::MaterializationFailed(error.clone())
                        }
                        ComponentReconstructionResult::DematerializationFailed(error) => {
                            ComponentReconciliationResult::DematerializationFailed(error.clone())
                        }
                        ComponentReconstructionResult::PresentButUnmanaged => {
                            ComponentReconciliationResult::PresentButUnmanaged
                        }
                    },
                })
                .collect(),
        }
    }
}

impl ComponentReconciliationOutcome {
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    pub fn desired(&self) -> ComponentDesiredState {
        self.desired
    }
    pub fn observed_before(&self) -> ComponentObservedParticipation {
        self.observed_before
    }
    pub fn result(&self) -> &ComponentReconciliationResult {
        &self.result
    }
}

fn module_observations(
    report: &fabric_core::InstanceReport,
) -> BTreeMap<ModuleId, (LifecycleState, Health)> {
    let mut observations = BTreeMap::new();
    for block in &report.blocks {
        for module in &block.modules {
            observations.insert(module.module_id.clone(), (block.lifecycle, module.health));
        }
    }
    observations
}

fn observe_realization(
    provenance: &RealizationProvenance,
    module_observations: &BTreeMap<ModuleId, (LifecycleState, Health)>,
) -> SemanticRealizationObservation {
    match provenance {
        RealizationProvenance::DeclarationOnly => SemanticRealizationObservation {
            kind: SemanticRealizationKind::DeclarationOnly,
            adapter_definition_id: None,
            local: Vec::new(),
        },
        RealizationProvenance::SelfRealization { runtime_module_id } => {
            SemanticRealizationObservation {
                kind: SemanticRealizationKind::SelfRealization,
                adapter_definition_id: None,
                local: runtime_module_id
                    .iter()
                    .map(|module_id| {
                        observe_module(
                            LocalRealizationRole::SemanticOwner,
                            module_id,
                            module_observations,
                        )
                    })
                    .collect(),
            }
        }
        RealizationProvenance::Adapter(provenance) => {
            let mut local = Vec::new();
            if let Some(module_id) = &provenance.semantic_owner_module_id {
                local.push(observe_module(
                    LocalRealizationRole::SemanticOwner,
                    module_id,
                    module_observations,
                ));
            }
            local.push(observe_module(
                LocalRealizationRole::AdapterProvider,
                &provenance.provider_module_id,
                module_observations,
            ));
            SemanticRealizationObservation {
                kind: match provenance.mode {
                    AdapterRealizationMode::Direct => SemanticRealizationKind::AdapterDirect,
                    AdapterRealizationMode::Mediated => SemanticRealizationKind::AdapterMediated,
                },
                adapter_definition_id: Some(provenance.definition_id.clone()),
                local,
            }
        }
    }
}

fn observe_module(
    role: LocalRealizationRole,
    module_id: &ModuleId,
    module_observations: &BTreeMap<ModuleId, (LifecycleState, Health)>,
) -> LocalRealizationObservation {
    let (lifecycle, health) = module_observations
        .get(module_id)
        .copied()
        .unwrap_or((LifecycleState::Stopped, Health::Unavailable));
    LocalRealizationObservation {
        role,
        module_id: module_id.clone(),
        lifecycle,
        health,
    }
}

fn component_observed_participation(status: &ComponentStatus) -> ComponentObservedParticipation {
    match status.state() {
        ParticipationState::Preparing => ComponentObservedParticipation::Preparing,
        ParticipationState::Active => ComponentObservedParticipation::Active,
    }
}

fn observed_from_reconstruction(
    state: &ComponentReconstructionRuntimeState,
) -> ComponentObservedParticipation {
    match state {
        ComponentReconstructionRuntimeState::Absent => ComponentObservedParticipation::Absent,
        ComponentReconstructionRuntimeState::Preparing(_) => {
            ComponentObservedParticipation::Preparing
        }
        ComponentReconstructionRuntimeState::Active(_) => ComponentObservedParticipation::Active,
    }
}
