use std::sync::{Arc, Mutex};

use fabric::authoring::{BlockAuthor, ComponentAugmentationDefinition};
use fabric::component::operator::{
    ComponentControlRail, ComponentReconstructionRail, ComponentReconstructionResult,
    ComponentReconstructionRuntimeState, ComponentRegistry,
};
use fabric::component::participation::ComponentInstanceBinding;
use fabric::core::{
    ContractId, ContractKey, ContractRequirement, Health, Module, ModuleBindings, ModuleContract,
    ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
};
use fabric::prelude::*;

fn host() -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("architecture"),
    )
}

fabric::component! {
    IntentSelf {
        id: "fabric.test.intent.self";
        api { fn run(&self); }
        runtime {
            prepare { Ok(()) }
            fn run(&self) {}
        }
    }
}

fabric::component! {
    IntentAdapterTarget {
        id: "fabric.test.intent.adapter";
        api { fn run(&self); }
    }
}

fabric::adapter! {
    IntentAdapter for IntentAdapterTarget {
        id: "fabric.test.intent.adapter-realization";
        runtime { fn run(&self) {} }
    }
}

fabric::component! {
    IntentDeclarationOnly {
        id: "fabric.test.intent.declaration-only";
        api { fn run(&self); }
    }
}

fabric::resource! {
    IntentOnlyResource {
        id: "fabric.test.intent.resource-only";
        api { fn read(&self) -> u64; }
        runtime { fn read(&self) -> u64 { 1 } }
    }
}

struct IntentAugmentation;

#[derive(Clone)]
struct IntentAugmentationContract;

impl ComponentAugmentationDefinition<IntentSelf> for IntentAugmentation {
    type Config = ();
    type Contract = IntentAugmentationContract;

    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.intent.component-augmentation").expect("contract"),
        )
    }
}

#[derive(Clone)]
struct CapturedRails {
    control: Arc<ComponentControlRail>,
    reconstruction: Arc<ComponentReconstructionRail>,
    registry: Arc<ComponentRegistry>,
}

struct CaptureModule {
    module_id: ModuleId,
    capture: Arc<Mutex<Option<CapturedRails>>>,
}

impl CaptureModule {
    fn new(id: &'static str, capture: Arc<Mutex<Option<CapturedRails>>>) -> Self {
        Self {
            module_id: ModuleId::new(id).expect("capture module id"),
            capture,
        }
    }
}

impl Module for CaptureModule {
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::new(self.module_id.clone()).with_required_contracts(vec![
            ContractRequirement::<ComponentControlRail>::provisional(
                fabric::component::component_control_contract_id(),
            )
            .declaration()
            .clone(),
            ContractRequirement::<ComponentReconstructionRail>::provisional(
                fabric::component::component_reconstruction_contract_id(),
            )
            .declaration()
            .clone(),
            ContractRequirement::<ComponentRegistry>::provisional(
                fabric::component::component_registry_contract_id(),
            )
            .declaration()
            .clone(),
        ])
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(CaptureRuntime {
            module_id: self.module_id.clone(),
            capture: Arc::clone(&self.capture),
        }))
    }
}

struct CaptureRuntime {
    module_id: ModuleId,
    capture: Arc<Mutex<Option<CapturedRails>>>,
}

