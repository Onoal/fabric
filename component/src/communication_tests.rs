use std::sync::{Arc, Mutex};

use fabric_core::{
    BlockBuilder, BlockId, Composition, CompositionBuilder, CompositionId, ContractId, ContractKey,
    ContractRequirement, Health, Instance, InstanceId, ModuleBindings, ModuleContract, ModuleError,
    ModuleId, ModuleRuntime,
};

use crate::{
    ComponentCommunication, ComponentContract, ComponentDeclaration, ComponentError, ComponentHost,
    ComponentHostModule, ComponentHostService, ComponentId, ComponentInstanceBinding,
    ComponentParticipation, ComponentRegistry, ComponentRequirementKind, ComponentRequirementRail,
    ResolvedComponentRequirement,
};

const NOTES_CONTRACT_ID: &str = "fabric.component.tests.notes";

#[derive(Clone)]
struct NotesContract;

impl NotesContract {
    fn reply(&self, message: &str) -> String {
        format!("notes:{message}")
    }
}

fn notes_contract_key() -> ContractKey<NotesContract> {
    ContractKey::provisional(
        ContractId::new(NOTES_CONTRACT_ID).expect("static synthetic contract id"),
    )
}

struct Rails {
    runtime: Arc<ComponentHost>,
    registry: Arc<ComponentRegistry>,
    communication: Arc<ComponentCommunication>,
    requirements: Arc<ComponentRequirementRail>,
    core_resolved_notes: Arc<NotesContract>,
}

type CapturedRails = Arc<Mutex<Option<Rails>>>;

#[derive(Clone)]
struct SyntheticContractProvider {
    module_id: ModuleId,
}

impl SyntheticContractProvider {
    fn new() -> Self {
        Self {
            module_id: ModuleId::new("runtime.communication.provider").expect("module id"),
        }
    }
}

