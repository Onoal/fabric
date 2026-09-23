use std::sync::{Arc, Mutex};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionError, CompositionId,
    ContractRequirement, Health, Instance, InstanceId, ModuleBindings, ModuleContract, ModuleError,
    ModuleId, ModuleRuntime,
};

use fabric_component::{
    ComponentDeclaration, ComponentError, ComponentHost, ComponentHostModule, ComponentId,
    ComponentInstanceBinding, ComponentParticipation, ComponentRegistry, ComponentStatus,
    ParticipationState,
};

type CapturedRegistry = Arc<Mutex<Option<Arc<ComponentRegistry>>>>;

#[derive(Clone)]
struct RuntimeParticipantModule {
    module_id: ModuleId,
    component_id: ComponentId,
    runtime_requirement: ContractRequirement<ComponentHost>,
    registry_requirement: ContractRequirement<ComponentRegistry>,
    component: Option<ComponentInstanceBinding>,
    participation: Option<ComponentParticipation>,
    registry: Option<Arc<ComponentRegistry>>,
    start_health: Health,
    stop_health: Health,
}

impl RuntimeParticipantModule {
    fn new(module_id: &str, component_id: &str) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            component_id: ComponentId::new(component_id).expect("component id"),
            runtime_requirement: ContractRequirement::provisional(
                fabric_component::component_host_contract_id(),
            ),
            registry_requirement: ContractRequirement::provisional(
                fabric_component::component_registry_contract_id(),
            ),
            component: None,
            participation: None,
            registry: None,
            start_health: Health::Healthy,
            stop_health: Health::Unavailable,
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
        self.component = Some(ComponentInstanceBinding::bind(
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
                self.start_health,
            )
            .map(|status| self.participation = Some(status.participation().clone()))
            .map_err(|error| ModuleError::new(error.to_string()))
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        if let (Some(participation), Some(registry)) = (&self.participation, &self.registry) {
            let _ = registry.update_health(participation, self.stop_health);
            let _ = registry.unregister(participation);
        }
        Ok(())
    }

    fn health(&self) -> Health {
        self.start_health
    }
}

#[derive(Clone)]
struct RegistryCaptureModule {
    module_id: ModuleId,
    registry_requirement: ContractRequirement<ComponentRegistry>,
    captured_registry: CapturedRegistry,
}

impl RegistryCaptureModule {
    fn new(captured_registry: CapturedRegistry) -> Self {
        Self {
            module_id: ModuleId::new("runtime.registry.capture").expect("module id"),
            registry_requirement: ContractRequirement::provisional(
                fabric_component::component_registry_contract_id(),
            ),
            captured_registry,
        }
    }
}

