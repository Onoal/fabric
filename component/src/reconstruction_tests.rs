use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::task::{Context, Poll, Wake, Waker};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractId, ContractRequirement,
    Health, Instance, InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId,
    ModuleRuntime,
};

use crate::{
    Component, ComponentControlRail, ComponentDeclaration, ComponentDesiredState, ComponentError,
    ComponentId, ComponentReadinessPolicy, ComponentReadinessRail, ComponentReconstructionOutcome,
    ComponentReconstructionRail, ComponentReconstructionReport, ComponentReconstructionResult,
    ComponentReconstructionRuntimeState, ComponentRegistry, ComponentRequirementKind,
    ComponentRequirementRail, ComponentRuntime, ComponentRuntimeDefinition,
    ComponentRuntimeFailurePhase, ComponentRuntimeLifecycle, ComponentRuntimeModule,
    ComponentRuntimeScope, InvocationRail, OperationId, OperationKey, OperationRail,
    ParticipationState, ResolvedComponentRequirement, SurfaceRegistry,
};

const OPERATION_ID: &str = "fabric.component.reconstruction.echo";
const REQUIREMENT_CONTRACT_ID: &str = "fabric.component.reconstruction.requirement";

#[derive(Clone, Debug, PartialEq, Eq)]
struct EchoInput(&'static str);

#[derive(Clone, Debug, PartialEq, Eq)]
struct EchoOutput(&'static str);

#[derive(Clone)]
struct Rails {
    runtime: Arc<ComponentRuntime>,
    registry: Arc<ComponentRegistry>,
    control: Arc<ComponentControlRail>,
    readiness: Arc<ComponentReadinessRail>,
    reconstruction: Arc<ComponentReconstructionRail>,
    requirements: Arc<ComponentRequirementRail>,
    invocation: Arc<InvocationRail>,
    operations: Arc<OperationRail>,
}

type Capture = Arc<Mutex<Option<Rails>>>;

#[derive(Clone)]
struct CaptureModule {
    module_id: ModuleId,
    runtime: ContractRequirement<ComponentRuntime>,
    registry: ContractRequirement<ComponentRegistry>,
    control: ContractRequirement<ComponentControlRail>,
    readiness: ContractRequirement<ComponentReadinessRail>,
    reconstruction: ContractRequirement<ComponentReconstructionRail>,
    requirements: ContractRequirement<ComponentRequirementRail>,
    invocation: ContractRequirement<InvocationRail>,
    operations: ContractRequirement<OperationRail>,
    surfaces: ContractRequirement<SurfaceRegistry>,
    capture: Capture,
}

impl CaptureModule {
    fn new(capture: Capture) -> Self {
        Self {
            module_id: ModuleId::new("runtime.reconstruction.capture").expect("module id"),
            runtime: ContractRequirement::provisional(crate::component_runtime_contract_id()),
            registry: ContractRequirement::provisional(crate::component_registry_contract_id()),
            control: ContractRequirement::provisional(crate::component_control_contract_id()),
            readiness: ContractRequirement::provisional(crate::component_readiness_contract_id()),
            reconstruction: ContractRequirement::provisional(
                crate::component_reconstruction_contract_id(),
            ),
            requirements: ContractRequirement::provisional(
                crate::component_requirement_contract_id(),
            ),
            invocation: ContractRequirement::provisional(crate::invocation_contract_id()),
            operations: ContractRequirement::provisional(crate::operation_rail_contract_id()),
            surfaces: ContractRequirement::provisional(crate::surface_contract_id()),
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
            self.reconstruction.id().clone(),
            self.requirements.id().clone(),
            self.invocation.id().clone(),
            self.operations.id().clone(),
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
        *self.capture.lock().expect("capture lock") = Some(Rails {
            runtime: bindings.resolve(&self.runtime).map_err(module_error)?,
            registry: bindings.resolve(&self.registry).map_err(module_error)?,
            control: bindings.resolve(&self.control).map_err(module_error)?,
            readiness: bindings.resolve(&self.readiness).map_err(module_error)?,
            reconstruction: bindings
                .resolve(&self.reconstruction)
                .map_err(module_error)?,
            requirements: bindings.resolve(&self.requirements).map_err(module_error)?,
            invocation: bindings.resolve(&self.invocation).map_err(module_error)?,
            operations: bindings.resolve(&self.operations).map_err(module_error)?,
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

fn component(runtime_contract: &ComponentRuntime, component_id: &str) -> Component {
    Component::bind(
        ComponentId::new(component_id).expect("component id"),
        runtime_contract,
    )
}

fn definition(
    component_id: &str,
    prepare: impl Fn(&ComponentRuntimeScope) -> Result<Health, ComponentError> + Send + Sync + 'static,
) -> (
    crate::ComponentDeclaration,
    Option<ComponentRuntimeDefinition>,
) {
    (
        crate::ComponentDeclaration::new(
            ComponentId::new(component_id).expect("component id"),
            Vec::new(),
        ),
        Some(ComponentRuntimeDefinition::new(
            ComponentId::new(component_id).expect("component id"),
            prepare,
        )),
    )
}

fn definition_with_operation(
    component_id: &str,
    prepare: impl Fn(&ComponentRuntimeScope) -> Result<Health, ComponentError> + Send + Sync + 'static,
) -> (
    crate::ComponentDeclaration,
    Option<ComponentRuntimeDefinition>,
) {
    (
        crate::ComponentDeclaration::new(
            ComponentId::new(component_id).expect("component id"),
            vec![operation_key().definition().clone()],
        ),
        Some(ComponentRuntimeDefinition::new(
            ComponentId::new(component_id).expect("component id"),
            prepare,
        )),
    )
}

fn requirement(
    consumer: Component,
    provider: Component,
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
        crate::OperationTypeId::new("fabric.test.reconstruction.echo.input")
            .expect("input type id"),
        crate::OperationTypeId::new("fabric.test.reconstruction.echo.output")
            .expect("output type id"),
    )
}

fn fixture(
    required_components: &[&str],
    components: impl IntoIterator<
        Item = (
            crate::ComponentDeclaration,
            Option<ComponentRuntimeDefinition>,
        ),
    >,
) -> (Instance, Rails) {
    let capture = Arc::new(Mutex::new(None));
    let (declarations, attachments): (Vec<_>, Vec<_>) = components.into_iter().unzip();
    let attachments = attachments.into_iter().flatten().collect::<Vec<_>>();
    let policy = ComponentReadinessPolicy::new(
        required_components
            .iter()
            .map(|component_id| ComponentId::new(*component_id).expect("required component id")),
    )
    .expect("policy");
    let composition =
        CompositionBuilder::new(CompositionId::new("runtime.reconstruction").expect("id"))
            .register_block(
                BlockBuilder::new(BlockId::new("runtime.reconstruction.block").expect("block"))
                    .register_module(
                        ComponentRuntimeModule::with_configuration(
                            policy,
                            declarations,
                            attachments,
                        )
                        .expect("component runtime"),
                    )
                    .register_module(CaptureModule::new(Arc::clone(&capture)))
                    .build(),
            )
            .build()
            .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("runtime.reconstruction").expect("instance id"))
        .expect("materialize instance");
    instance.start().expect("start instance");
    let rails = capture
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured rails");
    (instance, rails)
}

fn report_outcome<'a>(
    report: &'a ComponentReconstructionReport,
    component_id: &str,
) -> &'a ComponentReconstructionOutcome {
    let component_id = ComponentId::new(component_id).expect("component id");
    report
        .outcomes()
        .iter()
        .find(|outcome| outcome.component_id() == &component_id)
        .expect("outcome present")
}

#[test]
fn reconstruction_converges_enabled_and_disabled_without_auto_applying_controls() {
    let starts = Arc::new(AtomicUsize::new(0));
    let starts_for_definition = Arc::clone(&starts);
    let (mut composition, rails) = fixture(
        &[],
        [definition_with_operation("component.a", move |scope| {
            starts_for_definition.fetch_add(1, Ordering::SeqCst);
            scope.operation(operation_key(), |input: EchoInput| async move {
                Ok(EchoOutput(input.0))
            })?;
            Ok(Health::Healthy)
        })],
    );
    let component_a = component(&rails.runtime, "component.a");

    rails
        .control
        .enable(component_a.clone())
        .expect("enable control only");
    assert_eq!(
        rails.registry.component(component_a.component_id()),
        Err(ComponentError::UnknownComponent(
            component_a.component_id().clone(),
        ))
    );

    let report = rails.reconstruction.reconstruct().expect("reconstruct");
    assert!(matches!(
        report_outcome(&report, "component.a").result(),
        ComponentReconstructionResult::Materialized(status)
            if status.state() == ParticipationState::Active
    ));
    let status = rails
        .registry
        .component(component_a.component_id())
        .expect("materialized");
    let first_participation = status.participation().clone();
    assert_eq!(starts.load(Ordering::SeqCst), 1);

    let second_report = rails
        .reconstruction
        .reconstruct()
        .expect("reconstruct again");
    assert_eq!(
        report_outcome(&second_report, "component.a").result(),
        &ComponentReconstructionResult::AlreadyConverged
    );
    assert_eq!(
        rails
            .registry
            .component(component_a.component_id())
            .expect("still active")
            .participation(),
        &first_participation
    );
    assert_eq!(starts.load(Ordering::SeqCst), 1);

    rails
        .control
        .disable(component_a.clone())
        .expect("disable control only");
    assert_eq!(
        rails
            .registry
            .component(component_a.component_id())
            .expect("still active before reconstruct")
            .participation(),
        &first_participation
    );

    let disable_report = rails.reconstruction.reconstruct().expect("disable pass");
    assert!(matches!(
        report_outcome(&disable_report, "component.a").result(),
        ComponentReconstructionResult::Dematerialized(status)
            if status.participation() == &first_participation
    ));
    assert_eq!(
        rails.registry.component(component_a.component_id()),
        Err(ComponentError::UnknownComponent(
            component_a.component_id().clone(),
        ))
    );

    composition.stop().expect("stop runtime");
}

#[test]
fn reconstruction_ignores_unmanaged_control_absence_and_reports_undeclared_components() {
    let (mut composition, rails) = fixture(&[], [definition("known.a", |_| Ok(Health::Healthy))]);
    let known = component(&rails.runtime, "known.a");
    let missing = component(&rails.runtime, "missing.x");
    let disabled_unknown = component(&rails.runtime, "missing.y");

    rails
        .control
        .enable(missing.clone())
        .expect("enable missing");
    rails
        .control
        .disable(disabled_unknown.clone())
        .expect("disable unknown absent");

    let report = rails.reconstruction.reconstruct().expect("reconstruct");
    assert_eq!(report.outcomes().len(), 2);
    assert_eq!(
        report_outcome(&report, "missing.x").result(),
        &ComponentReconstructionResult::UndeclaredComponent
    );
    assert_eq!(
        report_outcome(&report, "missing.y").result(),
        &ComponentReconstructionResult::AlreadyConverged
    );
    assert_eq!(
        rails.registry.component(known.component_id()),
        Err(ComponentError::UnknownComponent(
            known.component_id().clone(),
        ))
    );

    composition.stop().expect("stop runtime");
}

#[test]
fn reconstruction_rejects_undeclared_participation_without_affecting_control_state() {
    let (mut composition, rails) = fixture(&[], [definition("known.a", |_| Ok(Health::Healthy))]);
    let known = component(&rails.runtime, "known.a");
    let unmanaged = component(&rails.runtime, "unknown.z");

    let preparing_known = rails
        .registry
        .register(known.clone(), Health::Healthy)
        .expect("register preparing");
    assert_eq!(
        rails
            .registry
            .register(unmanaged.clone(), Health::Healthy)
            .expect_err("undeclared component must not participate"),
        ComponentError::UnknownComponent(unmanaged.component_id().clone())
    );

    rails.control.enable(known.clone()).expect("enable known");
    rails
        .control
        .enable(unmanaged.clone())
        .expect("enable undeclared desired state");

    let report = rails.reconstruction.reconstruct().expect("reconstruct");
    assert_eq!(
        report_outcome(&report, "known.a").result(),
        &ComponentReconstructionResult::BlockedPreparing(preparing_known.participation().clone(),)
    );
    assert_eq!(
        report_outcome(&report, "unknown.z").result(),
        &ComponentReconstructionResult::UndeclaredComponent
    );
    assert_eq!(
        report_outcome(&report, "unknown.z").observed_runtime(),
        &ComponentReconstructionRuntimeState::Absent
    );

    rails.control.disable(known.clone()).expect("disable known");
    rails
        .control
        .disable(unmanaged.clone())
        .expect("disable unmanaged");
    let disable_report = rails.reconstruction.reconstruct().expect("disable pass");
    assert!(matches!(
        report_outcome(&disable_report, "known.a").result(),
        ComponentReconstructionResult::Dematerialized(status)
            if status.participation() == preparing_known.participation()
    ));
    assert_eq!(
        report_outcome(&disable_report, "unknown.z").result(),
        &ComponentReconstructionResult::AlreadyConverged
    );
    assert_eq!(
        rails.registry.component(known.component_id()),
        Err(ComponentError::UnknownComponent(
            known.component_id().clone(),
        ))
    );

    composition.stop().expect("stop runtime");
}

#[test]
fn reconstruction_failures_stay_local_and_retry_cleanly() {
    let attempts_a = Arc::new(AtomicUsize::new(0));
    let attempts_b = Arc::new(AtomicUsize::new(0));
    let attempts_a_for_definition = Arc::clone(&attempts_a);
    let attempts_b_for_definition = Arc::clone(&attempts_b);
    let (mut composition, rails) = fixture(
        &[],
        [
            definition("component.a", move |_| {
                let attempt = attempts_a_for_definition.fetch_add(1, Ordering::SeqCst);
                if attempt == 0 {
                    return Err(ComponentError::ComponentRuntimeMaterializationFailed {
                        component_id: ComponentId::new("component.a").expect("component id"),
                        phase: ComponentRuntimeFailurePhase::Prepare,
                    });
                }
                Ok(Health::Healthy)
            }),
            definition("component.b", move |_| {
                attempts_b_for_definition.fetch_add(1, Ordering::SeqCst);
                Ok(Health::Healthy)
            }),
        ],
    );
    let component_a = component(&rails.runtime, "component.a");
    let component_b = component(&rails.runtime, "component.b");
    rails.control.enable(component_a.clone()).expect("enable a");
    rails.control.enable(component_b.clone()).expect("enable b");

    let first_report = rails.reconstruction.reconstruct().expect("first pass");
    assert!(matches!(
        report_outcome(&first_report, "component.a").result(),
        ComponentReconstructionResult::MaterializationFailed(_)
    ));
    assert!(matches!(
        report_outcome(&first_report, "component.b").result(),
        ComponentReconstructionResult::Materialized(_)
    ));
    assert_eq!(
        rails.registry.component(component_a.component_id()),
        Err(ComponentError::UnknownComponent(
            component_a.component_id().clone(),
        ))
    );

    let retry_report = rails.reconstruction.reconstruct().expect("retry pass");
    assert!(matches!(
        report_outcome(&retry_report, "component.a").result(),
        ComponentReconstructionResult::Materialized(status)
            if status.state() == ParticipationState::Active
    ));
    assert_eq!(
        report_outcome(&retry_report, "component.b").result(),
        &ComponentReconstructionResult::AlreadyConverged
    );
    assert_eq!(attempts_a.load(Ordering::SeqCst), 2);
    assert_eq!(attempts_b.load(Ordering::SeqCst), 1);

    composition.stop().expect("stop runtime");
}

#[test]
fn reconstruction_reports_controls_in_component_id_order() {
    let (mut composition, rails) = fixture(
        &[],
        [
            definition("component.b", |_| Ok(Health::Healthy)),
            definition("component.a", |_| Ok(Health::Healthy)),
        ],
    );
    let component_a = component(&rails.runtime, "component.a");
    let component_b = component(&rails.runtime, "component.b");

    rails.control.enable(component_b).expect("enable b");
    rails.control.enable(component_a).expect("enable a");

    let report = rails.reconstruction.reconstruct().expect("reconstruct");
    let ordered: Vec<_> = report
        .outcomes()
        .iter()
        .map(|outcome| outcome.component_id().to_string())
        .collect();
    assert_eq!(ordered, vec!["component.a", "component.b"]);

    composition.stop().expect("stop runtime");
}

#[test]
fn reconstruction_updates_readiness_without_mutating_desired_or_semantic_truth() {
    let (mut composition, rails) = fixture(
        &["component.a"],
        [
            definition_with_operation("component.a", |scope| {
                scope.operation(operation_key(), |input: EchoInput| async move {
                    Ok(EchoOutput(input.0))
                })?;
                Ok(Health::Healthy)
            }),
            definition("component.b", |_| Ok(Health::Healthy)),
        ],
    );
    let component_a = component(&rails.runtime, "component.a");
    let component_b = component(&rails.runtime, "component.b");
    assert_eq!(
        rails.readiness.aggregate_readiness().status().lifecycle(),
        ComponentRuntimeLifecycle::Degraded
    );

    rails
        .requirements
        .register(requirement(
            component_a.clone(),
            component_b.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("requirement");
    rails.control.enable(component_a.clone()).expect("enable a");
    rails.control.enable(component_b.clone()).expect("enable b");

    let report = rails.reconstruction.reconstruct().expect("reconstruct");
    assert_eq!(report.outcomes().len(), 2);
    assert_eq!(
        rails.readiness.aggregate_readiness().status().lifecycle(),
        ComponentRuntimeLifecycle::Ready
    );
    assert_eq!(
        rails
            .control
            .control(component_a.component_id())
            .expect("control a")
            .desired(),
        ComponentDesiredState::Enabled
    );

    rails
        .control
        .disable(component_a.clone())
        .expect("disable a");
    rails
        .control
        .disable(component_b.clone())
        .expect("disable b");
    rails.reconstruction.reconstruct().expect("dematerialize");
    assert_eq!(
        rails
            .requirements
            .requirements(component_a.component_id())
            .len(),
        1
    );

    composition.stop().expect("stop runtime");
}

#[test]
fn reconstruction_materialized_runtime_executes_normally_after_convergence() {
    let (mut composition, rails) = fixture(
        &[],
        [definition_with_operation("component.a", |scope| {
            scope.operation(operation_key(), |input: EchoInput| async move {
                Ok(EchoOutput(input.0))
            })?;
            Ok(Health::Healthy)
        })],
    );
    let component_a = component(&rails.runtime, "component.a");
    rails.control.enable(component_a.clone()).expect("enable");
    rails.reconstruction.reconstruct().expect("reconstruct");

    let context = rails
        .invocation
        .begin_component(
            rails
                .registry
                .component(component_a.component_id())
                .expect("active component")
                .participation()
                .clone(),
        )
        .expect("context");
    let output = block_on(rails.operations.invoke_with_context(
        context,
        &operation_key(),
        EchoInput("hello"),
    ))
    .expect("operation");
    assert_eq!(output, EchoOutput("hello"));

    composition.stop().expect("stop runtime");
}

#[test]
fn reconstruction_rejects_stopped_runtime_globally() {
    let (mut composition, rails) =
        fixture(&[], [definition("component.a", |_| Ok(Health::Healthy))]);
    let component_a = component(&rails.runtime, "component.a");
    rails.control.enable(component_a.clone()).expect("enable");
    composition.stop().expect("stop runtime");
    assert_eq!(
        rails.reconstruction.reconstruct(),
        Err(ComponentError::ComponentReconstructionUnavailableLifecycle(
            ComponentRuntimeLifecycle::Stopped,
        ))
    );
    assert_eq!(
        rails.registry.component(component_a.component_id()),
        Err(ComponentError::UnknownComponent(
            component_a.component_id().clone(),
        ))
    );
}

#[test]
fn reconstruction_distinguishes_attached_unattached_and_undeclared() {
    let bare = (
        ComponentDeclaration::new(
            ComponentId::new("component.bare").expect("component id"),
            Vec::new(),
        ),
        None,
    );
    let (mut composition, rails) = fixture(
        &[],
        [
            definition("component.attached", |_| Ok(Health::Healthy)),
            bare,
        ],
    );
    let attached = component(&rails.runtime, "component.attached");
    let bare_component = component(&rails.runtime, "component.bare");
    let ghost = component(&rails.runtime, "component.ghost");

    rails.control.enable(attached).expect("enable attached");
    rails.control.enable(bare_component).expect("enable bare");
    rails.control.enable(ghost).expect("enable ghost");

    let report = rails.reconstruction.reconstruct().expect("reconstruct");
    assert!(
        matches!(
            report_outcome(&report, "component.attached").result(),
            ComponentReconstructionResult::Materialized(_)
        ),
        "declared component with attachment converges through materialization"
    );
    assert_eq!(
        report_outcome(&report, "component.bare").result(),
        &ComponentReconstructionResult::MissingRuntimeAttachment
    );
    assert_eq!(
        report_outcome(&report, "component.ghost").result(),
        &ComponentReconstructionResult::UndeclaredComponent
    );

    composition.stop().expect("stop runtime");
}
