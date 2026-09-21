use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractRequirement, Health,
    Instance, InstanceGeneration, InstanceId, ModuleBindings, ModuleContract, ModuleError,
    ModuleId, ModuleRuntime,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, Wake, Waker};

use crate::{
    Component, ComponentControlRail, ComponentDeclaration, ComponentError, ComponentId,
    ComponentParticipation, ComponentRegistry, ComponentRuntime, ComponentRuntimeModule,
    OperationDefinition, OperationKey, OperationRail, OperationRegistrar, OperationTypeId,
    SurfaceRegistry,
};

const OPERATION_ID: &str = "component.participation.echo";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Echo(String);

struct Rails {
    instance_id: InstanceId,
    generation: InstanceGeneration,
    registry: Arc<ComponentRegistry>,
    operations: Arc<OperationRail>,
    registrar: Arc<OperationRegistrar>,
}

type CapturedRails = Arc<Mutex<Option<Rails>>>;

#[derive(Clone)]
struct RailsCapture {
    module_id: ModuleId,
    runtime: ContractRequirement<ComponentRuntime>,
    registry: ContractRequirement<ComponentRegistry>,
    control: ContractRequirement<ComponentControlRail>,
    operations: ContractRequirement<OperationRail>,
    registrar: ContractRequirement<OperationRegistrar>,
    surfaces: ContractRequirement<SurfaceRegistry>,
    capture: CapturedRails,
    runtime_contract: Option<Arc<ComponentRuntime>>,
    registry_rail: Option<Arc<ComponentRegistry>>,
    control_rail: Option<Arc<ComponentControlRail>>,
    operation_rail: Option<Arc<OperationRail>>,
    operation_registrar: Option<Arc<OperationRegistrar>>,
    surface_registry: Option<Arc<SurfaceRegistry>>,
}

impl RailsCapture {
    fn new(capture: CapturedRails) -> Self {
        Self {
            module_id: ModuleId::new("runtime.participation.capture").expect("module id"),
            runtime: ContractRequirement::provisional(crate::component_runtime_contract_id()),
            registry: ContractRequirement::provisional(crate::component_registry_contract_id()),
            control: ContractRequirement::provisional(crate::component_control_contract_id()),
            operations: ContractRequirement::provisional(crate::operation_rail_contract_id()),
            registrar: ContractRequirement::provisional(crate::operation_registrar_contract_id()),
            surfaces: ContractRequirement::provisional(crate::surface_contract_id()),
            capture,
            runtime_contract: None,
            registry_rail: None,
            control_rail: None,
            operation_rail: None,
            operation_registrar: None,
            surface_registry: None,
        }
    }
}

impl ModuleRuntime for RailsCapture {
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
            self.runtime.id().clone(),
            self.registry.id().clone(),
            self.control.id().clone(),
            self.operations.id().clone(),
            self.registrar.id().clone(),
            self.surfaces.id().clone(),
        ]
        .into_iter()
        .map(fabric_core::ContractRequirementDeclaration::provisional)
        .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.runtime_contract = Some(bindings.resolve(&self.runtime).map_err(module_error)?);
        self.registry_rail = Some(bindings.resolve(&self.registry).map_err(module_error)?);
        self.control_rail = Some(bindings.resolve(&self.control).map_err(module_error)?);
        self.operation_rail = Some(bindings.resolve(&self.operations).map_err(module_error)?);
        self.operation_registrar = Some(bindings.resolve(&self.registrar).map_err(module_error)?);
        self.surface_registry = Some(bindings.resolve(&self.surfaces).map_err(module_error)?);
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        let runtime = self
            .runtime_contract
            .as_ref()
            .expect("runtime contract bound");
        let rails = Rails {
            instance_id: runtime.instance_id(),
            generation: runtime
                .current_status()
                .generation()
                .expect("runtime generation"),
            registry: Arc::clone(self.registry_rail.as_ref().expect("registry bound")),
            operations: Arc::clone(self.operation_rail.as_ref().expect("operations bound")),
            registrar: Arc::clone(
                self.operation_registrar
                    .as_ref()
                    .expect("operation registrar bound"),
            ),
        };
        *self.capture.lock().expect("rails capture lock") = Some(rails);
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

fn module_error(error: impl std::fmt::Display) -> ModuleError {
    ModuleError::new(error.to_string())
}

fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    struct ThreadWaker {
        ready: Mutex<bool>,
        wake: Condvar,
    }

    impl Wake for ThreadWaker {
        fn wake(self: Arc<Self>) {
            let mut ready = self.ready.lock().expect("waker lock");
            *ready = true;
            self.wake.notify_one();
        }
    }

    let waker = Arc::new(ThreadWaker {
        ready: Mutex::new(true),
        wake: Condvar::new(),
    });
    let waker_ref = Waker::from(Arc::clone(&waker));
    let mut context = Context::from_waker(&waker_ref);
    let mut future = Pin::from(Box::new(future));
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
        let mut ready = waker.ready.lock().expect("waker lock");
        while !*ready {
            ready = waker.wake.wait(ready).expect("waker wait");
        }
        *ready = false;
    }
}

fn fixture() -> (Instance, Rails) {
    let capture = Arc::new(Mutex::new(None));
    // The host under test knows exactly the Components and endpoints these
    // rail-level fencing tests exercise. Nothing here is materialized through
    // a runtime attachment; the catalog only authorizes handler registration.
    let declarations = vec![
        echo_declaration("component.a", &[OPERATION_ID, "component.a.echo"]),
        echo_declaration("component.b", &["component.b.echo"]),
        echo_declaration("component.incarnation", &["component.incarnation.current"]),
        echo_declaration(
            "component.availability.operation",
            &["component.availability.operation.echo"],
        ),
        echo_declaration("component.shutdown", &[OPERATION_ID]),
    ];
    let composition = CompositionBuilder::new(
        CompositionId::new("runtime.participation").expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("runtime.participation.block").expect("block id"))
            .register_module(
                ComponentRuntimeModule::with_components(declarations, Vec::new())
                    .expect("component host"),
            )
            .register_module(RailsCapture::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("runtime.participation").expect("instance id"))
        .expect("materialize composition");
    instance.start().expect("start composition");
    let rails = capture
        .lock()
        .expect("rails capture lock")
        .take()
        .expect("captured rails");
    (instance, rails)
}

fn component(rails: &Rails, component_id: &str) -> Component {
    let contract = ComponentRuntime::new(Arc::new(StaticComponentRuntimeService(
        rails.instance_id.clone(),
    )));
    Component::bind(
        ComponentId::new(component_id).expect("component id"),
        &contract,
    )
}

fn prepare(rails: &Rails, component: Component, health: Health) -> ComponentParticipation {
    rails
        .registry
        .register(component, health)
        .expect("prepare")
        .participation()
        .clone()
}

fn echo_operation(operation_id: &str) -> OperationKey<Echo, Echo> {
    OperationKey::new(
        crate::OperationId::new(operation_id).expect("operation id"),
        OperationTypeId::new(format!("{operation_id}.input")).expect("input type id"),
        OperationTypeId::new(format!("{operation_id}.output")).expect("output type id"),
    )
}

fn echo_declaration(component_id: &str, operation_ids: &[&str]) -> ComponentDeclaration {
    ComponentDeclaration::new(
        ComponentId::new(component_id).expect("component id"),
        operation_ids
            .iter()
            .map(|operation_id| {
                OperationDefinition::new(
                    crate::OperationId::new(*operation_id).expect("operation id"),
                    OperationTypeId::new(format!("{operation_id}.input")).expect("input type id"),
                    OperationTypeId::new(format!("{operation_id}.output")).expect("output type id"),
                )
            })
            .collect(),
    )
}

fn register_echo(rails: &Rails, owner: ComponentParticipation, operation_id: &str) {
    rails
        .registrar
        .register(
            owner,
            echo_operation(operation_id),
            |input: Echo| async move { Ok(input) },
        )
        .expect("register operation");
}

fn invoke_echo(rails: &Rails, operation_id: &str, input: Echo) -> crate::OperationFuture<Echo> {
    rails.operations.invoke_with_context(
        crate::InvocationContext::new(
            rails.instance_id.clone(),
            rails.generation,
            crate::InvocationId::new(1),
            crate::InvocationOrigin::External,
        ),
        &echo_operation(operation_id),
        input,
    )
}

#[derive(Clone)]
struct StaticComponentRuntimeService(InstanceId);

