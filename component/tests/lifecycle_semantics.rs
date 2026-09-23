use std::sync::{Arc, Mutex};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractRequirement, Health,
    InstanceError, InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId,
    ModuleRuntime,
};

use fabric_component::{
    ComponentError, ComponentHost, ComponentHostLifecycle, ComponentHostModule, ComponentHostStatus,
};

type CapturedContract = Arc<Mutex<Option<Arc<ComponentHost>>>>;
type CapturedStatuses = Arc<Mutex<Vec<ComponentHostStatus>>>;

#[derive(Clone)]
struct ComponentRuntimeLifecycleProbeModule {
    module_id: ModuleId,
    runtime_requirement: ContractRequirement<ComponentHost>,
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
                fabric_component::component_host_contract_id(),
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

#[derive(Clone, Copy)]
enum LifecycleFailurePhase {
    Initialize,
    Start,
}

#[derive(Clone)]
struct LifecycleFailureModule {
    module_id: ModuleId,
    phase: LifecycleFailurePhase,
    runtime_requirement: Option<ContractRequirement<ComponentHost>>,
    captured_contract: CapturedContract,
    observed_statuses: CapturedStatuses,
}

impl LifecycleFailureModule {
    fn after_component_initialize(
        captured_contract: CapturedContract,
        observed_statuses: CapturedStatuses,
    ) -> Self {
        Self {
            module_id: ModuleId::new("runtime.lifecycle.initialize-failure").expect("module id"),
            phase: LifecycleFailurePhase::Initialize,
            runtime_requirement: Some(ContractRequirement::provisional(
                fabric_component::component_host_contract_id(),
            )),
            captured_contract,
            observed_statuses,
        }
    }

