use std::sync::{Arc, Mutex};

use fabric::{
    authoring::{CompositionExt, FabricBuilder},
    core::ContractProviderSelection,
    ids::module,
    prelude::*,
};
use fabric_core::{
    CompositionError, ContractIdentity, Health, ModuleBindings, ModuleContract, ModuleError,
    ModuleId, ModuleRuntime,
};
use fabric_test_resource_counter::{AdaptedCounter, AdaptedCounterConfig};
use fabric_test_system_operations::{
    AdaptedOperations, AdaptedOperationsConfig, OperationMarker, TestOperations,
    TestOperationsConfig, adapted_operations_contract_version,
};

use crate::{
    ExternalCounterAdapter, ExternalCounterAdapterConfig, ExternalOperationsAdapter,
    ExternalOperationsAdapterConfig, ExternalOperationsWithSystemAdapter,
    ExternalOperationsWithSystemAdapterConfig, HostBoundExternalCounterAdapter,
    HostBoundExternalCounterAdapterConfig, external_counter_host_facility,
};

fn test_host() -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("arch"),
    )
}

#[test]
fn external_targets_expose_public_typed_realization_interfaces() {
    fn resource_interface<T: fabric_test_resource_counter::AdaptedCounterRealization>() {}
    fn system_interface<T: fabric_test_system_operations::AdaptedOperationsRealization>() {}

    let _ = resource_interface::<crate::resource_adapter::external_counter_adapter::raw::Runtime>;
    let _ = system_interface::<crate::system_adapter::external_operations_adapter::raw::Runtime>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CounterCapture {
    value: u64,
    provider: String,
    identity: String,
}

#[derive(Clone)]
struct CounterConsumer {
    module_id: ModuleId,
    requirement: fabric::Requires<AdaptedCounter>,
    capture: Arc<Mutex<Option<CounterCapture>>>,
}

impl CounterConsumer {
    fn new(capture: Arc<Mutex<Option<CounterCapture>>>) -> Self {
        Self {
            module_id: ModuleId::new("external.counter.consumer").expect("module id"),
            requirement: fabric::Requires::<AdaptedCounter>::provisional(),
            capture,
        }
    }
}

impl ModuleRuntime for CounterConsumer {
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
        *self.capture.lock().expect("capture") = Some(CounterCapture {
            value: resolved.value().current_value().value(),
            provider: resolved.value().adapter_provider(),
            identity: resolved.value().adapter_identity(),
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct OperationsCapture {
    marker: OperationMarker,
    provider: ModuleId,
    identity: ContractIdentity,
}

#[derive(Clone)]
struct OperationsConsumer {
    module_id: ModuleId,
    requirement: SystemRequires<AdaptedOperations>,
    capture: Arc<Mutex<Option<OperationsCapture>>>,
}

impl OperationsConsumer {
    fn new(capture: Arc<Mutex<Option<OperationsCapture>>>) -> Self {
        Self {
            module_id: ModuleId::new("external.operations.consumer").expect("module id"),
            requirement: SystemRequires::<AdaptedOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            ),
            capture,
        }
    }
}

impl ModuleRuntime for OperationsConsumer {
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
        *self.capture.lock().expect("capture") = Some(OperationsCapture {
            marker: resolved.value().current_marker(),
            provider: resolved.provider().clone(),
            identity: resolved.identity().clone(),
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
fn external_resource_adapter_can_target_a_macro_generated_resource_from_another_crate() {
    let capture = Arc::new(Mutex::new(None));
    let adapted = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(ExternalCounterAdapter::new(ExternalCounterAdapterConfig {
            value: 41,
        }))
        .expect("adapter");
    let resource_module_id = adapted.resource().module_id().clone();
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.external.counter")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("runtime")
        .block("consumer", |block| {
            block.module(CounterConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer")
        .select_provider(selection)
        .select_provider(ContractProviderSelection::new(
            module("external.counter.consumer").expect("consumer id"),
            AdaptedCounter::primary_contract_key().id().clone(),
            resource_module_id,
        ))
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named_on("fabric.test.external.counter.instance", &test_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(
        capture.lock().expect("capture").clone(),
        Some(CounterCapture {
            value: 41,
            provider: "fabric.test.counter.adapted.selection.7072696d617279.realization".to_owned(),
            identity: "1.0.0".to_owned(),
        })
    );
}

#[test]
fn external_system_adapter_can_target_a_macro_generated_system_from_another_crate() {
    let capture = Arc::new(Mutex::new(None));
    let adapted = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(ExternalOperationsAdapter::new(
            ExternalOperationsAdapterConfig { value: 77 },
        ))
        .expect("adapter");
    let system_module_id = adapted.system().module_id().clone();
    let (system, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.external.operations")
        .expect("builder")
        .block("runtime", |block| block.module(system).module(adapter))
        .expect("runtime")
        .block("consumer", |block| {
            block.module(OperationsConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer")
        .select_provider(selection)
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named_on("fabric.test.external.operations.instance", &test_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(
        capture.lock().expect("capture").clone(),
        Some(OperationsCapture {
            marker: OperationMarker::new(77),
            provider: system_module_id,
            identity: ContractIdentity::versioned(adapted_operations_contract_version()),
        })
    );
}

#[test]
fn external_adapter_can_consume_systems_through_the_canonical_adapter_surface() {
    let capture = Arc::new(Mutex::new(None));
    let base = TestOperations::select(TestOperationsConfig::new(10, 5)).expect("system");
    let adapted = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(ExternalOperationsWithSystemAdapter::new(
            ExternalOperationsWithSystemAdapterConfig { offset: 3 },
        ))
        .expect("adapter");
    let (base_system, adapted_system, adapter, selection) = {
        let (system, adapter, selection) = adapted.into_raw_parts();
        (base, system, adapter, selection)
    };

    let composition = FabricBuilder::new("fabric.test.external.operations.dependency")
        .expect("builder")
        .block("runtime", |block| {
            block
                .module(base_system)
                .module(adapted_system)
                .module(adapter)
        })
        .expect("runtime")
        .block("consumer", |block| {
            block.module(OperationsConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer")
        .select_provider(selection)
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named_on(
            "fabric.test.external.operations.dependency.instance",
            &test_host(),
        )
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(
        capture
            .lock()
            .expect("capture")
            .clone()
            .expect("capture")
            .marker,
        OperationMarker::new(13)
    );
}

#[test]
fn external_host_bound_resource_adapter_still_requires_explicit_host_truth() {
    let adapted = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(HostBoundExternalCounterAdapter::new(
            HostBoundExternalCounterAdapterConfig { value: 9 },
        ))
        .expect("adapter");
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.external.counter.host")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("runtime")
        .select_provider(selection)
        .build()
        .expect("composition");

    let error = composition
        .materialize_named("fabric.test.external.counter.host.instance")
        .expect_err("hostless materialization should fail");
    assert!(matches!(
        error,
        CompositionError::HostDescriptorRequired { .. }
    ));

    let error = composition
        .materialize_named_on("fabric.test.external.counter.host.missing", &test_host())
        .expect_err("missing facility");
    assert!(matches!(
        error,
        CompositionError::HostIncompatible { source: HostCompatibilityError::MissingRequiredFacilities { missing }, .. }
            if missing == vec![external_counter_host_facility()]
    ));

    let mut instance = composition
        .materialize_named_on(
            "fabric.test.external.counter.host.instance",
            &test_host().with_facility(external_counter_host_facility()),
        )
        .expect("compatible host");
    instance.start().expect("start");
    instance.stop().expect("stop instance");
}
