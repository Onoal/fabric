use std::sync::{Arc, Mutex};

use fabric::authoring::{CompositionExt, FabricBuilder};
use fabric::prelude::*;
use fabric_core::{
    ContractIdentity, Health, ModuleBindings, ModuleContract, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_test_system_operations::{
    OperationMarker, TestOperations, TestOperationsConfig, operations_contract_version,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct CapturedSystemResolution {
    provider: ModuleId,
    identity: ContractIdentity,
    marker: OperationMarker,
}

#[derive(Clone)]
struct SystemConsumer {
    module_id: ModuleId,
    requirement: SystemRequires<TestOperations>,
    capture: Arc<Mutex<Option<CapturedSystemResolution>>>,
}

impl SystemConsumer {
    fn new(capture: Arc<Mutex<Option<CapturedSystemResolution>>>) -> Self {
        Self {
            module_id: ModuleId::new("sdk.system.consumer").expect("module id"),
            requirement: SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            ),
            capture,
        }
    }
}

impl ModuleRuntime for SystemConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let resolved = self
            .requirement
            .resolve_with_provider(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.capture.lock().expect("capture lock") = Some(CapturedSystemResolution {
            provider: resolved.provider().clone(),
            identity: resolved.identity().clone(),
            marker: resolved.value().current_marker(),
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

#[test]
fn sdk_exposes_handwritten_system_authoring_without_a_second_runtime_model() {
    let capture = Arc::new(Mutex::new(None));
    let system = TestOperations::select(TestOperationsConfig::new(9, 4)).expect("system selection");
    let system_module_id = system.module_id().clone();

    let composition = FabricBuilder::new("fabric.test.sdk.system")
        .expect("builder")
        .block("runtime", |block| block.module(system))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(SystemConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer block")
        .build()
        .expect("composition");

    takes_raw_composition(&composition);
    let mut instance = composition
        .materialize_named("fabric.test.sdk.system.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(CapturedSystemResolution {
            provider: system_module_id,
            identity: ContractIdentity::versioned(operations_contract_version()),
            marker: OperationMarker::new(9),
        })
    );
}

fn takes_raw_composition(_composition: &fabric_core::Composition) {}