impl ModuleRuntime for SyntheticContractProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![notes_contract_key().id().clone()]
            .into_iter()
            .map(fabric_core::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        Vec::new()
            .into_iter()
            .map(fabric_core::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &notes_contract_key(),
            Arc::new(NotesContract),
        )])
    }

    fn bind(&mut self, _bindings: &ModuleBindings) -> Result<(), ModuleError> {
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
struct RailsCapture {
    module_id: ModuleId,
    runtime_requirement: ContractRequirement<ComponentHost>,
    registry_requirement: ContractRequirement<ComponentRegistry>,
    communication_requirement: ContractRequirement<ComponentCommunication>,
    requirement_requirement: ContractRequirement<ComponentRequirementRail>,
    notes_requirement: ContractRequirement<NotesContract>,
    capture: CapturedRails,
}

impl RailsCapture {
    fn new(capture: CapturedRails) -> Self {
        Self {
            module_id: ModuleId::new("runtime.communication.capture").expect("module id"),
            runtime_requirement: ContractRequirement::provisional(
                crate::component_host_contract_id(),
            ),
            registry_requirement: ContractRequirement::provisional(
                crate::component_registry_contract_id(),
            ),
            communication_requirement: ContractRequirement::provisional(
                crate::component_communication_contract_id(),
            ),
            requirement_requirement: ContractRequirement::provisional(
                crate::component_requirement_contract_id(),
            ),
            notes_requirement: ContractRequirement::provisional(notes_contract_key().id().clone()),
            capture,
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
            self.runtime_requirement.id().clone(),
            self.registry_requirement.id().clone(),
            self.communication_requirement.id().clone(),
            self.requirement_requirement.id().clone(),
            self.notes_requirement.id().clone(),
        ]
        .into_iter()
        .map(fabric_core::ContractRequirementDeclaration::provisional)
        .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let rails = Rails {
            runtime: bindings
                .resolve(&self.runtime_requirement)
                .map_err(module_error)?,
            registry: bindings
                .resolve(&self.registry_requirement)
                .map_err(module_error)?,
            communication: bindings
                .resolve(&self.communication_requirement)
                .map_err(module_error)?,
            requirements: bindings
                .resolve(&self.requirement_requirement)
                .map_err(module_error)?,
            core_resolved_notes: bindings
                .resolve(&self.notes_requirement)
                .map_err(module_error)?,
        };
        *self.capture.lock().expect("capture lock") = Some(rails);
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

fn start_instance(composition: Composition, instance_id: &str) -> Instance {
    let mut instance = composition
        .materialize(InstanceId::new(instance_id.to_owned()).expect("instance id"))
        .expect("materialize instance");
    instance.start().expect("start instance");
    instance
}

fn fixture() -> (Instance, Rails) {
    let capture = Arc::new(Mutex::new(None));
    let composition = CompositionBuilder::new(
        CompositionId::new("runtime.communication").expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("runtime.communication.block").expect("block id"))
            .register_module(
                ComponentHostModule::with_components(
                    [
                        "component.available.caller",
                        "component.available.provider",
                        "component.consumer",
                        "component.notes",
                        "component.unrelated",
                    ]
                    .into_iter()
                    .map(|component_id| {
                        ComponentDeclaration::new(
                            ComponentId::new(component_id).expect("component id"),
                            Vec::new(),
                        )
                    }),
                    Vec::new(),
                )
                .expect("component host"),
            )
            .register_module(SyntheticContractProvider::new())
            .register_module(RailsCapture::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");
    let instance = start_instance(composition, "runtime.communication");
    let rails = capture
        .lock()
        .expect("capture lock")
        .take()
        .expect("captured rails");
    (instance, rails)
}

fn component(rails: &Rails, component_id: &str) -> ComponentInstanceBinding {
    ComponentInstanceBinding::bind(
        ComponentId::new(component_id).expect("component id"),
        rails.runtime.as_ref(),
    )
}

fn notes_handle(
    rails: &Rails,
    consumer: ComponentInstanceBinding,
    provider: ComponentInstanceBinding,
) -> ComponentContract<NotesContract> {
    let handle = rails
        .communication
        .bind(
            ResolvedComponentRequirement::synthetic(
                consumer.clone(),
                notes_contract_key().id().clone(),
                provider,
                ComponentRequirementKind::Required,
            ),
            Arc::clone(&rails.core_resolved_notes),
        )
        .expect("bind core-resolved contract");
    assert_eq!(
        rails
            .requirements
            .requirements(consumer.component_id())
            .len(),
        1
    );
    handle
}

fn participate(rails: &Rails, component: ComponentInstanceBinding) -> ComponentParticipation {
    let participation = rails
        .registry
        .register(component, Health::Healthy)
        .expect("participate")
        .participation()
        .clone();
    rails
        .registry
        .activate(&participation)
        .expect("activate participation");
    participation
}

#[test]
fn core_resolved_contract_communicates_between_participating_components() {
    let (mut composition, rails) = fixture();
    let provider = component(&rails, "component.notes");
    let caller = component(&rails, "component.consumer");
    participate(&rails, provider.clone());
    let caller_component = caller.clone();
    let caller = participate(&rails, caller);

    let notes = notes_handle(&rails, caller_component, provider.clone());
    assert_eq!(notes.provider(), &provider);
    assert_eq!(
        notes.call(&caller, |contract| contract.reply("hello")),
        Ok("notes:hello".to_owned())
    );
    composition.stop().expect("stop runtime");
}

#[test]
fn core_resolved_contract_rejects_absent_provider() {
    let (mut composition, rails) = fixture();
    let provider = component(&rails, "component.notes");
    let caller = component(&rails, "component.consumer");
    let caller_component = caller.clone();
    let caller = participate(&rails, caller);

    let notes = notes_handle(&rails, caller_component, provider);
    assert_eq!(
        notes.call(&caller, |contract| contract.reply("hello")),
        Err(ComponentError::ComponentContractProviderNotParticipating(
            ComponentId::new("component.notes").expect("component id")
        ))
    );
    composition.stop().expect("stop runtime");
}

#[test]
fn stale_handle_rejects_new_calls_after_provider_leaves_and_allows_rejoin() {
    let (mut composition, rails) = fixture();
    let provider = component(&rails, "component.notes");
    let caller = component(&rails, "component.consumer");
    let provider_participation = participate(&rails, provider.clone());
    let caller_component = caller.clone();
    let caller = participate(&rails, caller);
    let notes = notes_handle(&rails, caller_component, provider.clone());
    assert!(
        notes
            .call(&caller, |contract| contract.reply("first"))
            .is_ok()
    );

    rails
        .registry
        .unregister(&provider_participation)
        .expect("leave");
    assert!(matches!(
        notes.call(&caller, |contract| contract.reply("stale")),
        Err(ComponentError::ComponentContractProviderNotParticipating(_))
    ));

    participate(&rails, provider);
    assert_eq!(
        notes.call(&caller, |contract| contract.reply("rejoined")),
        Ok("notes:rejoined".to_owned())
    );
    composition.stop().expect("stop runtime");
}

#[test]
fn stale_handle_rejects_new_calls_after_caller_leaves_without_affecting_another_caller() {
    let (mut composition, rails) = fixture();
    let provider = component(&rails, "component.notes");
    let caller = component(&rails, "component.consumer");
    let unrelated = component(&rails, "component.unrelated");
    participate(&rails, provider.clone());
    let caller_component = caller.clone();
    let caller = participate(&rails, caller);
    let unrelated = participate(&rails, unrelated);
    let notes = notes_handle(&rails, caller_component, provider);

    rails.registry.unregister(&caller).expect("caller leaves");
    assert!(matches!(
        notes.call(&caller, |contract| contract.reply("stale")),
        Err(ComponentError::ComponentContractCallerNotParticipating(_))
    ));
    assert_eq!(
        notes.call(&unrelated, |contract| contract.reply("still here")),
        Err(ComponentError::ComponentContractConsumerMismatch {
            expected_component_id: ComponentId::new("component.consumer").expect("component id"),
            actual_component_id: ComponentId::new("component.unrelated").expect("component id"),
        })
    );
    composition.stop().expect("stop runtime");
}

#[test]
fn call_that_started_while_participants_exist_may_finish_after_provider_leaves() {
    let (mut composition, rails) = fixture();
    let provider = component(&rails, "component.notes");
    let caller = component(&rails, "component.consumer");
    let provider_participation = participate(&rails, provider.clone());
    let caller_component = caller.clone();
    let caller = participate(&rails, caller);
    let notes = notes_handle(&rails, caller_component, provider.clone());
    let registry = Arc::clone(&rails.registry);

    assert_eq!(
        notes.call(&caller, move |contract| {
            registry
                .unregister(&provider_participation)
                .expect("provider leaves");
            contract.reply("finishes")
        }),
        Ok("notes:finishes".to_owned())
    );
    assert!(matches!(
        notes.call(&caller, |contract| contract.reply("next")),
        Err(ComponentError::ComponentContractProviderNotParticipating(_))
    ));
    composition.stop().expect("stop runtime");
}

#[test]
fn availability_gates_contract_caller_and_provider_without_reresolution() {
    let (mut composition, rails) = fixture();
    let provider = component(&rails, "component.available.provider");
    let caller = component(&rails, "component.available.caller");
    let provider_participation = participate(&rails, provider.clone());
    let caller_component = caller.clone();
    let caller_participation = participate(&rails, caller);
    let notes = notes_handle(&rails, caller_component, provider);

    rails
        .registry
        .update_health(&provider_participation, Health::Unavailable)
        .expect("provider unavailable");
    assert!(matches!(
        notes.call(&caller_participation, |contract| contract.reply("blocked")),
        Err(ComponentError::ComponentUnavailable(_))
    ));
    rails
        .registry
        .update_health(&provider_participation, Health::Healthy)
        .expect("provider recovered");
    assert_eq!(
        notes.call(&caller_participation, |contract| contract
            .reply("recovered")),
        Ok("notes:recovered".to_owned())
    );
    rails
        .registry
        .update_health(&caller_participation, Health::Unavailable)
        .expect("caller unavailable");
    assert!(matches!(
        notes.call(&caller_participation, |contract| contract.reply("blocked")),
        Err(ComponentError::ComponentUnavailable(_))
    ));
    composition.stop().expect("stop runtime");
}

#[test]
fn cross_instance_provider_and_caller_are_rejected() {
    let (mut composition, rails) = fixture();
    let local_provider = component(&rails, "component.notes");
    let local_caller = component(&rails, "component.consumer");
    participate(&rails, local_provider.clone());
    let local_caller_component = local_caller.clone();
    participate(&rails, local_caller);
    let _notes = notes_handle(&rails, local_caller_component, local_provider);
    let foreign = foreign_component("runtime.foreign", "component.foreign");

    assert!(
        rails
            .communication
            .bind(
                ResolvedComponentRequirement::synthetic(
                    component(&rails, "component.consumer"),
                    notes_contract_key().id().clone(),
                    foreign.clone(),
                    ComponentRequirementKind::Required,
                ),
                Arc::clone(&rails.core_resolved_notes),
            )
            .is_err()
    );
    composition.stop().expect("stop runtime");
}

fn foreign_component(instance_id: &str, component_id: &str) -> ComponentInstanceBinding {
    let contract = ComponentHost::new(Arc::new(StaticComponentRuntimeService {
        instance_id: InstanceId::new(instance_id).expect("instance id"),
    }));
    ComponentInstanceBinding::bind(
        ComponentId::new(component_id).expect("component id"),
        &contract,
    )
}

#[derive(Clone)]
struct StaticComponentRuntimeService {
    instance_id: InstanceId,
}

impl ComponentHostService for StaticComponentRuntimeService {
    fn instance_id(&self) -> InstanceId {
        self.instance_id.clone()
    }

    fn current_instance_id(&self) -> Result<InstanceId, ComponentError> {
        Ok(self.instance_id())
    }

    fn current_status(&self) -> crate::ComponentHostStatus {
        crate::ComponentHostStatus::new(
            self.instance_id(),
            None,
            crate::ComponentHostLifecycle::Stopped,
            Health::Unavailable,
        )
    }

    fn current_lifecycle(&self) -> crate::ComponentHostLifecycle {
        crate::ComponentHostLifecycle::Ready
    }

    fn current_health(&self) -> Health {
        Health::Healthy
    }
}
