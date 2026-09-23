use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, Wake, Waker};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractId, ContractRequirement,
    Health, Instance, InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId,
    ModuleRuntime,
};

use crate::{
    ComponentAggregateBlocker, ComponentControlRail, ComponentDeclaration, ComponentDesiredState,
    ComponentEffectiveHealth, ComponentHost, ComponentHostLifecycle, ComponentHostModule,
    ComponentId, ComponentInstanceBinding, ComponentParticipation, ComponentReadinessPolicy,
    ComponentReadinessRail, ComponentRegistry, ComponentRequirementKind, ComponentRequirementRail,
    DegradedComponentHealth, InvocationRail, OperationId, OperationKey, OperationRail,
    OperationRegistrar, ResolvedComponentRequirement,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct EchoInput(&'static str);

#[derive(Clone, Debug, PartialEq, Eq)]
struct EchoOutput(&'static str);

const REQUIREMENT_CONTRACT_ID: &str = "fabric.component.readiness.requirement";
const OPERATION_ID: &str = "fabric.component.readiness.echo";

struct Rails {
    runtime: Arc<ComponentHost>,
    registry: Arc<ComponentRegistry>,
    control: Arc<ComponentControlRail>,
    readiness: Arc<ComponentReadinessRail>,
    requirements: Arc<ComponentRequirementRail>,
    invocation: Arc<InvocationRail>,
    operations: Arc<OperationRail>,
    registrar: Arc<OperationRegistrar>,
}

type Capture = Arc<Mutex<Option<Rails>>>;

#[derive(Clone)]
struct CaptureModule {
    module_id: ModuleId,
    runtime: ContractRequirement<ComponentHost>,
    registry: ContractRequirement<ComponentRegistry>,
    control: ContractRequirement<ComponentControlRail>,
    readiness: ContractRequirement<ComponentReadinessRail>,
    requirements: ContractRequirement<ComponentRequirementRail>,
    invocation: ContractRequirement<InvocationRail>,
    operations: ContractRequirement<OperationRail>,
    registrar: ContractRequirement<OperationRegistrar>,
    capture: Capture,
}

impl CaptureModule {
    fn new(capture: Capture) -> Self {
        Self {
            module_id: ModuleId::new("runtime.readiness.capture").expect("module id"),
            runtime: ContractRequirement::provisional(crate::component_host_contract_id()),
            registry: ContractRequirement::provisional(crate::component_registry_contract_id()),
            control: ContractRequirement::provisional(crate::component_control_contract_id()),
            readiness: ContractRequirement::provisional(crate::component_readiness_contract_id()),
            requirements: ContractRequirement::provisional(
                crate::component_requirement_contract_id(),
            ),
            invocation: ContractRequirement::provisional(crate::invocation_contract_id()),
            operations: ContractRequirement::provisional(crate::operation_rail_contract_id()),
            registrar: ContractRequirement::provisional(crate::operation_registrar_contract_id()),
            capture,
        }
    }
}

impl ModuleRuntime for CaptureModule {
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
            self.readiness.id().clone(),
            self.requirements.id().clone(),
            self.invocation.id().clone(),
            self.operations.id().clone(),
            self.registrar.id().clone(),
        ]
        .into_iter()
        .map(fabric_core::ContractRequirementDeclaration::provisional)
        .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        *self.capture.lock().expect("capture lock") = Some(Rails {
            runtime: bindings.resolve(&self.runtime).map_err(module_error)?,
            registry: bindings.resolve(&self.registry).map_err(module_error)?,
            control: bindings.resolve(&self.control).map_err(module_error)?,
            readiness: bindings.resolve(&self.readiness).map_err(module_error)?,
            requirements: bindings.resolve(&self.requirements).map_err(module_error)?,
            invocation: bindings.resolve(&self.invocation).map_err(module_error)?,
            operations: bindings.resolve(&self.operations).map_err(module_error)?,
            registrar: bindings.resolve(&self.registrar).map_err(module_error)?,
        });
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

fn fixture(required_components: &[&str]) -> (Instance, Rails) {
    let capture = Arc::new(Mutex::new(None));
    let policy = ComponentReadinessPolicy::new(
        required_components
            .iter()
            .map(|component_id| ComponentId::new(*component_id).expect("required component id")),
    )
    .expect("policy");
    let composition =
        CompositionBuilder::new(CompositionId::new("runtime.readiness").expect("composition id"))
            .register_block(
                BlockBuilder::new(BlockId::new("runtime.readiness.block").expect("block id"))
                    .register_module(
                        ComponentHostModule::with_configuration(
                            policy,
                            vec![
                                ComponentDeclaration::new(
                                    ComponentId::new("component.diagnostics")
                                        .expect("component id"),
                                    vec![operation_key().definition().clone()],
                                ),
                                ComponentDeclaration::new(
                                    ComponentId::new("component.a").expect("component id"),
                                    Vec::new(),
                                ),
                                ComponentDeclaration::new(
                                    ComponentId::new("component.b").expect("component id"),
                                    Vec::new(),
                                ),
                                ComponentDeclaration::new(
                                    ComponentId::new("component.c").expect("component id"),
                                    Vec::new(),
                                ),
                                ComponentDeclaration::new(
                                    ComponentId::new("component.required").expect("component id"),
                                    Vec::new(),
                                ),
                                ComponentDeclaration::new(
                                    ComponentId::new("component.x").expect("component id"),
                                    Vec::new(),
                                ),
                                ComponentDeclaration::new(
                                    ComponentId::new("component.y").expect("component id"),
                                    Vec::new(),
                                ),
                            ],
                            Vec::new(),
                        )
                        .expect("component host"),
                    )
                    .register_module(CaptureModule::new(Arc::clone(&capture)))
                    .build(),
            )
            .build()
            .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("runtime.readiness").expect("instance id"))
        .expect("materialize instance");
    instance.start().expect("start instance");
    let rails = capture.lock().expect("capture lock").take().expect("rails");
    (instance, rails)
}

fn component(rails: &Rails, component_id: &str) -> ComponentInstanceBinding {
    ComponentInstanceBinding::bind(
        ComponentId::new(component_id).expect("component id"),
        rails.runtime.as_ref(),
    )
}

fn participate(
    rails: &Rails,
    component: ComponentInstanceBinding,
    health: Health,
) -> ComponentParticipation {
    let participation = rails
        .registry
        .register(component, health)
        .expect("participate")
        .participation()
        .clone();
    rails
        .registry
        .activate(&participation)
        .expect("activate participation");
    participation
}

fn prepare(
    rails: &Rails,
    component: ComponentInstanceBinding,
    health: Health,
) -> ComponentParticipation {
    rails
        .registry
        .register(component, health)
        .expect("prepare")
        .participation()
        .clone()
}

fn requirement(
    consumer: ComponentInstanceBinding,
    provider: ComponentInstanceBinding,
    kind: ComponentRequirementKind,
) -> ResolvedComponentRequirement {
    ResolvedComponentRequirement::synthetic(
        consumer,
        ContractId::new(REQUIREMENT_CONTRACT_ID).expect("contract id"),
        provider,
        kind,
    )
}

fn operation_key() -> OperationKey<EchoInput, EchoOutput> {
    OperationKey::new(
        OperationId::new(OPERATION_ID).expect("operation id"),
        crate::OperationTypeId::new("fabric.test.readiness.echo.input").expect("input type id"),
        crate::OperationTypeId::new("fabric.test.readiness.echo.output").expect("output type id"),
    )
}

fn register_echo(rails: &Rails, owner: ComponentParticipation) {
    rails
        .registrar
        .register(owner, operation_key(), |input: EchoInput| async move {
            Ok(EchoOutput(input.0))
        })
        .expect("register operation");
}

#[test]
fn empty_policy_starts_ready_healthy_and_status_surfaces_agree() {
    let (mut instance, rails) = fixture(&[]);
    let readiness = rails.readiness.aggregate_readiness();
    assert!(readiness.policy().required_components().is_empty());
    assert!(readiness.blockers().is_empty());
    assert_eq!(
        readiness.status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(readiness.status().health(), Health::Healthy);
    assert_eq!(rails.runtime.current_status(), readiness.status().clone());
    assert_eq!(
        rails.runtime.current_lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_health(), Health::Healthy);
    instance.stop().expect("stop instance");
}

#[test]
fn required_component_runtime_state_projects_aggregate_status_and_participation_churn() {
    let (mut instance, rails) = fixture(&["component.a"]);
    let a = component(&rails, "component.a");
    let absent = rails.readiness.aggregate_readiness();
    assert_eq!(absent.status().lifecycle(), ComponentHostLifecycle::Ready);
    assert_eq!(absent.status().health(), Health::Unavailable);
    assert_eq!(
        absent.blockers(),
        &[
            ComponentAggregateBlocker::RequiredComponentNotParticipating {
                component_id: ComponentId::new("component.a").expect("component id"),
            }
        ]
    );

    let participation = participate(&rails, a.clone(), Health::Healthy);
    assert_eq!(
        rails.readiness.aggregate_readiness().status().clone(),
        rails.runtime.current_status()
    );
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_status().health(), Health::Healthy);

    rails
        .registry
        .update_health(&participation, Health::Degraded)
        .expect("degraded");
    let degraded = rails.readiness.aggregate_readiness();
    assert_eq!(degraded.status().lifecycle(), ComponentHostLifecycle::Ready);
    assert_eq!(degraded.status().health(), Health::Degraded);
    assert_eq!(
        degraded.blockers(),
        &[ComponentAggregateBlocker::RequiredComponentDegraded {
            component_id: ComponentId::new("component.a").expect("component id"),
            effective_health: ComponentEffectiveHealth::Degraded(
                DegradedComponentHealth::Intrinsic {
                    component: a.clone(),
                    health: Health::Degraded,
                },
            ),
        }]
    );

    rails
        .registry
        .update_health(&participation, Health::Unavailable)
        .expect("unavailable");
    let unavailable = rails.readiness.aggregate_readiness();
    assert_eq!(
        unavailable.status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(unavailable.status().health(), Health::Unavailable);

    rails
        .registry
        .update_health(&participation, Health::Healthy)
        .expect("recovered");
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_status().health(), Health::Healthy);

    rails.registry.unregister(&participation).expect("leave");
    assert_eq!(rails.runtime.current_status().health(), Health::Unavailable);
    participate(&rails, a, Health::Healthy);
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_status().health(), Health::Healthy);
    instance.stop().expect("stop instance");
}

#[test]
fn dependency_health_propagates_transitively_and_recovers_without_restart() {
    let (mut instance, rails) = fixture(&["component.a"]);
    let a = component(&rails, "component.a");
    let b = component(&rails, "component.b");
    let c = component(&rails, "component.c");
    participate(&rails, a.clone(), Health::Healthy);
    participate(&rails, b.clone(), Health::Healthy);
    let c_participation = participate(&rails, c.clone(), Health::Healthy);
    rails
        .requirements
        .register(requirement(
            a.clone(),
            b.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("a->b");
    rails
        .requirements
        .register(requirement(
            b.clone(),
            c.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("b->c");

    rails
        .registry
        .update_health(&c_participation, Health::Degraded)
        .expect("c degraded");
    let degraded = rails.readiness.aggregate_readiness();
    assert_eq!(degraded.status().lifecycle(), ComponentHostLifecycle::Ready);
    assert_eq!(degraded.status().health(), Health::Degraded);

    rails
        .registry
        .update_health(&c_participation, Health::Unavailable)
        .expect("c unavailable");
    let unavailable = rails.readiness.aggregate_readiness();
    assert_eq!(
        unavailable.status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(unavailable.status().health(), Health::Unavailable);

    rails
        .registry
        .update_health(&c_participation, Health::Healthy)
        .expect("c recovered");
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_status().health(), Health::Healthy);
    instance.stop().expect("stop instance");
}

#[test]
fn unlisted_and_optional_components_do_not_degrade_aggregate_without_required_path() {
    let (mut instance, rails) = fixture(&["component.a"]);
    let a = component(&rails, "component.a");
    let x = component(&rails, "component.x");
    participate(&rails, a.clone(), Health::Healthy);
    let x_participation = participate(&rails, x.clone(), Health::Healthy);

    rails
        .registry
        .update_health(&x_participation, Health::Unavailable)
        .expect("x unavailable");
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_status().health(), Health::Healthy);

    rails
        .requirements
        .register(requirement(
            a.clone(),
            x.clone(),
            ComponentRequirementKind::Optional,
        ))
        .expect("a optional x");
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_status().health(), Health::Healthy);

    let y = component(&rails, "component.y");
    let y_participation = participate(&rails, y.clone(), Health::Healthy);
    rails
        .requirements
        .register(requirement(
            a,
            y.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("a required y");
    rails
        .registry
        .update_health(&y_participation, Health::Unavailable)
        .expect("y unavailable");
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_status().health(), Health::Unavailable);
    instance.stop().expect("stop instance");
}

#[test]
fn desired_state_is_separate_from_readiness_and_aggregate_failure_does_not_globally_block() {
    let (mut instance, rails) = fixture(&["component.required"]);
    let required = component(&rails, "component.required");
    let diagnostics = component(&rails, "component.diagnostics");
    let diagnostics_participation = prepare(&rails, diagnostics.clone(), Health::Healthy);
    register_echo(&rails, diagnostics_participation.clone());
    rails
        .registry
        .activate(&diagnostics_participation)
        .expect("activate diagnostics");

    rails
        .control
        .set_desired(required.clone(), ComponentDesiredState::Disabled)
        .expect("set desired");
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_status().health(), Health::Unavailable);

    let result = block_on(rails.operations.invoke_with_context(
        rails.invocation.begin_external().expect("gateway root"),
        &operation_key(),
        EchoInput("ok"),
    ));
    assert_eq!(result, Ok(EchoOutput("ok")));

    let required_participation = participate(&rails, required.clone(), Health::Healthy);
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_status().health(), Health::Healthy);
    rails
        .control
        .set_desired(required, ComponentDesiredState::Disabled)
        .expect("desired remains separate");
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    assert_eq!(rails.runtime.current_status().health(), Health::Healthy);

    rails
        .registry
        .update_health(&required_participation, Health::Unavailable)
        .expect("required unavailable");
    let second = block_on(rails.operations.invoke_with_context(
        rails.invocation.begin_external().expect("gateway root"),
        &operation_key(),
        EchoInput("still-ok"),
    ));
    assert_eq!(second, Ok(EchoOutput("still-ok")));
    instance.stop().expect("stop instance");
}

#[test]
fn stopped_runtime_remains_stopped_even_with_policy_and_components() {
    let (mut instance, rails) = fixture(&["component.a"]);
    participate(&rails, component(&rails, "component.a"), Health::Healthy);
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Ready
    );
    instance.stop().expect("stop instance");
    let readiness = rails.readiness.aggregate_readiness();
    assert_eq!(
        readiness.status().lifecycle(),
        ComponentHostLifecycle::Stopped
    );
    assert_eq!(readiness.status().health(), Health::Unavailable);
    assert_eq!(
        rails.runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Stopped
    );
    assert_eq!(rails.runtime.current_health(), Health::Unavailable);
}

#[test]
fn policy_is_supplied_immutably_from_composition_configuration() {
    let (mut instance, rails) = fixture(&["component.a", "component.b"]);
    let required_components = rails
        .readiness
        .policy()
        .required_components()
        .iter()
        .map(|component_id| component_id.as_str().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        required_components,
        vec!["component.a".to_owned(), "component.b".to_owned()]
    );
    instance.stop().expect("stop instance");
}
