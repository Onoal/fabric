use std::sync::{Arc, Mutex};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionError, CompositionId,
    ContractRequirement, Health, InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId,
    ModuleRuntime,
};

use fabric_component::{
    Component, ComponentControl, ComponentControlRail, ComponentDeclaration, ComponentDesiredState,
    ComponentError, ComponentId, ComponentParticipation, ComponentRegistry, ComponentRuntime,
    ComponentRuntimeModule, ComponentRuntimeService,
};

type CapturedControlRail = Arc<Mutex<Option<Arc<ComponentControlRail>>>>;
type CapturedRegistry = Arc<Mutex<Option<Arc<ComponentRegistry>>>>;

#[derive(Clone)]
struct RuntimeParticipantModule {
    module_id: ModuleId,
    component_id: ComponentId,
    runtime_requirement: ContractRequirement<ComponentRuntime>,
    registry_requirement: ContractRequirement<ComponentRegistry>,
    component: Option<Component>,
    participation: Option<ComponentParticipation>,
    registry: Option<Arc<ComponentRegistry>>,
}

impl RuntimeParticipantModule {
    fn new(module_id: &str, component_id: &str) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            component_id: ComponentId::new(component_id).expect("component id"),
            runtime_requirement: ContractRequirement::provisional(
                fabric_component::component_runtime_contract_id(),
            ),
            registry_requirement: ContractRequirement::provisional(
                fabric_component::component_registry_contract_id(),
            ),
            component: None,
            participation: None,
            registry: None,
        }
    }
}

