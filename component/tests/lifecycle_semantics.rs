use std::sync::{Arc, Mutex};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractRequirement, Health,
    InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId, ModuleRuntime,
};

use fabric_component::{
    ComponentError, ComponentRuntime, ComponentRuntimeLifecycle, ComponentRuntimeModule,
    ComponentRuntimeStatus,
};

type CapturedContract = Arc<Mutex<Option<Arc<ComponentRuntime>>>>;
type CapturedStatuses = Arc<Mutex<Vec<ComponentRuntimeStatus>>>;

#[derive(Clone)]
struct ComponentRuntimeLifecycleProbeModule {
    module_id: ModuleId,
    runtime_requirement: ContractRequirement<ComponentRuntime>,
    captured_contract: CapturedContract,
    observed_statuses: CapturedStatuses,
    health: Health,
}

impl ComponentRuntimeLifecycleProbeModule {
    fn new(
        module_id: &str,
        captured_contract: CapturedContract,
        observed_statuses: CapturedStatuses,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            runtime_requirement: ContractRequirement::provisional(
                fabric_component::component_runtime_contract_id(),
            ),
            captured_contract,
            observed_statuses,
            health: Health::Degraded,
        }
    }
}

impl ModuleRuntime for ComponentRuntimeLifecycleProbeModule {
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
        vec![self.runtime_requirement.id().clone()]
            .into_iter()
            .map(fabric_core::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let contract = bindings
            .resolve(&self.runtime_requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self
            .captured_contract
            .lock()
            .expect("runtime contract capture") = Some(contract);
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        let contract = self
            .captured_contract
            .lock()
            .expect("runtime contract capture")
            .clone()
            .expect("runtime contract bound");
        self.observed_statuses
            .lock()
            .expect("observed statuses")
            .push(contract.current_status());
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        let contract = self
            .captured_contract
            .lock()
            .expect("runtime contract capture")
            .clone()
            .expect("runtime contract bound");
        self.observed_statuses
            .lock()
            .expect("observed statuses")
            .push(contract.current_status());
        self.health = Health::Healthy;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        self.health = Health::Unavailable;
        Ok(())
    }

    fn health(&self) -> Health {
        self.health
    }
}

#[test]
fn runtime_instance_identity_remains_stable_across_lifecycle_transitions() {
    let captured_contract = Arc::new(Mutex::new(None));
    let observed_statuses = Arc::new(Mutex::new(Vec::new()));

    let composition =
        CompositionBuilder::new(CompositionId::new("runtime.lifecycle").expect("composition"))
            .register_block(
                BlockBuilder::new(BlockId::new("runtime.lifecycle.block").expect("block"))
                    .register_module(ComponentRuntimeModule::new())
                    .register_module(ComponentRuntimeLifecycleProbeModule::new(
                        "runtime.lifecycle.probe",
                        Arc::clone(&captured_contract),
                        Arc::clone(&observed_statuses),
                    ))
                    .build(),
            )
            .build()
            .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("runtime.alpha").expect("instance id"))
        .expect("materialize instance");

    let contract = captured_contract
        .lock()
        .expect("runtime contract capture")
        .clone()
        .expect("captured contract");
    let built_status = contract.current_status();
    assert_eq!(built_status.instance_id().as_str(), "runtime.alpha");
    assert_eq!(built_status.lifecycle(), ComponentRuntimeLifecycle::Stopped);
    assert_eq!(built_status.health(), Health::Unavailable);

    instance.start().expect("start instance");

    let observed = observed_statuses.lock().expect("observed statuses").clone();
    assert_eq!(observed.len(), 2);
    assert_eq!(observed[0].lifecycle(), ComponentRuntimeLifecycle::Starting);
    assert_eq!(observed[0].health(), Health::Degraded);
    assert_eq!(observed[0].instance_id().as_str(), "runtime.alpha");
    assert_eq!(observed[1].lifecycle(), ComponentRuntimeLifecycle::Ready);
    assert_eq!(observed[1].health(), Health::Healthy);

    let running_status = contract.current_status();
    assert_eq!(running_status.lifecycle(), ComponentRuntimeLifecycle::Ready);
    assert_eq!(running_status.health(), Health::Healthy);

    instance.stop().expect("stop instance");

    let stopped_status = contract.current_status();
    assert_eq!(
        stopped_status.lifecycle(),
        ComponentRuntimeLifecycle::Stopped
    );
    assert_eq!(stopped_status.health(), Health::Unavailable);
    assert_eq!(stopped_status.instance_id().as_str(), "runtime.alpha");
    assert_eq!(
        contract
            .current_instance_id()
            .expect_err("runtime unavailable"),
        ComponentError::Unavailable
    );
}

#[test]
fn runtime_lifecycle_invalid_transition_fails_deterministically() {
    let error = ComponentRuntimeLifecycle::Stopped
        .transition_to(ComponentRuntimeLifecycle::Ready)
        .expect_err("stopped cannot transition directly to ready");

    assert!(matches!(
        error,
        ComponentError::InvalidComponentRuntimeLifecycleTransition {
            from: ComponentRuntimeLifecycle::Stopped,
            to: ComponentRuntimeLifecycle::Ready,
        }
    ));
}

#[test]
fn runtime_lifecycle_truth_is_shared_across_multiple_consumers() {
    let first_contract = Arc::new(Mutex::new(None));
    let first_observed = Arc::new(Mutex::new(Vec::new()));
    let second_contract = Arc::new(Mutex::new(None));
    let second_observed = Arc::new(Mutex::new(Vec::new()));

    let composition =
        CompositionBuilder::new(CompositionId::new("runtime.shared.truth").expect("composition"))
            .register_block(
                BlockBuilder::new(BlockId::new("runtime.shared.truth.block").expect("block"))
                    .register_module(ComponentRuntimeModule::new())
                    .register_module(ComponentRuntimeLifecycleProbeModule::new(
                        "runtime.shared.probe.a",
                        Arc::clone(&first_contract),
                        Arc::clone(&first_observed),
                    ))
                    .register_module(ComponentRuntimeLifecycleProbeModule::new(
                        "runtime.shared.probe.b",
                        Arc::clone(&second_contract),
                        Arc::clone(&second_observed),
                    ))
                    .build(),
            )
            .build()
            .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("runtime.shared").expect("instance id"))
        .expect("materialize instance");

    let first = first_contract
        .lock()
        .expect("first contract capture")
        .clone()
        .expect("first contract");
    let second = second_contract
        .lock()
        .expect("second contract capture")
        .clone()
        .expect("second contract");
    assert_eq!(first.current_status(), second.current_status());

    instance.start().expect("start instance");
    assert_eq!(first.current_status(), second.current_status());

    instance.stop().expect("stop instance");
    assert_eq!(first.current_status(), second.current_status());
}

#[test]
fn valid_runtime_lifecycle_progression_is_explicit() {
    let lifecycle = ComponentRuntimeLifecycle::Stopped
        .transition_to(ComponentRuntimeLifecycle::Starting)
        .expect("stopped -> starting");
    let lifecycle = lifecycle
        .transition_to(ComponentRuntimeLifecycle::Ready)
        .expect("starting -> ready");
    let lifecycle = lifecycle
        .transition_to(ComponentRuntimeLifecycle::Stopping)
        .expect("ready -> stopping");
    let lifecycle = lifecycle
        .transition_to(ComponentRuntimeLifecycle::Stopped)
        .expect("stopping -> stopped");

    assert_eq!(lifecycle, ComponentRuntimeLifecycle::Stopped);
}
