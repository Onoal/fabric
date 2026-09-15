use std::sync::{Arc, Mutex};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractId, ContractRequirement,
    Health, Instance, InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId,
    ModuleRuntime,
};

use crate::{
    Component, ComponentCommunication, ComponentContract, ComponentDeclaration,
    ComponentDependencyAvailabilityBlocker, ComponentEffectiveAvailability, ComponentError,
    ComponentId, ComponentParticipation, ComponentRegistry, ComponentRequirementKind,
    ComponentRequirementRail, ComponentRuntime, ComponentRuntimeModule, ComponentRuntimeService,
    InvocationRail,
};

const NOTES_CONTRACT_ID: &str = "fabric.component.requirement.notes";

#[derive(Clone)]
struct Notes;

impl Notes {
    fn reply(&self, input: &str) -> String {
        format!("notes:{input}")
    }
}

struct Rails {
    runtime: Arc<ComponentRuntime>,
    registry: Arc<ComponentRegistry>,
    requirements: Arc<ComponentRequirementRail>,
    communication: Arc<ComponentCommunication>,
    invocation: Arc<InvocationRail>,
}

type Capture = Arc<Mutex<Option<Rails>>>;

#[derive(Clone)]
struct CaptureModule {
    module_id: ModuleId,
    runtime: ContractRequirement<ComponentRuntime>,
    registry: ContractRequirement<ComponentRegistry>,
    requirements: ContractRequirement<ComponentRequirementRail>,
    communication: ContractRequirement<ComponentCommunication>,
    invocation: ContractRequirement<InvocationRail>,
    capture: Capture,
}