impl ModuleRuntime for RuntimeParticipantModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        Vec::new()
            .into_iter()
            .map(fabric_core::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![
            self.runtime_requirement.id().clone(),
            self.registry_requirement.id().clone(),
        ]
        .into_iter()
        .map(fabric_core::ContractRequirementDeclaration::provisional)
        .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let runtime_contract = bindings
            .resolve(&self.runtime_requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        let registry = bindings
            .resolve(&self.registry_requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        self.component = Some(Component::bind(
            self.component_id.clone(),
            runtime_contract.as_ref(),
        ));
        self.registry = Some(registry);
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.registry
            .as_ref()
            .expect("registry bound")
            .register(
                self.component.clone().expect("component bound"),
                Health::Healthy,
            )
            .map(|status| self.participation = Some(status.participation().clone()))
            .map_err(|error| ModuleError::new(error.to_string()))
    }

    fn stop(&mut self) {
        if let (Some(participation), Some(registry)) = (&self.participation, &self.registry) {
            let _ = registry.update_health(participation, Health::Unavailable);
            let _ = registry.unregister(participation);
        }
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

#[derive(Clone)]
struct ControlCaptureModule {
    module_id: ModuleId,
    control_requirement: ContractRequirement<ComponentControlRail>,
    registry_requirement: ContractRequirement<ComponentRegistry>,
    control_capture: CapturedControlRail,
    registry_capture: CapturedRegistry,
}

impl ControlCaptureModule {
    fn new(control_capture: CapturedControlRail, registry_capture: CapturedRegistry) -> Self {
        Self {
            module_id: ModuleId::new("runtime.control.capture").expect("module id"),
            control_requirement: ContractRequirement::provisional(
                fabric_component::component_control_contract_id(),
            ),
            registry_requirement: ContractRequirement::provisional(
                fabric_component::component_registry_contract_id(),
            ),
            control_capture,
            registry_capture,
        }
    }
}

impl ModuleRuntime for ControlCaptureModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        Vec::new()
            .into_iter()
            .map(fabric_core::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![
            self.control_requirement.id().clone(),
            self.registry_requirement.id().clone(),
        ]
        .into_iter()
        .map(fabric_core::ContractRequirementDeclaration::provisional)
        .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let control = bindings
            .resolve(&self.control_requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        let registry = bindings
            .resolve(&self.registry_requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.control_capture.lock().expect("control capture") = Some(control);
        *self.registry_capture.lock().expect("registry capture") = Some(registry);
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) {}

    fn health(&self) -> Health {
        Health::Healthy
    }
}

#[derive(Clone)]
struct StaticComponentRuntimeService {
    instance_id: InstanceId,
}

impl ComponentRuntimeService for StaticComponentRuntimeService {
    fn instance_id(&self) -> InstanceId {
        self.instance_id.clone()
    }

    fn current_instance_id(&self) -> Result<InstanceId, ComponentError> {
        Ok(self.instance_id())
    }

    fn current_status(&self) -> fabric_component::ComponentRuntimeStatus {
        fabric_component::ComponentRuntimeStatus::new(
            self.instance_id(),
            None,
            fabric_component::ComponentRuntimeLifecycle::Stopped,
            Health::Unavailable,
        )
    }

    fn current_lifecycle(&self) -> fabric_component::ComponentRuntimeLifecycle {
        fabric_component::ComponentRuntimeLifecycle::Ready
    }

    fn current_health(&self) -> Health {
        Health::Healthy
    }
}

fn runtime_fixture(
    component_module_id: &str,
    component_id: &str,
) -> (
    fabric_core::Instance,
    Arc<ComponentControlRail>,
    Arc<ComponentRegistry>,
) {
    let control_capture = Arc::new(Mutex::new(None));
    let registry_capture = Arc::new(Mutex::new(None));

    let composition = CompositionBuilder::new(
        CompositionId::new("runtime.control.runtime").expect("composition"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("runtime.control.runtime.block").expect("block"))
            .register_module(
                ComponentRuntimeModule::with_components(
                    vec![ComponentDeclaration::new(
                        ComponentId::new(component_id).expect("component id"),
                        Vec::new(),
                    )],
                    Vec::new(),
                )
                .expect("component host"),
            )
            .register_module(RuntimeParticipantModule::new(
                component_module_id,
                component_id,
            ))
            .register_module(ControlCaptureModule::new(
                Arc::clone(&control_capture),
                Arc::clone(&registry_capture),
            ))
            .build(),
    )
    .build()
    .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("runtime.control").expect("instance id"))
        .expect("materialize composition");
    instance.start().expect("start composition");

    let control = control_capture
        .lock()
        .expect("control capture")
        .clone()
        .expect("control captured");
    let registry = registry_capture
        .lock()
        .expect("registry capture")
        .clone()
        .expect("registry captured");
    (instance, control, registry)
}

#[test]
fn desired_state_can_exist_without_runtime_participation() {
    let control_capture = Arc::new(Mutex::new(None));
    let registry_capture = Arc::new(Mutex::new(None));
    let composition = CompositionBuilder::new(
        CompositionId::new("runtime.control.desired-only").expect("composition"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("runtime.control.desired-only.block").expect("block"))
            .register_module(ComponentRuntimeModule::new())
            .register_module(ControlCaptureModule::new(
                Arc::clone(&control_capture),
                Arc::clone(&registry_capture),
            ))
            .build(),
    )
    .build()
    .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("runtime.control").expect("instance id"))
        .expect("materialize composition");
    instance.start().expect("start composition");

    let control = control_capture
        .lock()
        .expect("control capture")
        .clone()
        .expect("control captured");
    let registry = registry_capture
        .lock()
        .expect("registry capture")
        .clone()
        .expect("registry captured");
    let runtime_contract = ComponentRuntime::new(Arc::new(StaticComponentRuntimeService {
        instance_id: InstanceId::new("runtime.control").expect("instance id"),
    }));
    let component = Component::bind(
        ComponentId::new("component.desired").expect("component id"),
        &runtime_contract,
    );

    let record = control
        .enable(component.clone())
        .expect("desired state created");

    assert_eq!(record.component(), &component);
    assert_eq!(record.desired(), ComponentDesiredState::Enabled);
    assert!(registry.components().is_empty());

    instance.stop();
}

#[test]
fn runtime_participation_can_differ_from_desired_state() {
    let (mut instance, control, registry) =
        runtime_fixture("runtime.control.runtime.component", "component.runtime");

    assert_eq!(registry.components().len(), 1);
    let runtime_status = registry
        .component(&ComponentId::new("component.runtime").expect("component id"))
        .expect("runtime status");
    let disabled = control
        .disable(runtime_status.component().clone())
        .expect("disable desired state");

    assert_eq!(disabled.desired(), ComponentDesiredState::Disabled);
    assert_eq!(registry.components().len(), 1);
    assert_eq!(
        registry
            .component(&ComponentId::new("component.runtime").expect("component id"))
            .expect("runtime still present"),
        runtime_status
    );

    instance.stop();
}

#[test]
fn desired_state_mutation_is_idempotent_and_preserves_identity() {
    let (mut instance, control, _) = runtime_fixture(
        "runtime.control.idempotent.component",
        "component.idempotent",
    );
    let runtime_contract = ComponentRuntime::new(Arc::new(StaticComponentRuntimeService {
        instance_id: InstanceId::new("runtime.control").expect("instance id"),
    }));
    let component = Component::bind(
        ComponentId::new("component.idempotent").expect("component id"),
        &runtime_contract,
    );

    let first_enabled = control.enable(component.clone()).expect("first enable");
    let second_enabled = control.enable(component.clone()).expect("second enable");
    let disabled = control.disable(component.clone()).expect("disable");
    let second_disabled = control.disable(component.clone()).expect("second disable");
    let enabled_again = control.enable(component.clone()).expect("enable again");

    assert_eq!(first_enabled, second_enabled);
    assert_eq!(disabled, second_disabled);
    assert_eq!(
        enabled_again.component().component_id(),
        component.component_id()
    );
    assert_eq!(
        enabled_again.component().instance_id(),
        component.instance_id()
    );

    instance.stop();
}

#[test]
fn foreign_instance_component_cannot_be_controlled() {
    let (mut instance, control, _) =
        runtime_fixture("runtime.control.local.component", "component.local");
    let foreign_contract = ComponentRuntime::new(Arc::new(StaticComponentRuntimeService {
        instance_id: InstanceId::new("runtime.foreign").expect("instance id"),
    }));
    let foreign_component = Component::bind(
        ComponentId::new("component.foreign").expect("component id"),
        &foreign_contract,
    );

    let error = control
        .enable(foreign_component)
        .expect_err("foreign component rejected");

    assert!(matches!(
        error,
        ComponentError::ComponentControlInstanceMismatch { .. }
    ));

    instance.stop();
}

#[test]
fn control_lookup_and_listing_are_deterministic() {
    let (mut instance, control, _) =
        runtime_fixture("runtime.control.listing.component", "component.listing");
    let runtime_contract = ComponentRuntime::new(Arc::new(StaticComponentRuntimeService {
        instance_id: InstanceId::new("runtime.control").expect("instance id"),
    }));
    let component_a = Component::bind(
        ComponentId::new("component.alpha").expect("component id"),
        &runtime_contract,
    );
    let component_b = Component::bind(
        ComponentId::new("component.beta").expect("component id"),
        &runtime_contract,
    );

    control.enable(component_b.clone()).expect("enable beta");
    control.disable(component_a.clone()).expect("disable alpha");

    let alpha = control
        .control(&ComponentId::new("component.alpha").expect("component id"))
        .expect("alpha control");
    let listed = control.controls();
    assert_eq!(alpha.component(), &component_a);
    assert_eq!(alpha.desired(), ComponentDesiredState::Disabled);
    assert_eq!(
        listed
            .iter()
            .map(|record| record.component().component_id().as_str())
            .collect::<Vec<_>>(),
        vec!["component.alpha", "component.beta"]
    );

    instance.stop();
}

#[test]
fn control_lookup_reports_missing_control_without_runtime_requirement() {
    let (mut instance, control, _) =
        runtime_fixture("runtime.control.lookup.component", "component.lookup");

    let error = control
        .control(&ComponentId::new("component.unmanaged").expect("component id"))
        .expect_err("missing control");
    assert_eq!(
        error,
        ComponentError::UnknownComponentControl(
            ComponentId::new("component.unmanaged").expect("component id")
        )
    );

    instance.stop();
}

#[test]
fn control_does_not_require_runtime_membership_path() {
    let error =
        CompositionBuilder::new(CompositionId::new("runtime.control.orphan").expect("composition"))
            .register_block(
                BlockBuilder::new(BlockId::new("runtime.control.orphan.block").expect("block"))
                    .register_module(ControlCaptureModule::new(
                        Arc::new(Mutex::new(None)),
                        Arc::new(Mutex::new(None)),
                    ))
                    .build(),
            )
            .build()
            .expect_err("control rail requires component runtime");

    assert!(matches!(error, CompositionError::MissingProvider { .. }));
}

#[test]
fn control_records_round_trip_desired_state() {
    let contract = ComponentRuntime::new(Arc::new(StaticComponentRuntimeService {
        instance_id: InstanceId::new("runtime.control").expect("instance id"),
    }));
    let component = Component::bind(
        ComponentId::new("component.record").expect("component id"),
        &contract,
    );
    let control = ComponentControl::new(component.clone(), ComponentDesiredState::Enabled);

    assert_eq!(control.component(), &component);
    assert_eq!(control.desired(), ComponentDesiredState::Enabled);
    assert_eq!(
        control
            .with_desired(ComponentDesiredState::Disabled)
            .desired(),
        ComponentDesiredState::Disabled
    );
}
