use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, Wake, Waker};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractRequirement, Health,
    Instance, InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId, ModuleRuntime,
};

use crate::{
    ComponentControlRail, ComponentControlSnapshot, ComponentControlSnapshotEntry,
    ComponentDesiredState, ComponentError, ComponentHost, ComponentHostModule, ComponentId,
    ComponentInstanceBinding, ComponentParticipationRealization, ComponentParticipationScope,
    ComponentReconstructionRail, ComponentRegistry, InvocationRail, OperationId, OperationKey,
    OperationRail,
};

const OPERATION_ID: &str = "fabric.component.snapshot.echo";

#[derive(Clone, Debug, PartialEq, Eq)]
struct EchoInput(&'static str);

#[derive(Clone, Debug, PartialEq, Eq)]
struct EchoOutput(&'static str);

#[derive(Clone)]
struct Rails {
    runtime: Arc<ComponentHost>,
    control: Arc<ComponentControlRail>,
    registry: Arc<ComponentRegistry>,
    reconstruction: Arc<ComponentReconstructionRail>,
    invocation: Arc<InvocationRail>,
    operations: Arc<OperationRail>,
}

type Capture = Arc<Mutex<Option<Rails>>>;

#[derive(Clone)]
struct CaptureModule {
    module_id: ModuleId,
    runtime: ContractRequirement<ComponentHost>,
    control: ContractRequirement<ComponentControlRail>,
    registry: ContractRequirement<ComponentRegistry>,
    reconstruction: ContractRequirement<ComponentReconstructionRail>,
    invocation: ContractRequirement<InvocationRail>,
    operations: ContractRequirement<OperationRail>,
    capture: Capture,
}

impl CaptureModule {
    fn new(capture: Capture) -> Self {
        Self {
            module_id: ModuleId::new("runtime.snapshot.capture").expect("module id"),
            runtime: ContractRequirement::provisional(crate::component_host_contract_id()),
            control: ContractRequirement::provisional(crate::component_control_contract_id()),
            registry: ContractRequirement::provisional(crate::component_registry_contract_id()),
            reconstruction: ContractRequirement::provisional(
                crate::component_reconstruction_contract_id(),
            ),
            invocation: ContractRequirement::provisional(crate::invocation_contract_id()),
            operations: ContractRequirement::provisional(crate::operation_rail_contract_id()),
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
            self.control.id().clone(),
            self.registry.id().clone(),
            self.reconstruction.id().clone(),
            self.invocation.id().clone(),
            self.operations.id().clone(),
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
            control: bindings.resolve(&self.control).map_err(module_error)?,
            registry: bindings.resolve(&self.registry).map_err(module_error)?,
            reconstruction: bindings
                .resolve(&self.reconstruction)
                .map_err(module_error)?,
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

fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
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

fn component(runtime: &ComponentHost, component_id: &str) -> ComponentInstanceBinding {
    ComponentInstanceBinding::bind(
        ComponentId::new(component_id).expect("component id"),
        runtime,
    )
}

fn operation_key() -> OperationKey<EchoInput, EchoOutput> {
    OperationKey::new(
        OperationId::new(OPERATION_ID).expect("operation id"),
        crate::OperationTypeId::new("fabric.test.snapshot.echo.input").expect("input type id"),
        crate::OperationTypeId::new("fabric.test.snapshot.echo.output").expect("output type id"),
    )
}

fn declaration_with_operation(component_id: &str) -> crate::ComponentDeclaration {
    crate::ComponentDeclaration::new(
        ComponentId::new(component_id).expect("component id"),
        vec![operation_key().definition().clone()],
    )
}

fn empty_declaration(component_id: &str) -> crate::ComponentDeclaration {
    crate::ComponentDeclaration::new(
        ComponentId::new(component_id).expect("component id"),
        Vec::new(),
    )
}

fn definition_with_operation(
    component_id: &str,
) -> (
    crate::ComponentDeclaration,
    Option<ComponentParticipationRealization>,
) {
    (
        declaration_with_operation(component_id),
        Some(ComponentParticipationRealization::new(
            ComponentId::new(component_id).expect("component id"),
            |scope: &ComponentParticipationScope| {
                scope.operation_with_context(
                    operation_key(),
                    |_context, input: EchoInput| async move { Ok(EchoOutput(input.0)) },
                )?;
                Ok(Health::Healthy)
            },
        )),
    )
}

fn passive_definition(
    component_id: &str,
) -> (
    crate::ComponentDeclaration,
    Option<ComponentParticipationRealization>,
) {
    (empty_declaration(component_id), None)
}

fn fixture(
    instance_id: &str,
    components: impl IntoIterator<
        Item = (
            crate::ComponentDeclaration,
            Option<ComponentParticipationRealization>,
        ),
    >,
    control_snapshot: Option<ComponentControlSnapshot>,
) -> (Instance, Rails) {
    let capture = Arc::new(Mutex::new(None));
    let (declarations, attachments): (Vec<_>, Vec<_>) = components.into_iter().unzip();
    let attachments = attachments.into_iter().flatten().collect::<Vec<_>>();
    let module = match control_snapshot {
        Some(control_snapshot) => ComponentHostModule::with_configuration_and_control_snapshot(
            crate::ComponentReadinessPolicy::empty(),
            declarations,
            attachments,
            control_snapshot,
        )
        .expect("component runtime with snapshot"),
        None => ComponentHostModule::with_configuration(
            crate::ComponentReadinessPolicy::empty(),
            declarations,
            attachments,
        )
        .expect("component runtime"),
    };
    let composition =
        CompositionBuilder::new(CompositionId::new("runtime.snapshot").expect("composition id"))
            .register_block(
                BlockBuilder::new(BlockId::new("runtime.snapshot.block").expect("block id"))
                    .register_module(module)
                    .register_module(CaptureModule::new(Arc::clone(&capture)))
                    .build(),
            )
            .build()
            .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new(instance_id).expect("instance id"))
        .expect("materialize instance");
    instance.start().expect("start instance");
    let rails = capture.lock().expect("capture lock").take().expect("rails");
    (instance, rails)
}

#[test]
fn snapshot_is_empty_until_explicit_controls_exist_and_preserves_deterministic_entries() {
    let (mut instance, rails) = fixture("runtime.snapshot", Vec::new(), None);

    let empty = rails.control.snapshot();
    assert_eq!(
        empty.instance_id(),
        &InstanceId::new("runtime.snapshot").expect("instance id"),
    );
    assert!(empty.entries().is_empty());

    let component_b = component(rails.runtime.as_ref(), "component.beta");
    let component_a = component(rails.runtime.as_ref(), "component.alpha");
    rails.control.enable(component_b).expect("enable beta");
    rails.control.disable(component_a).expect("disable alpha");

    let snapshot = rails.control.snapshot();
    assert_eq!(
        snapshot.instance_id(),
        &InstanceId::new("runtime.snapshot").expect("instance id"),
    );
    assert_eq!(
        snapshot.entries(),
        &[
            ComponentControlSnapshotEntry::new(
                ComponentId::new("component.alpha").expect("component id"),
                ComponentDesiredState::Disabled,
            ),
            ComponentControlSnapshotEntry::new(
                ComponentId::new("component.beta").expect("component id"),
                ComponentDesiredState::Enabled,
            ),
        ],
    );

    instance.stop().expect("stop instance");
    assert_eq!(rails.control.snapshot(), snapshot);
}

#[test]
fn snapshot_constructor_rejects_duplicate_component_entries() {
    let error = ComponentControlSnapshot::new(
        InstanceId::new("runtime.snapshot").expect("instance id"),
        [
            ComponentControlSnapshotEntry::new(
                ComponentId::new("component.dup").expect("component id"),
                ComponentDesiredState::Enabled,
            ),
            ComponentControlSnapshotEntry::new(
                ComponentId::new("component.dup").expect("component id"),
                ComponentDesiredState::Disabled,
            ),
        ],
    )
    .expect_err("duplicate snapshot entries rejected");

    assert_eq!(
        error,
        ComponentError::DuplicateComponentControlSnapshotEntry(
            ComponentId::new("component.dup").expect("component id"),
        ),
    );
}

#[test]
fn default_runtime_module_remains_instance_neutral_without_snapshot() {
    ComponentHostModule::new();
}

#[test]
fn snapshot_carries_stable_instance_identity_but_defers_validation_to_materialization() {
    let snapshot = ComponentControlSnapshot::new(
        InstanceId::new("runtime.snapshot").expect("instance id"),
        [ComponentControlSnapshotEntry::new(
            ComponentId::new("component.alpha").expect("component id"),
            ComponentDesiredState::Enabled,
        )],
    )
    .expect("snapshot");

    ComponentHostModule::with_control_snapshot(snapshot.clone())
        .expect("snapshot should load without declaration-time instance binding");

    let (_instance, rails) = fixture("runtime.snapshot", Vec::new(), Some(snapshot.clone()));
    assert_eq!(
        rails.control.snapshot().instance_id(),
        snapshot.instance_id()
    );
}

#[test]
fn snapshot_rejects_materialization_for_different_instance_id() {
    let snapshot = ComponentControlSnapshot::new(
        InstanceId::new("runtime.snapshot.a").expect("instance id"),
        [ComponentControlSnapshotEntry::new(
            ComponentId::new("component.alpha").expect("component id"),
            ComponentDesiredState::Enabled,
        )],
    )
    .expect("snapshot");

    let capture = Arc::new(Mutex::new(None));
    let composition =
        CompositionBuilder::new(CompositionId::new("runtime.snapshot.reject").expect("id"))
            .register_block(
                BlockBuilder::new(BlockId::new("runtime.snapshot.reject.block").expect("id"))
                    .register_module(
                        ComponentHostModule::with_control_snapshot(snapshot)
                            .expect("module with snapshot"),
                    )
                    .register_module(CaptureModule::new(Arc::clone(&capture)))
                    .build(),
            )
            .build()
            .expect("composition");

    let error = composition
        .materialize(InstanceId::new("runtime.snapshot.b").expect("instance id"))
        .expect_err("different instance id should reject snapshot");
    assert!(
        error.to_string().contains(
            &ComponentError::ComponentControlSnapshotInstanceMismatch {
                snapshot_instance_id: InstanceId::new("runtime.snapshot.a").expect("instance id"),
                instance_id: InstanceId::new("runtime.snapshot.b").expect("instance id"),
            }
            .to_string()
        )
    );
}

#[test]
fn snapshot_load_round_trip_reconstructs_fresh_runtime_without_reviving_old_authority() {
    let runtime_definitions = vec![
        definition_with_operation("component.a"),
        passive_definition("component.b"),
        passive_definition("component.c"),
    ];
    let (mut runtime_one, rails_one) = fixture("runtime.snapshot", runtime_definitions, None);
    let component_a = component(rails_one.runtime.as_ref(), "component.a");
    let component_b = component(rails_one.runtime.as_ref(), "component.b");

    rails_one
        .control
        .enable(component_a.clone())
        .expect("enable a");
    rails_one
        .control
        .disable(component_b.clone())
        .expect("disable b");
    let snapshot = rails_one.control.snapshot();
    assert_eq!(
        snapshot.entries(),
        &[
            ComponentControlSnapshotEntry::new(
                ComponentId::new("component.a").expect("component id"),
                ComponentDesiredState::Enabled,
            ),
            ComponentControlSnapshotEntry::new(
                ComponentId::new("component.b").expect("component id"),
                ComponentDesiredState::Disabled,
            ),
        ],
    );

    let first_report = rails_one
        .reconstruction
        .reconstruct()
        .expect("reconstruct r1");
    assert_eq!(first_report.outcomes().len(), 2);
    let old_participation = rails_one
        .registry
        .component(component_a.component_id())
        .expect("r1 component a")
        .participation()
        .clone();
    let old_context = rails_one
        .invocation
        .begin_component(old_participation.clone())
        .expect("r1 component context");
    let runtime_one_id = rails_one
        .runtime
        .current_status()
        .generation()
        .expect("runtime one generation");

    let runtime_definitions = vec![
        definition_with_operation("component.a"),
        passive_definition("component.b"),
        passive_definition("component.c"),
    ];
    let (mut runtime_two, rails_two) = fixture(
        "runtime.snapshot",
        runtime_definitions,
        Some(snapshot.clone()),
    );
    let runtime_two_id = rails_two
        .runtime
        .current_status()
        .generation()
        .expect("runtime two generation");

    assert_eq!(
        rails_two
            .control
            .controls()
            .iter()
            .map(|control| (
                control.component().component_id().as_str(),
                control.desired(),
            ))
            .collect::<Vec<_>>(),
        vec![
            ("component.a", ComponentDesiredState::Enabled),
            ("component.b", ComponentDesiredState::Disabled),
        ],
    );
    assert_ne!(runtime_one_id, runtime_two_id);
    assert!(rails_two.registry.components().is_empty());
    assert_eq!(rails_two.control.snapshot(), snapshot);
    assert_eq!(
        snapshot.instance_id(),
        &InstanceId::new("runtime.snapshot").expect("instance id"),
    );

    let second_report = rails_two
        .reconstruction
        .reconstruct()
        .expect("reconstruct r2");
    assert_eq!(second_report.outcomes().len(), 2);
    let new_participation = rails_two
        .registry
        .component(&ComponentId::new("component.a").expect("component id"))
        .expect("r2 component a")
        .participation()
        .clone();
    assert_eq!(
        rails_two
            .registry
            .component(&ComponentId::new("component.b").expect("component id")),
        Err(ComponentError::UnknownComponent(
            ComponentId::new("component.b").expect("component id"),
        )),
    );
    assert_eq!(snapshot, rails_two.control.snapshot());
    assert_eq!(
        old_participation.participation_id(),
        new_participation.participation_id()
    );
    assert_ne!(
        old_participation.generation(),
        new_participation.generation()
    );

    assert_eq!(
        rails_two
            .registry
            .update_health(&old_participation, Health::Degraded),
        Err(ComponentError::StaleComponentParticipation(
            old_participation.clone(),
        )),
    );
    assert_eq!(
        block_on(rails_two.operations.invoke_with_context(
            old_context.clone(),
            &operation_key(),
            EchoInput("stale"),
        )),
        Err(ComponentError::InvocationContextGenerationMismatch {
            context_generation: old_context.generation(),
            generation: runtime_two_id,
        }),
    );

    rails_two
        .control
        .disable(component(rails_two.runtime.as_ref(), "component.a"))
        .expect("disable loaded control");
    assert_eq!(
        rails_two.control.snapshot().entries(),
        &[
            ComponentControlSnapshotEntry::new(
                ComponentId::new("component.a").expect("component id"),
                ComponentDesiredState::Disabled,
            ),
            ComponentControlSnapshotEntry::new(
                ComponentId::new("component.b").expect("component id"),
                ComponentDesiredState::Disabled,
            ),
        ],
    );

    runtime_one.stop().expect("stop runtime");
    runtime_two.stop().expect("stop runtime");
}

#[test]
fn snapshot_load_keeps_unknown_controls_and_reconstruction_reports_missing_runtime_definitions() {
    let snapshot = ComponentControlSnapshot::new(
        InstanceId::new("runtime.snapshot").expect("instance id"),
        [
            ComponentControlSnapshotEntry::new(
                ComponentId::new("component.disabled").expect("component id"),
                ComponentDesiredState::Disabled,
            ),
            ComponentControlSnapshotEntry::new(
                ComponentId::new("component.enabled").expect("component id"),
                ComponentDesiredState::Enabled,
            ),
        ],
    )
    .expect("snapshot");
    let (mut instance, rails) = fixture("runtime.snapshot", Vec::new(), Some(snapshot.clone()));

    assert!(rails.registry.components().is_empty());
    assert_eq!(rails.control.snapshot(), snapshot);

    let report = rails.reconstruction.reconstruct().expect("reconstruct");
    assert_eq!(report.outcomes().len(), 2);
    assert_eq!(
        report
            .outcomes()
            .iter()
            .map(|outcome| (outcome.component_id().as_str(), outcome.result().clone()))
            .collect::<Vec<_>>(),
        vec![
            (
                "component.disabled",
                crate::ComponentReconstructionResult::AlreadyConverged,
            ),
            (
                "component.enabled",
                crate::ComponentReconstructionResult::UndeclaredComponent,
            ),
        ],
    );

    instance.stop().expect("stop instance");
}