impl CaptureModule {
    fn new(capture: Capture) -> Self {
        Self {
            module_id: ModuleId::new("component.requirement.capture").expect("module id"),
            runtime: ContractRequirement::provisional(crate::component_runtime_contract_id()),
            registry: ContractRequirement::provisional(crate::component_registry_contract_id()),
            requirements: ContractRequirement::provisional(
                crate::component_requirement_contract_id(),
            ),
            communication: ContractRequirement::provisional(
                crate::component_communication_contract_id(),
            ),
            invocation: ContractRequirement::provisional(crate::invocation_contract_id()),
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
            self.requirements.id().clone(),
            self.communication.id().clone(),
            self.invocation.id().clone(),
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
            requirements: bindings.resolve(&self.requirements).map_err(module_error)?,
            communication: bindings
                .resolve(&self.communication)
                .map_err(module_error)?,
            invocation: bindings.resolve(&self.invocation).map_err(module_error)?,
        });
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

fn module_error(error: impl std::fmt::Display) -> ModuleError {
    ModuleError::new(error.to_string())
}

fn fixture() -> (Instance, Rails) {
    let capture = Arc::new(Mutex::new(None));
    let composition = CompositionBuilder::new(
        CompositionId::new("component.requirement").expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("component.requirement.block").expect("block id"))
            .register_module(
                ComponentRuntimeModule::with_components(
                    [
                        "component.a",
                        "component.b",
                        "component.blocking",
                        "component.caller",
                        "component.consumer",
                        "component.local",
                        "component.optional",
                        "component.provider",
                        "component.provider.fallback",
                        "component.provider.required",
                        "component.provider.transitive",
                        "component.target",
                        "component.target.blocker",
                        "component.transitive",
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
            .register_module(CaptureModule::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("component.requirement").expect("instance id"))
        .expect("materialize instance");
    instance.start().expect("start instance");
    let rails = capture.lock().expect("capture lock").take().expect("rails");
    (instance, rails)
}

fn component(rails: &Rails, component_id: &str) -> Component {
    Component::bind(
        ComponentId::new(component_id).expect("component id"),
        rails.runtime.as_ref(),
    )
}

fn participate(rails: &Rails, component: Component, health: Health) -> ComponentParticipation {
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

fn requirement(
    consumer: Component,
    provider: Component,
    kind: ComponentRequirementKind,
) -> crate::ResolvedComponentRequirement {
    crate::ResolvedComponentRequirement::synthetic(
        consumer,
        ContractId::new(NOTES_CONTRACT_ID).expect("contract id"),
        provider,
        kind,
    )
}

fn notes_handle(
    rails: &Rails,
    consumer: Component,
    provider: Component,
    kind: ComponentRequirementKind,
) -> ComponentContract<Notes> {
    rails
        .communication
        .bind(requirement(consumer, provider, kind), Arc::new(Notes))
        .expect("bind notes")
}

fn foreign_component(instance_id: &str, component_id: &str) -> Component {
    let runtime = ComponentRuntime::new(Arc::new(StaticComponentRuntimeService {
        instance_id: InstanceId::new(instance_id).expect("instance id"),
    }));
    Component::bind(
        ComponentId::new(component_id).expect("component id"),
        &runtime,
    )
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

    fn current_status(&self) -> crate::ComponentRuntimeStatus {
        crate::ComponentRuntimeStatus::new(
            self.instance_id(),
            None,
            crate::ComponentRuntimeLifecycle::Ready,
            Health::Healthy,
        )
    }

    fn current_lifecycle(&self) -> crate::ComponentRuntimeLifecycle {
        crate::ComponentRuntimeLifecycle::Ready
    }

    fn current_health(&self) -> Health {
        Health::Healthy
    }
}

fn provider_blocker(
    availability: &ComponentEffectiveAvailability,
) -> &ComponentDependencyAvailabilityBlocker {
    match availability {
        ComponentEffectiveAvailability::RequiredDependencyUnavailable(blocker) => blocker,
        other => panic!("expected dependency blocker, got {other:?}"),
    }
}

#[test]
fn required_dependency_availability_recovers_without_rewriting_consumer_health() {
    let (mut instance, rails) = fixture();
    let consumer = component(&rails, "component.consumer");
    let provider = component(&rails, "component.provider");
    let consumer_participation = participate(&rails, consumer.clone(), Health::Healthy);
    let provider_participation = participate(&rails, provider.clone(), Health::Healthy);
    rails
        .requirements
        .register(requirement(
            consumer.clone(),
            provider.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("register requirement");

    assert_eq!(
        rails
            .requirements
            .effective_availability(consumer.component_id())
            .expect("availability"),
        ComponentEffectiveAvailability::Available
    );
    rails
        .registry
        .update_health(&provider_participation, Health::Unavailable)
        .expect("provider unavailable");
    let availability = rails
        .requirements
        .effective_availability(consumer.component_id())
        .expect("availability");
    let blocker = provider_blocker(&availability);
    assert_eq!(blocker.provider(), &provider);
    assert_eq!(blocker.contract_id().as_str(), NOTES_CONTRACT_ID);
    assert_eq!(
        blocker.provider_availability(),
        &ComponentEffectiveAvailability::IntrinsicUnavailable {
            component: provider.clone(),
            health: Health::Unavailable,
        }
    );
    assert_eq!(
        rails
            .invocation
            .begin_component(consumer_participation.clone()),
        Err(ComponentError::ComponentUnavailable(
            consumer.component_id().clone()
        ))
    );
    rails
        .registry
        .update_health(&provider_participation, Health::Healthy)
        .expect("provider recovered");
    assert_eq!(
        rails
            .requirements
            .effective_availability(consumer.component_id())
            .expect("availability"),
        ComponentEffectiveAvailability::Available
    );
    assert!(
        rails
            .invocation
            .begin_component(consumer_participation)
            .is_ok()
    );
    instance.stop();
}

#[test]
fn required_relationship_survives_provider_participation_churn() {
    let (mut instance, rails) = fixture();
    let consumer = component(&rails, "component.consumer");
    let provider = component(&rails, "component.provider");
    participate(&rails, consumer.clone(), Health::Healthy);
    let provider_participation = participate(&rails, provider.clone(), Health::Healthy);
    rails
        .requirements
        .register(requirement(
            consumer.clone(),
            provider.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("register requirement");

    rails
        .registry
        .unregister(&provider_participation)
        .expect("provider leaves");
    let availability = rails
        .requirements
        .effective_availability(consumer.component_id())
        .expect("availability");
    let blocker = provider_blocker(&availability);
    assert_eq!(blocker.provider(), &provider);
    assert_eq!(
        blocker.provider_availability(),
        &ComponentEffectiveAvailability::NotParticipating
    );

    participate(&rails, provider, Health::Healthy);
    assert_eq!(
        rails
            .requirements
            .requirements(consumer.component_id())
            .len(),
        1
    );
    assert_eq!(
        rails
            .requirements
            .effective_availability(consumer.component_id())
            .expect("availability"),
        ComponentEffectiveAvailability::Available
    );
    instance.stop();
}

#[test]
fn optional_requirement_does_not_block_consumer_but_bound_call_still_fails() {
    let (mut instance, rails) = fixture();
    let consumer = component(&rails, "component.consumer");
    let optional_provider = component(&rails, "component.optional");
    let transitive_provider = component(&rails, "component.transitive");
    let consumer_participation = participate(&rails, consumer.clone(), Health::Healthy);
    let optional_participation = participate(&rails, optional_provider.clone(), Health::Healthy);
    let transitive_participation =
        participate(&rails, transitive_provider.clone(), Health::Healthy);
    rails
        .requirements
        .register(requirement(
            consumer.clone(),
            optional_provider.clone(),
            ComponentRequirementKind::Optional,
        ))
        .expect("register optional");
    rails
        .requirements
        .register(requirement(
            optional_provider.clone(),
            transitive_provider.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("register transitive");
    let notes = notes_handle(
        &rails,
        consumer.clone(),
        optional_provider.clone(),
        ComponentRequirementKind::Optional,
    );

    rails
        .registry
        .update_health(&transitive_participation, Health::Unavailable)
        .expect("transitive unavailable");
    assert_eq!(
        rails
            .requirements
            .effective_availability(consumer.component_id())
            .expect("consumer availability"),
        ComponentEffectiveAvailability::Available
    );
    let optional_availability = rails
        .requirements
        .effective_availability(optional_provider.component_id())
        .expect("optional availability");
    let blocker = provider_blocker(&optional_availability);
    assert_eq!(blocker.provider(), &transitive_provider);
    assert_eq!(
        notes.call(&consumer_participation, |contract| contract
            .reply("blocked")),
        Err(ComponentError::ComponentUnavailable(
            optional_provider.component_id().clone()
        ))
    );

    rails
        .registry
        .update_health(&optional_participation, Health::Healthy)
        .expect("keep optional provider intrinsic health");
    instance.stop();
}

#[test]
fn effective_availability_is_transitive_and_has_no_fallback_provider_lookup() {
    let (mut instance, rails) = fixture();
    let consumer = component(&rails, "component.consumer");
    let required_provider = component(&rails, "component.provider.required");
    let fallback_provider = component(&rails, "component.provider.fallback");
    let transitive_provider = component(&rails, "component.provider.transitive");
    participate(&rails, consumer.clone(), Health::Healthy);
    participate(&rails, required_provider.clone(), Health::Healthy);
    participate(&rails, fallback_provider, Health::Healthy);
    let transitive_participation =
        participate(&rails, transitive_provider.clone(), Health::Healthy);
    rails
        .requirements
        .register(requirement(
            consumer.clone(),
            required_provider.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("register consumer requirement");
    rails
        .requirements
        .register(requirement(
            required_provider.clone(),
            transitive_provider.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("register transitive requirement");

    rails
        .registry
        .update_health(&transitive_participation, Health::Unavailable)
        .expect("transitive unavailable");
    let consumer_availability = rails
        .requirements
        .effective_availability(consumer.component_id())
        .expect("consumer availability");
    let consumer_blocker = provider_blocker(&consumer_availability);
    assert_eq!(consumer_blocker.provider(), &required_provider);
    let provider_blocker = provider_blocker(consumer_blocker.provider_availability());
    assert_eq!(provider_blocker.provider(), &transitive_provider);
    instance.stop();
}

#[test]
fn caller_and_provider_admission_use_effective_availability() {
    let (mut instance, rails) = fixture();
    let caller = component(&rails, "component.caller");
    let target = component(&rails, "component.target");
    let blocking = component(&rails, "component.blocking");
    let target_blocker = component(&rails, "component.target.blocker");
    let caller_participation = participate(&rails, caller.clone(), Health::Healthy);
    participate(&rails, target.clone(), Health::Healthy);
    let blocking_participation = participate(&rails, blocking.clone(), Health::Healthy);
    let target_blocker_participation = participate(&rails, target_blocker.clone(), Health::Healthy);
    rails
        .requirements
        .register(requirement(
            caller.clone(),
            blocking.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("register caller blocker");
    rails
        .requirements
        .register(requirement(
            target.clone(),
            target_blocker.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("register target blocker");
    let notes = notes_handle(
        &rails,
        caller.clone(),
        target.clone(),
        ComponentRequirementKind::Required,
    );

    rails
        .registry
        .update_health(&blocking_participation, Health::Unavailable)
        .expect("caller blocker unavailable");
    assert_eq!(
        notes.call(&caller_participation, |contract| contract.reply("blocked")),
        Err(ComponentError::ComponentUnavailable(
            caller.component_id().clone()
        ))
    );
    rails
        .registry
        .update_health(&blocking_participation, Health::Healthy)
        .expect("caller blocker recovered");
    rails
        .registry
        .update_health(&target_blocker_participation, Health::Unavailable)
        .expect("target blocker unavailable");
    assert_eq!(
        notes.call(&caller_participation, |contract| contract.reply("blocked")),
        Err(ComponentError::ComponentUnavailable(
            target.component_id().clone()
        ))
    );
    instance.stop();
}

#[test]
fn cross_instance_requirements_are_rejected() {
    let (mut instance, rails) = fixture();
    let local = component(&rails, "component.local");
    let foreign = foreign_component("component.requirement.foreign", "component.foreign");
    participate(&rails, local.clone(), Health::Healthy);

    assert!(matches!(
        rails.requirements.register(requirement(
            local.clone(),
            foreign.clone(),
            ComponentRequirementKind::Required,
        )),
        Err(ComponentError::ComponentRegistryInstanceMismatch { .. })
    ));
    assert!(matches!(
        rails.communication.bind(
            requirement(local, foreign, ComponentRequirementKind::Required),
            Arc::new(Notes)
        ),
        Err(ComponentError::ComponentRegistryInstanceMismatch { .. })
    ));
    instance.stop();
}

#[test]
fn malformed_requirement_cycles_fail_closed_without_recursing_forever() {
    let (mut instance, rails) = fixture();
    let a = component(&rails, "component.a");
    let b = component(&rails, "component.b");
    participate(&rails, a.clone(), Health::Healthy);
    participate(&rails, b.clone(), Health::Healthy);
    rails
        .requirements
        .register(requirement(
            a.clone(),
            b.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("register a->b");
    rails
        .requirements
        .register(requirement(
            b.clone(),
            a.clone(),
            ComponentRequirementKind::Required,
        ))
        .expect("register b->a");

    let availability = rails
        .requirements
        .effective_availability(a.component_id())
        .expect("availability");
    let blocker = provider_blocker(&availability);
    let nested = provider_blocker(blocker.provider_availability());
    assert_eq!(nested.provider(), &a);
    assert_eq!(
        nested.provider_availability(),
        &ComponentEffectiveAvailability::MalformedCycle {
            component: a.clone(),
        }
    );
    instance.stop();
}
