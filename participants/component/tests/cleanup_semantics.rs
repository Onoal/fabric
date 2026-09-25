use std::sync::{Arc, Mutex};

use fabric_component::{
    ComponentDeclaration, ComponentError, ComponentHost, ComponentHostLifecycle,
    ComponentHostModule, ComponentId, ComponentMaterializer, ComponentParticipationPreparation,
    ComponentParticipationRealization, ComponentReadinessPolicy,
};
use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractRequirement, Health,
    InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId, ModuleRuntime,
};

type Captured = Arc<Mutex<Option<(Arc<ComponentMaterializer>, Arc<ComponentHost>)>>>;

#[derive(Clone)]
struct CaptureModule {
    id: ModuleId,
    materializer: ContractRequirement<ComponentMaterializer>,
    runtime: ContractRequirement<ComponentHost>,
    captured: Captured,
}

impl CaptureModule {
    fn new(captured: Captured) -> Self {
        Self {
            id: ModuleId::new("component.cleanup.capture").expect("module id"),
            materializer: ContractRequirement::provisional(
                fabric_component::component_materializer_contract_id(),
            ),
            runtime: ContractRequirement::provisional(
                fabric_component::component_host_contract_id(),
            ),
            captured,
        }
    }
}

impl ModuleRuntime for CaptureModule {
    fn id(&self) -> &ModuleId {
        &self.id
    }
    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        Vec::new()
    }
    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![
            fabric_core::ContractRequirementDeclaration::provisional(
                self.materializer.id().clone(),
            ),
            fabric_core::ContractRequirementDeclaration::provisional(self.runtime.id().clone()),
        ]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }
    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        *self.captured.lock().expect("capture") = Some((
            bindings
                .resolve(&self.materializer)
                .map_err(|error| ModuleError::new(error.to_string()))?,
            bindings
                .resolve(&self.runtime)
                .map_err(|error| ModuleError::new(error.to_string()))?,
        ));
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

fn compose(host: ComponentHostModule, captured: Captured) -> fabric_core::Instance {
    let composition = CompositionBuilder::new(CompositionId::new("component.cleanup").expect("id"))
        .register_block(
            BlockBuilder::new(BlockId::new("component.cleanup.block").expect("block"))
                .register_module(host)
                .register_module(CaptureModule::new(captured))
                .build(),
        )
        .build()
        .expect("composition");
    composition
        .materialize(InstanceId::new("component.cleanup.instance").expect("instance"))
        .expect("materialize")
}

fn captured_contracts(captured: &Captured) -> (Arc<ComponentMaterializer>, Arc<ComponentHost>) {
    captured.lock().expect("capture").clone().expect("bound")
}

fn declaration(id: &ComponentId) -> ComponentDeclaration {
    ComponentDeclaration::new(id.clone(), Vec::new())
}

#[test]
fn teardown_reverses_base_and_augmentation_preparation_and_is_consumed_once() {
    let component_id = ComponentId::new("component.cleanup.reverse").expect("component");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let base_calls = Arc::clone(&calls);
    let base =
        ComponentParticipationRealization::new_with_teardown(component_id.clone(), move |_| {
            let calls = Arc::clone(&base_calls);
            Ok(ComponentParticipationPreparation::with_teardown(
                Health::Healthy,
                move || {
                    calls.lock().expect("calls").push("base");
                    Ok(())
                },
            ))
        });
    let augmentation = |name: &'static str| {
        let calls = Arc::clone(&calls);
        fabric_component::ComponentAugmentationParticipationRealization::new_with_teardown(
            component_id.clone(),
            move |_| {
                let calls = Arc::clone(&calls);
                Ok(
                    fabric_component::ComponentAugmentationParticipationPreparation::with_teardown(
                        move || {
                            calls.lock().expect("calls").push(name);
                            Ok(())
                        },
                    ),
                )
            },
        )
    };
    let captured = Arc::new(Mutex::new(None));
    let mut instance = compose(
        ComponentHostModule::with_components_and_augmentations(
            vec![declaration(&component_id)],
            vec![base],
            vec![
                augmentation("augmentation-a"),
                augmentation("augmentation-b"),
            ],
        )
        .expect("host"),
        Arc::clone(&captured),
    );
    instance.start().expect("start");
    let (materializer, _) = captured_contracts(&captured);
    materializer
        .materialize(&component_id)
        .expect("materialize component");
    materializer
        .dematerialize(&component_id)
        .expect("dematerialize component");
    assert_eq!(
        calls.lock().expect("calls").as_slice(),
        ["augmentation-b", "augmentation-a", "base"]
    );
    assert!(materializer.dematerialize(&component_id).is_err());
    assert_eq!(calls.lock().expect("calls").len(), 3);
}