impl crate::ComponentRuntimeService for StaticComponentRuntimeService {
    fn instance_id(&self) -> InstanceId {
        self.0.clone()
    }
    fn current_instance_id(&self) -> Result<InstanceId, ComponentError> {
        Ok(self.instance_id())
    }
    fn current_status(&self) -> crate::ComponentRuntimeStatus {
        crate::ComponentRuntimeStatus::new(
            self.instance_id(),
            None,
            crate::ComponentRuntimeLifecycle::Stopped,
            Health::Unavailable,
        )
    }
    fn current_lifecycle(&self) -> crate::ComponentRuntimeLifecycle {
        crate::ComponentRuntimeLifecycle::Stopped
    }
    fn current_health(&self) -> Health {
        Health::Unavailable
    }
}

#[test]
fn participating_component_can_register_and_invoke_an_operation() {
    let (mut instance, rails) = fixture();
    let component = component(&rails, "component.a");
    let participation = prepare(&rails, component.clone(), Health::Healthy);
    register_echo(&rails, participation, OPERATION_ID);
    rails
        .registry
        .activate(
            rails
                .registry
                .component(component.component_id())
                .expect("component status")
                .participation(),
        )
        .expect("activate");

    assert_eq!(
        block_on(invoke_echo(&rails, OPERATION_ID, Echo("ok".into()))),
        Ok(Echo("ok".into()))
    );
    instance.stop().expect("stop instance");
}

#[test]
fn non_participating_component_cannot_register_runtime_operation() {
    let (mut instance, rails) = fixture();
    let owner = component(&rails, "component.absent");
    let error = rails
        .registrar
        .register(
            ComponentParticipation::new(
                owner,
                rails.generation,
                crate::ComponentParticipationId::new(0),
            ),
            echo_operation(OPERATION_ID),
            |input: Echo| async move { Ok(input) },
        )
        .expect_err("must reject absent owner");

    assert_eq!(
        error,
        ComponentError::OperationOwnerNotParticipating {
            operation_id: crate::OperationId::new(OPERATION_ID).expect("operation id"),
            component_id: ComponentId::new("component.absent").expect("component id")
        }
    );
    instance.stop().expect("stop instance");
}

#[test]
fn unregister_removes_only_departed_components_operations_and_allows_rejoin() {
    let (mut instance, rails) = fixture();
    let a = component(&rails, "component.a");
    let b = component(&rails, "component.b");
    let a_participation = prepare(&rails, a.clone(), Health::Healthy);
    let b_participation = prepare(&rails, b.clone(), Health::Healthy);
    register_echo(&rails, a_participation.clone(), "component.a.echo");
    register_echo(&rails, b_participation.clone(), "component.b.echo");
    rails
        .registry
        .activate(&a_participation)
        .expect("activate a");
    rails
        .registry
        .activate(&b_participation)
        .expect("activate b");

    rails
        .registry
        .unregister(&a_participation)
        .expect("unregister a");
    assert_eq!(
        block_on(invoke_echo(&rails, "component.a.echo", Echo("a".into()))),
        Err(ComponentError::UnknownOperation(
            crate::OperationId::new("component.a.echo").expect("operation id")
        ))
    );
    assert_eq!(
        block_on(invoke_echo(&rails, "component.b.echo", Echo("b".into()))),
        Ok(Echo("b".into()))
    );

    let rejoined = prepare(&rails, a.clone(), Health::Healthy);
    register_echo(&rails, rejoined, "component.a.echo");
    rails
        .registry
        .activate(
            rails
                .registry
                .component(a.component_id())
                .expect("rejoined status")
                .participation(),
        )
        .expect("activate rejoined a");
    assert_eq!(
        block_on(invoke_echo(
            &rails,
            "component.a.echo",
            Echo("again".into())
        )),
        Ok(Echo("again".into()))
    );
    instance.stop().expect("stop instance");
}