impl ModuleRuntime for RegistryCaptureModule {
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
        vec![self.registry_requirement.id().clone()]
            .into_iter()
            .map(fabric_core::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let registry = bindings
            .resolve(&self.registry_requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.captured_registry.lock().expect("registry capture") = Some(registry);
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

#[derive(Clone)]
struct StaticComponentRuntimeService {
    instance_id: InstanceId,
}

impl fabric_component::ComponentHostService for StaticComponentRuntimeService {
    fn instance_id(&self) -> InstanceId {
        self.instance_id.clone()
    }

    fn current_instance_id(&self) -> Result<InstanceId, ComponentError> {
        Ok(self.instance_id())
    }

    fn current_status(&self) -> fabric_component::ComponentHostStatus {
        fabric_component::ComponentHostStatus::new(
            self.instance_id(),
            None,
            fabric_component::ComponentHostLifecycle::Stopped,
            Health::Unavailable,
        )
    }

    fn current_lifecycle(&self) -> fabric_component::ComponentHostLifecycle {
        fabric_component::ComponentHostLifecycle::Stopped
    }

    fn current_health(&self) -> Health {
        Health::Unavailable
    }
}

fn build_running_registry_fixture(
    component_module_id: &str,
    component_id: &str,
) -> (Instance, Arc<ComponentRegistry>) {
    let captured_registry = Arc::new(Mutex::new(None));
    let composition = CompositionBuilder::new(
        CompositionId::new("runtime.registry.runtime").expect("composition"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("runtime.registry.block").expect("block"))
            .register_module(
                ComponentHostModule::with_components(
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
            .register_module(RegistryCaptureModule::new(Arc::clone(&captured_registry)))
            .build(),
    )
    .build()
    .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("runtime.runtime").expect("instance id"))
        .expect("materialize composition");
    instance.start().expect("start composition");

    let registry = captured_registry
        .lock()
        .expect("registry capture")
        .clone()
        .expect("registry captured");
    (instance, registry)
}

#[test]
fn runtime_component_appears_in_registry_after_real_startup() {
    let (mut instance, registry) =
        build_running_registry_fixture("runtime.registry.component", "component.runtime");

    let status = registry
        .component(&ComponentId::new("component.runtime").expect("component id"))
        .expect("component present");
    assert_eq!(
        status.component().component_id().as_str(),
        "component.runtime"
    );
    assert_eq!(status.component().instance_id().as_str(), "runtime.runtime");
    assert_eq!(status.health(), Health::Healthy);

    instance.stop().expect("stop instance");
}

#[test]
fn runtime_component_requires_runtime_membership() {
    let error = CompositionBuilder::new(
        CompositionId::new("runtime.registry.no-runtime").expect("composition"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("runtime.registry.no-runtime.block").expect("block"))
            .register_module(RuntimeParticipantModule::new(
                "runtime.registry.orphan",
                "component.orphan",
            ))
            .build(),
    )
    .build()
    .expect_err("component requires component runtime registry and contract");

    assert!(matches!(error, CompositionError::MissingProvider { .. }));
}

#[test]
fn runtime_component_registration_rejects_foreign_instance_membership() {
    let (mut instance, registry) =
        build_running_registry_fixture("runtime.registry.local", "component.local");
    let foreign_contract = ComponentHost::new(Arc::new(StaticComponentRuntimeService {
        instance_id: InstanceId::new("runtime.foreign").expect("instance id"),
    }));
    let foreign_component = ComponentInstanceBinding::bind(
        ComponentId::new("component.foreign").expect("component id"),
        &foreign_contract,
    );

    let error = registry
        .register(foreign_component, Health::Healthy)
        .expect_err("foreign component rejected");

    assert!(matches!(
        error,
        ComponentError::ComponentRegistryInstanceMismatch { .. }
    ));

    instance.stop().expect("stop instance");
}

#[test]
fn runtime_component_identity_is_stable_and_listing_is_deterministic() {
    let (mut instance, registry) =
        build_running_registry_fixture("runtime.registry.alpha", "component.alpha");

    let listed = registry.components();
    assert_eq!(listed.len(), 1);
    assert_eq!(
        listed[0].component().component_id().as_str(),
        "component.alpha"
    );
    assert_eq!(listed[0].health(), Health::Healthy);

    instance.stop().expect("stop instance");
}

#[test]
fn duplicate_component_registration_fails_deterministically() {
    let composition = CompositionBuilder::new(
        CompositionId::new("runtime.registry.duplicate").expect("composition"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("runtime.registry.duplicate.block").expect("block"))
            .register_module(
                ComponentHostModule::with_components(
                    vec![ComponentDeclaration::new(
                        ComponentId::new("component.duplicate").expect("component id"),
                        Vec::new(),
                    )],
                    Vec::new(),
                )
                .expect("component host"),
            )
            .register_module(RuntimeParticipantModule::new(
                "runtime.registry.first",
                "component.duplicate",
            ))
            .register_module(RuntimeParticipantModule::new(
                "runtime.registry.second",
                "component.duplicate",
            ))
            .build(),
    )
    .build()
    .expect("composition");

    let mut instance = composition
        .materialize(InstanceId::new("runtime.duplicate").expect("instance id"))
        .expect("materialize composition");
    let error = instance.start().expect_err("duplicate component rejected");
    assert!(
        error
            .to_string()
            .contains("ComponentId `component.duplicate` is already registered")
    );
}

#[test]
fn unknown_component_health_update_fails_without_auto_registration() {
    let (mut instance, registry) =
        build_running_registry_fixture("runtime.registry.health", "component.health");
    let foreign_contract = ComponentHost::new(Arc::new(StaticComponentRuntimeService {
        instance_id: InstanceId::new("runtime.runtime").expect("instance id"),
    }));
    let unknown_component = ComponentInstanceBinding::bind(
        ComponentId::new("component.unknown").expect("component id"),
        &foreign_contract,
    );

    let error = registry
        .update_health(
            &registry
                .component(&ComponentId::new("component.health").expect("component id"))
                .expect("component status")
                .participation()
                .clone()
                .with_component(unknown_component),
            Health::Degraded,
        )
        .expect_err("unknown component rejected");
    assert_eq!(
        error,
        ComponentError::UnknownComponent(
            ComponentId::new("component.unknown").expect("component id")
        )
    );

    instance.stop().expect("stop instance");
}

#[test]
fn shutdown_cleanup_removes_active_runtime_participants() {
    let (mut instance, registry) =
        build_running_registry_fixture("runtime.registry.cleanup", "component.cleanup");

    assert_eq!(registry.components().len(), 1);
    instance.stop().expect("stop instance");
    assert!(registry.components().is_empty());
    let error = registry
        .component(&ComponentId::new("component.cleanup").expect("component id"))
        .expect_err("component removed");
    assert_eq!(
        error,
        ComponentError::UnknownComponent(
            ComponentId::new("component.cleanup").expect("component id")
        )
    );
}

#[test]
fn runtime_component_status_round_trips_health() {
    let (mut instance, registry) =
        build_running_registry_fixture("runtime.registry.status", "component.status");
    let component = registry
        .component(&ComponentId::new("component.status").expect("component id"))
        .expect("component present")
        .component()
        .clone();
    let participation = registry
        .component(&ComponentId::new("component.status").expect("component id"))
        .expect("component present")
        .participation()
        .clone();
    let status = ComponentStatus::new(
        participation,
        ParticipationState::Preparing,
        Health::Healthy,
    );

    assert_eq!(status.component(), &component);
    assert_eq!(status.state(), ParticipationState::Preparing);
    assert_eq!(status.health(), Health::Healthy);
    assert_eq!(
        status.with_health(Health::Degraded).health(),
        Health::Degraded
    );
    assert!(status.with_state(ParticipationState::Active).is_active());

    instance.stop().expect("stop instance");
}