#[test]
fn failed_augmentation_rolls_back_successful_contributions_and_host_stop_cleans_current_ones() {
    let component_id = ComponentId::new("component.cleanup.rollback").expect("component");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let base_calls = Arc::clone(&calls);
    let base =
        ComponentParticipationRealization::new_with_teardown(component_id.clone(), move |_| {
            let calls = Arc::clone(&base_calls);
            Ok(ComponentParticipationPreparation::with_teardown(
                Health::Healthy,
                move || {
                    calls.lock().expect("calls").push("base");
                    Ok(())
                },
            ))
        });
    let a_calls = Arc::clone(&calls);
    let augmentation_a =
        fabric_component::ComponentAugmentationParticipationRealization::new_with_teardown(
            component_id.clone(),
            move |_| {
                let calls = Arc::clone(&a_calls);
                Ok(
                    fabric_component::ComponentAugmentationParticipationPreparation::with_teardown(
                        move || {
                            calls.lock().expect("calls").push("augmentation-a");
                            Ok(())
                        },
                    ),
                )
            },
        );
    let failing = fabric_component::ComponentAugmentationParticipationRealization::new(
        component_id.clone(),
        |_| Err(ComponentError::Unavailable),
    );
    let captured = Arc::new(Mutex::new(None));
    let mut instance = compose(
        ComponentHostModule::with_components_and_augmentations(
            vec![declaration(&component_id)],
            vec![base],
            vec![augmentation_a, failing],
        )
        .expect("host"),
        Arc::clone(&captured),
    );
    instance.start().expect("start");
    let (materializer, _) = captured_contracts(&captured);
    assert!(materializer.materialize(&component_id).is_err());
    assert_eq!(
        calls.lock().expect("calls").as_slice(),
        ["augmentation-a", "base"]
    );

    let component_id = ComponentId::new("component.cleanup.stop").expect("component");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let calls_for_base = Arc::clone(&calls);
    let base =
        ComponentParticipationRealization::new_with_teardown(component_id.clone(), move |_| {
            let calls = Arc::clone(&calls_for_base);
            Ok(ComponentParticipationPreparation::with_teardown(
                Health::Healthy,
                move || {
                    calls.lock().expect("calls").push("stopped");
                    Ok(())
                },
            ))
        });
    let captured = Arc::new(Mutex::new(None));
    let mut instance = compose(
        ComponentHostModule::with_components(vec![declaration(&component_id)], vec![base])
            .expect("host"),
        Arc::clone(&captured),
    );
    instance.start().expect("start");
    let (materializer, _) = captured_contracts(&captured);
    materializer
        .materialize(&component_id)
        .expect("materialize");
    instance.stop().expect("host stop");
    assert_eq!(calls.lock().expect("calls").as_slice(), ["stopped"]);
}

#[test]
fn host_lifecycle_stays_ready_when_readiness_health_is_unavailable() {
    let required = ComponentId::new("component.cleanup.required").expect("component");
    let captured = Arc::new(Mutex::new(None));
    let mut instance = compose(
        ComponentHostModule::with_readiness_policy(
            ComponentReadinessPolicy::new(vec![required]).expect("readiness policy"),
        ),
        Arc::clone(&captured),
    );
    instance.start().expect("start");
    let (_, runtime) = captured_contracts(&captured);
    let status = runtime.current_status();
    assert_eq!(status.lifecycle(), ComponentHostLifecycle::Ready);
    assert_eq!(status.health(), Health::Unavailable);
}

#[test]
fn teardown_failures_are_aggregated_and_host_stop_surfaces_them_to_core() {
    let component_id = ComponentId::new("component.cleanup.failures").expect("component");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let base_calls = Arc::clone(&calls);
    let base =
        ComponentParticipationRealization::new_with_teardown(component_id.clone(), move |_| {
            let calls = Arc::clone(&base_calls);
            Ok(ComponentParticipationPreparation::with_teardown(
                Health::Healthy,
                move || {
                    calls.lock().expect("calls").push("base");
                    Err(ComponentError::Unavailable)
                },
            ))
        });
    let augmentation_calls = Arc::clone(&calls);
    let augmentation =
        fabric_component::ComponentAugmentationParticipationRealization::new_with_teardown(
            component_id.clone(),
            move |_| {
                let calls = Arc::clone(&augmentation_calls);
                Ok(
                    fabric_component::ComponentAugmentationParticipationPreparation::with_teardown(
                        move || {
                            calls.lock().expect("calls").push("augmentation");
                            Err(ComponentError::Unavailable)
                        },
                    ),
                )
            },
        );
    let captured = Arc::new(Mutex::new(None));
    let mut instance = compose(
        ComponentHostModule::with_components_and_augmentations(
            vec![declaration(&component_id)],
            vec![base],
            vec![augmentation],
        )
        .expect("host"),
        Arc::clone(&captured),
    );
    instance.start().expect("start");
    let (materializer, _) = captured_contracts(&captured);
    materializer
        .materialize(&component_id)
        .expect("materialize");
    let error = materializer
        .dematerialize(&component_id)
        .expect_err("teardown failures are observable");
    match error {
        ComponentError::ComponentParticipationCleanupFailed(cleanup) => {
            assert_eq!(cleanup.failures().len(), 2);
        }
        other => panic!("unexpected teardown error: {other}"),
    }
    assert_eq!(
        calls.lock().expect("calls").as_slice(),
        ["augmentation", "base"]
    );

    // Fresh materialization proves terminal teardown ownership was consumed.
    materializer
        .materialize(&component_id)
        .expect("rematerialize");
    assert!(
        instance.stop().is_err(),
        "Core observes host cleanup failure"
    );
    assert_eq!(instance.lifecycle(), fabric_core::LifecycleState::Stopped);
}