impl ModuleRuntime for CaptureRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric::core::ContractRequirementDeclaration> {
        CaptureModule::new(
            "fabric.test.intent.capture.declaration",
            Arc::new(Mutex::new(None)),
        )
        .declaration()
        .required_contracts()
        .to_vec()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let control = bindings
            .resolve(&ContractRequirement::<ComponentControlRail>::provisional(
                fabric::component::component_control_contract_id(),
            ))
            .map_err(|error| ModuleError::new(error.to_string()))?;
        let reconstruction = bindings
            .resolve(
                &ContractRequirement::<ComponentReconstructionRail>::provisional(
                    fabric::component::component_reconstruction_contract_id(),
                ),
            )
            .map_err(|error| ModuleError::new(error.to_string()))?;
        let registry = bindings
            .resolve(&ContractRequirement::<ComponentRegistry>::provisional(
                fabric::component::component_registry_contract_id(),
            ))
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.capture.lock().expect("capture lock") = Some(CapturedRails {
            control,
            reconstruction,
            registry,
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

fn component_id(value: &'static str) -> ComponentId {
    ComponentId::new(value).expect("component id")
}

fn control_for(rails: &CapturedRails, id: &'static str) -> fabric::component::ComponentControl {
    rails.control.control(&component_id(id)).expect("control")
}

fn captured(capture: &Arc<Mutex<Option<CapturedRails>>>) -> CapturedRails {
    capture
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured rails")
}

fn composition_with_capture(
    composition_id: &'static str,
    capture_id: &'static str,
    capture: Arc<Mutex<Option<CapturedRails>>>,
) -> Composition {
    Fabric::new(composition_id)
        .expect("fabric")
        .component(IntentSelf::define())
        .component(
            IntentAdapterTarget::define()
                .using(IntentAdapter::new())
                .expect("adapter")
                .initially_disabled(),
        )
        .component(IntentDeclarationOnly::define())
        .with_block(
            BlockAuthor::new(format!("{capture_id}.block"))
                .expect("capture block")
                .module(CaptureModule::new(capture_id, capture))
                .build(),
        )
        .build()
        .expect("composition")
}

#[test]
fn component_initial_intent_defaults_override_and_inspection_are_composition_truth() {
    let capture = Arc::new(Mutex::new(None));
    let composition = composition_with_capture(
        "fabric.test.intent.composition",
        "fabric.test.intent.capture.inspect",
        capture,
    );

    let components = composition.components().collect::<Vec<_>>();
    assert_eq!(components.len(), 3);
    assert_eq!(
        components
            .iter()
            .find(|component| component.component_id().as_str() == "fabric.test.intent.self")
            .expect("self")
            .initial_participation(),
        ComponentDesiredState::Enabled
    );
    assert_eq!(
        components
            .iter()
            .find(|component| component.component_id().as_str() == "fabric.test.intent.adapter")
            .expect("adapter")
            .initial_participation(),
        ComponentDesiredState::Disabled
    );
    assert_eq!(
        components
            .iter()
            .find(|component| {
                component.component_id().as_str() == "fabric.test.intent.declaration-only"
            })
            .expect("declaration-only")
            .initial_participation(),
        ComponentDesiredState::Disabled
    );
}

#[test]
fn materialization_seeds_desired_controls_without_observed_participation() {
    let capture = Arc::new(Mutex::new(None));
    let composition = composition_with_capture(
        "fabric.test.intent.materialize",
        "fabric.test.intent.capture.materialize",
        Arc::clone(&capture),
    );

    let instance = composition
        .materialize_on("fabric.test.intent.materialize.instance", &host())
        .expect("materialize");
    let rails = captured(&capture);

    let self_control = control_for(&rails, "fabric.test.intent.self");
    assert_eq!(self_control.desired(), ComponentDesiredState::Enabled);
    assert_eq!(
        self_control.component().instance_id(),
        instance.instance_id()
    );
    assert_eq!(
        rails
            .registry
            .component(&component_id("fabric.test.intent.self")),
        Err(fabric::component::ComponentError::UnknownComponent(
            component_id("fabric.test.intent.self")
        ))
    );
    assert_eq!(rails.registry.components(), Vec::new());
}

#[test]
fn reconstruction_observes_seeded_intent_after_start_without_start_reconciling() {
    let capture = Arc::new(Mutex::new(None));
    let composition = composition_with_capture(
        "fabric.test.intent.reconstruct",
        "fabric.test.intent.capture.reconstruct",
        Arc::clone(&capture),
    );

    let mut instance = composition
        .materialize_on("fabric.test.intent.reconstruct.instance", &host())
        .expect("materialize");
    instance.start().expect("start");
    let rails = captured(&capture);
    assert!(rails.registry.components().is_empty());

    let report = rails.reconstruction.reconstruct().expect("reconstruct");
    let self_outcome = report
        .outcomes()
        .iter()
        .find(|outcome| outcome.component_id().as_str() == "fabric.test.intent.self")
        .expect("self outcome");
    assert_eq!(self_outcome.desired(), ComponentDesiredState::Enabled);
    assert_eq!(
        self_outcome.observed_runtime(),
        &ComponentReconstructionRuntimeState::Absent
    );
    assert!(matches!(
        self_outcome.result(),
        ComponentReconstructionResult::Materialized(_)
    ));

    let disabled_outcome = report
        .outcomes()
        .iter()
        .find(|outcome| outcome.component_id().as_str() == "fabric.test.intent.adapter")
        .expect("disabled outcome");
    assert_eq!(disabled_outcome.desired(), ComponentDesiredState::Disabled);
    assert_eq!(
        disabled_outcome.result(),
        &ComponentReconstructionResult::AlreadyConverged
    );
}

#[test]
fn multi_instance_controls_are_fresh_and_runtime_mutation_does_not_leak() {
    let capture_a = Arc::new(Mutex::new(None));
    let composition = composition_with_capture(
        "fabric.test.intent.multi",
        "fabric.test.intent.capture.multi",
        Arc::clone(&capture_a),
    );

    let mut instance_a = composition
        .materialize_on("fabric.test.intent.multi.a", &host())
        .expect("materialize a");
    let rails_a = captured(&capture_a);
    rails_a
        .control
        .disable(ComponentInstanceBinding::for_instance(
            component_id("fabric.test.intent.self"),
            instance_a.instance_id().clone(),
        ))
        .expect("disable a");
    assert_eq!(
        control_for(&rails_a, "fabric.test.intent.self").desired(),
        ComponentDesiredState::Disabled
    );
    instance_a.stop().expect("stop a");
    assert_eq!(instance_a.lifecycle(), LifecycleState::Stopped);

    *capture_a.lock().expect("capture lock") = None;
    let instance_b = composition
        .materialize_on("fabric.test.intent.multi.b", &host())
        .expect("materialize b");
    let rails_b = captured(&capture_a);

    assert_ne!(instance_a.instance_id(), instance_b.instance_id());
    assert_ne!(instance_a.generation(), instance_b.generation());
    assert_eq!(
        control_for(&rails_b, "fabric.test.intent.self").desired(),
        ComponentDesiredState::Enabled
    );
    assert_eq!(
        composition
            .components()
            .find(|component| component.component_id().as_str() == "fabric.test.intent.self")
            .expect("composition self")
            .initial_participation(),
        ComponentDesiredState::Enabled
    );
}

#[test]
fn contributions_preserve_initial_intent_and_augmentation_is_component_level() {
    fn contributed_component() -> impl IntoFabricContribution {
        FabricContribution::new().component(
            IntentSelf::define()
                .augment::<IntentAugmentation>(())
                .expect("augmentation"),
        )
    }

    let inline = Fabric::new("fabric.test.intent.inline")
        .expect("fabric")
        .component(
            IntentSelf::define()
                .augment::<IntentAugmentation>(())
                .expect("augmentation"),
        )
        .build()
        .expect("inline");
    let contributed = Fabric::new("fabric.test.intent.contributed")
        .expect("fabric")
        .with(contributed_component())
        .build()
        .expect("contributed");

    let inline_component = inline.components().next().expect("inline component");
    let contributed_component = contributed
        .components()
        .next()
        .expect("contributed component");
    assert_eq!(
        inline_component.initial_participation(),
        contributed_component.initial_participation()
    );
    assert_eq!(contributed_component.augmentations().count(), 1);
}

#[test]
fn resource_system_only_composition_has_no_component_controls() {
    let composition = Fabric::new("fabric.test.intent.resource-system-only")
        .expect("fabric")
        .resource(IntentOnlyResource::select("primary").expect("resource"))
        .build()
        .expect("build");
    let instance = composition
        .materialize("fabric.test.intent.resource-system-only.instance")
        .expect("materialize");

    assert_eq!(composition.components().count(), 0);
    assert!(instance.components().is_none());
}