#[test]
fn stale_participation_never_regains_runtime_authority_after_rejoin() {
    let (mut instance, rails) = fixture();
    let component = component(&rails, "component.incarnation");
    let first = prepare(&rails, component.clone(), Health::Healthy);
    rails.registry.unregister(&first).expect("first leaves");
    let second = prepare(&rails, component, Health::Healthy);

    assert_ne!(first.participation_id(), second.participation_id());
    assert_eq!(
        rails.registry.unregister(&first),
        Err(ComponentError::StaleComponentParticipation(first.clone()))
    );
    assert_eq!(
        rails.registry.update_health(&first, Health::Degraded),
        Err(ComponentError::StaleComponentParticipation(first.clone()))
    );
    assert_eq!(
        rails.registrar.register(
            first,
            echo_operation("component.incarnation.stale"),
            |input: Echo| async move { Ok(input) },
        ),
        Err(ComponentError::OperationOwnerNotParticipating {
            operation_id: crate::OperationId::new("component.incarnation.stale")
                .expect("operation id"),
            component_id: ComponentId::new("component.incarnation").expect("component id"),
        })
    );
    register_echo(&rails, second, "component.incarnation.current");
    rails
        .registry
        .activate(
            rails
                .registry
                .component(&ComponentId::new("component.incarnation").expect("component id"))
                .expect("current status")
                .participation(),
        )
        .expect("activate current");
    assert_eq!(
        block_on(invoke_echo(
            &rails,
            "component.incarnation.current",
            Echo("current".into()),
        )),
        Ok(Echo("current".into()))
    );
    instance.stop().expect("stop instance");
}

#[test]
fn health_controls_operation_admission_without_rejoining_or_reregistering() {
    let (mut instance, rails) = fixture();
    let component = component(&rails, "component.availability.operation");
    let participation = prepare(&rails, component, Health::Healthy);
    register_echo(
        &rails,
        participation.clone(),
        "component.availability.operation.echo",
    );
    rails
        .registry
        .activate(&participation)
        .expect("activate participation");

    assert!(
        block_on(invoke_echo(
            &rails,
            "component.availability.operation.echo",
            Echo("healthy".into()),
        ))
        .is_ok()
    );
    rails
        .registry
        .update_health(&participation, Health::Unavailable)
        .expect("unavailable");
    assert_eq!(
        block_on(invoke_echo(
            &rails,
            "component.availability.operation.echo",
            Echo("blocked".into()),
        )),
        Err(ComponentError::ComponentUnavailable(
            ComponentId::new("component.availability.operation").expect("component id"),
        ))
    );
    assert_eq!(
        rails
            .registry
            .component(&ComponentId::new("component.availability.operation").expect("component id"))
            .expect("still participates")
            .participation(),
        &participation
    );
    rails
        .registry
        .update_health(&participation, Health::Degraded)
        .expect("degraded");
    assert!(
        block_on(invoke_echo(
            &rails,
            "component.availability.operation.echo",
            Echo("degraded".into()),
        ))
        .is_ok()
    );
    rails
        .registry
        .update_health(&participation, Health::Healthy)
        .expect("recovered");
    assert!(
        block_on(invoke_echo(
            &rails,
            "component.availability.operation.echo",
            Echo("recovered".into()),
        ))
        .is_ok()
    );
    instance.stop().expect("stop instance");
}

#[test]
fn cross_instance_component_cannot_participate_or_register_operations() {
    let (mut instance, rails) = fixture();
    let foreign_contract = ComponentRuntime::new(Arc::new(StaticComponentRuntimeService(
        InstanceId::new("runtime.foreign").expect("instance id"),
    )));
    let foreign = Component::bind(
        ComponentId::new("component.foreign").expect("component id"),
        &foreign_contract,
    );

    assert!(
        rails
            .registry
            .register(foreign.clone(), Health::Healthy)
            .is_err()
    );
    assert!(
        rails
            .registrar
            .register(
                ComponentParticipation::new(
                    foreign,
                    rails.generation,
                    crate::ComponentParticipationId::new(0),
                ),
                echo_operation(OPERATION_ID),
                |input: Echo| async move { Ok(input) }
            )
            .is_err()
    );
    instance.stop().expect("stop instance");
}

#[test]
fn runtime_shutdown_removes_runtime_operation_handlers() {
    let (mut instance, rails) = fixture();
    let owner = component(&rails, "component.shutdown");
    let participation = rails
        .registry
        .register(owner.clone(), Health::Healthy)
        .expect("participate")
        .participation()
        .clone();
    register_echo(&rails, participation, OPERATION_ID);

    instance.stop().expect("stop instance");
    assert_eq!(
        rails.operations.owner(&echo_operation(OPERATION_ID)),
        Err(ComponentError::UnknownOperation(
            crate::OperationId::new(OPERATION_ID).expect("operation id")
        ))
    );
    assert_eq!(
        block_on(invoke_echo(&rails, OPERATION_ID, Echo("gone".into()))),
        Err(ComponentError::Unavailable)
    );
}