    fn before_component_start() -> Self {
        Self {
            module_id: ModuleId::new("runtime.lifecycle.start-failure").expect("module id"),
            phase: LifecycleFailurePhase::Start,
            runtime_requirement: None,
            captured_contract: Arc::new(Mutex::new(None)),
            observed_statuses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn observe_component_status(&self) {
        let Some(contract) = self
            .captured_contract
            .lock()
            .expect("runtime contract capture")
            .clone()
        else {
            return;
        };
        self.observed_statuses
            .lock()
            .expect("observed statuses")
            .push(contract.current_status());
    }
}

impl ModuleRuntime for LifecycleFailureModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        self.runtime_requirement
            .iter()
            .map(|requirement| {
                fabric_core::ContractRequirementDeclaration::provisional(requirement.id().clone())
            })
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let Some(requirement) = &self.runtime_requirement else {
            return Ok(());
        };
        let contract = bindings
            .resolve(requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self
            .captured_contract
            .lock()
            .expect("runtime contract capture") = Some(contract);
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.observe_component_status();
        match self.phase {
            LifecycleFailurePhase::Initialize => Err(ModuleError::new("forced initialize failure")),
            LifecycleFailurePhase::Start => Ok(()),
        }
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        match self.phase {
            LifecycleFailurePhase::Initialize => Ok(()),
            LifecycleFailurePhase::Start => Err(ModuleError::new("forced start failure")),
        }
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
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
                    .register_module(ComponentHostModule::new())
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
    assert_eq!(built_status.lifecycle(), ComponentHostLifecycle::Stopped);
    assert_eq!(built_status.health(), Health::Unavailable);

    instance.start().expect("start instance");

    let observed = observed_statuses.lock().expect("observed statuses").clone();
    assert_eq!(observed.len(), 2);
    assert_eq!(observed[0].lifecycle(), ComponentHostLifecycle::Starting);
    assert_eq!(observed[0].health(), Health::Degraded);
    assert_eq!(observed[0].instance_id().as_str(), "runtime.alpha");
    assert_eq!(observed[1].lifecycle(), ComponentHostLifecycle::Ready);
    assert_eq!(observed[1].health(), Health::Healthy);

    let running_status = contract.current_status();
    assert_eq!(running_status.lifecycle(), ComponentHostLifecycle::Ready);
    assert_eq!(running_status.health(), Health::Healthy);

    instance.stop().expect("stop instance");
    instance.stop().expect("stopped instance is idempotent");

    let stopped_status = contract.current_status();
    assert_eq!(stopped_status.lifecycle(), ComponentHostLifecycle::Stopped);
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
    let error = ComponentHostLifecycle::Stopped
        .transition_to(ComponentHostLifecycle::Ready)
        .expect_err("stopped cannot transition directly to ready");

    assert!(matches!(
        error,
        ComponentError::InvalidComponentHostLifecycleTransition {
            from: ComponentHostLifecycle::Stopped,
            to: ComponentHostLifecycle::Ready,
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
                    .register_module(ComponentHostModule::new())
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
    let lifecycle = ComponentHostLifecycle::Stopped
        .transition_to(ComponentHostLifecycle::Starting)
        .expect("stopped -> starting");
    let lifecycle = lifecycle
        .transition_to(ComponentHostLifecycle::Ready)
        .expect("starting -> ready");
    let lifecycle = lifecycle
        .transition_to(ComponentHostLifecycle::Stopping)
        .expect("ready -> stopping");
    let lifecycle = lifecycle
        .transition_to(ComponentHostLifecycle::Stopped)
        .expect("stopping -> stopped");

    assert_eq!(lifecycle, ComponentHostLifecycle::Stopped);
}

#[test]
fn abandoned_startup_uses_the_same_stopping_cleanup_path() {
    let lifecycle = ComponentHostLifecycle::Stopped
        .transition_to(ComponentHostLifecycle::Starting)
        .expect("stopped -> starting");
    let lifecycle = lifecycle
        .transition_to(ComponentHostLifecycle::Stopping)
        .expect("starting -> stopping");
    let lifecycle = lifecycle
        .transition_to(ComponentHostLifecycle::Stopped)
        .expect("stopping -> stopped");

    assert_eq!(lifecycle, ComponentHostLifecycle::Stopped);
    assert!(
        ComponentHostLifecycle::Starting
            .transition_to(ComponentHostLifecycle::Stopped)
            .is_err()
    );
}

#[test]
fn initialize_failure_cleans_a_starting_component_host_without_cleanup_error() {
    let captured_contract = Arc::new(Mutex::new(None));
    let observed_statuses = Arc::new(Mutex::new(Vec::new()));
    let composition = CompositionBuilder::new(
        CompositionId::new("runtime.lifecycle.initialize-abandonment").expect("composition"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("runtime.lifecycle.block").expect("block"))
            .register_module(ComponentHostModule::new())
            .register_module(LifecycleFailureModule::after_component_initialize(
                Arc::clone(&captured_contract),
                Arc::clone(&observed_statuses),
            ))
            .build(),
    )
    .build()
    .expect("composition");
    let instance_id = InstanceId::new("runtime.lifecycle.initialize-abandonment").expect("id");
    let mut instance = composition
        .materialize(instance_id.clone())
        .expect("materialize instance");

    let error = instance.start().expect_err("initialize must fail");
    assert!(matches!(
        error,
        InstanceError::ModuleFailure {
            ref module_id,
            phase: "initialize",
            cleanup: None,
            ..
        } if module_id.as_str() == "runtime.lifecycle.initialize-failure"
    ));
    assert_eq!(instance.lifecycle(), fabric_core::LifecycleState::Stopped);

    let observed = observed_statuses.lock().expect("observed statuses").clone();
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].instance_id(), &instance_id);
    assert_eq!(observed[0].lifecycle(), ComponentHostLifecycle::Starting);
    assert_eq!(observed[0].health(), Health::Degraded);

    let runtime = captured_contract
        .lock()
        .expect("runtime contract capture")
        .clone()
        .expect("captured contract");
    assert_eq!(
        runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Stopped
    );
    assert_eq!(runtime.current_status().health(), Health::Unavailable);
}

#[test]
fn start_failure_before_component_host_start_cleans_the_starting_host_without_cleanup_error() {
    let captured_contract = Arc::new(Mutex::new(None));
    let observed_statuses = Arc::new(Mutex::new(Vec::new()));
    let composition = CompositionBuilder::new(
        CompositionId::new("runtime.lifecycle.start-abandonment").expect("composition"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("runtime.lifecycle.block").expect("block"))
            .register_module(LifecycleFailureModule::before_component_start())
            .register_module(ComponentHostModule::new())
            .register_module(ComponentRuntimeLifecycleProbeModule::new(
                "runtime.lifecycle.start-observer",
                Arc::clone(&captured_contract),
                Arc::clone(&observed_statuses),
            ))
            .build(),
    )
    .build()
    .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("runtime.lifecycle.start-abandonment").expect("id"))
        .expect("materialize instance");

    let error = instance.start().expect_err("start must fail");
    assert!(matches!(
        error,
        InstanceError::ModuleFailure {
            ref module_id,
            phase: "start",
            cleanup: None,
            ..
        } if module_id.as_str() == "runtime.lifecycle.start-failure"
    ));
    assert_eq!(instance.lifecycle(), fabric_core::LifecycleState::Stopped);

    let observed = observed_statuses.lock().expect("observed statuses").clone();
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].lifecycle(), ComponentHostLifecycle::Starting);
    assert_eq!(observed[0].health(), Health::Degraded);

    let runtime = captured_contract
        .lock()
        .expect("runtime contract capture")
        .clone()
        .expect("captured contract");
    assert_eq!(
        runtime.current_status().lifecycle(),
        ComponentHostLifecycle::Stopped
    );
    assert_eq!(runtime.current_status().health(), Health::Unavailable);
}
