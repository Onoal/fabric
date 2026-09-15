use std::sync::{Arc, Mutex};

use fabric::{
    authoring::{CompositionExt, FabricBuilder},
    core::ContractCompatibilityRequirement,
    prelude::*,
};
use fabric_core::{
    CompositionError, ContractIdentity, ContractVersionRequirement, Health, Module, ModuleBindings,
    ModuleContract, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_test_resource_clock::{Clock, ClockConfig};

use crate::adapted::LifecycleCaptureOperationsAdapter;
use crate::{
    AdaptedOperations, AdaptedOperationsConfig, AlternateOperationsAdapter,
    AlternateOperationsAdapterConfig, DerivedOperations, DerivedOperationsConfig,
    FixedOperationsAdapter, FixedOperationsAdapterConfig, HostBoundOperationsAdapter,
    HostBoundOperationsAdapterConfig, IncompatibleSchemaOperationsAdapter,
    IncompatibleSchemaOperationsAdapterConfig, MissingContractOperationsAdapter, OperationMarker,
    OperationsDrivenClockAdapter, PackageOnlySystem, PackageOnlySystemConfig, SystemBackedResource,
    SystemBackedResourceConfig, TestOperations, TestOperationsConfig,
    WrongVersionOperationsAdapter, WrongVersionOperationsAdapterConfig,
    adapted_operations_contract_version, adapted_operations_realization_contract_id,
    adapted_operations_realization_contract_key, host_bound_operations_facility,
    operations_contract_id, operations_contract_version,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct CapturedSystemResolution {
    provider: ModuleId,
    identity: ContractIdentity,
    marker: OperationMarker,
}

fn test_host() -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("arch"),
    )
}

#[derive(Clone)]
struct OperationsConsumer {
    module_id: ModuleId,
    requirement: SystemRequires<TestOperations>,
    capture: Arc<Mutex<Option<CapturedSystemResolution>>>,
}

impl OperationsConsumer {
    fn new(module_id: &str, capture: Arc<Mutex<Option<CapturedSystemResolution>>>) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            requirement: SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("static requirement"),
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

    fn stop(&mut self) {}

    fn health(&self) -> Health {
        Health::Healthy
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CapturedDerivedSystem {
    provider: ModuleId,
    marker: OperationMarker,
    source_provider: String,
    source_identity: String,
}

#[derive(Clone)]
struct DerivedConsumer {
    module_id: ModuleId,
    requirement: SystemRequires<DerivedOperations>,
    capture: Arc<Mutex<Option<CapturedDerivedSystem>>>,
}

impl DerivedConsumer {
    fn new(capture: Arc<Mutex<Option<CapturedDerivedSystem>>>) -> Self {
        Self {
            module_id: ModuleId::new("derived.system.consumer").expect("module id"),
            requirement: SystemRequires::<DerivedOperations>::provisional(),
            capture,
        }
    }
}

impl ModuleRuntime for DerivedConsumer {
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
        *self.capture.lock().expect("capture lock") = Some(CapturedDerivedSystem {
            provider: resolved.provider().clone(),
            marker: resolved.value().current_marker(),
            source_provider: resolved.value().source_provider(),
            source_identity: resolved.value().source_identity(),
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct CapturedResourceValue {
    provider: ModuleId,
    value: u64,
    system_provider: String,
    system_identity: String,
}

#[derive(Clone)]
struct ResourceConsumer {
    module_id: ModuleId,
    requirement: Requires<SystemBackedResource>,
    capture: Arc<Mutex<Option<CapturedResourceValue>>>,
}

impl ResourceConsumer {
    fn new(capture: Arc<Mutex<Option<CapturedResourceValue>>>) -> Self {
        Self {
            module_id: ModuleId::new("resource.system.consumer").expect("module id"),
            requirement: Requires::<SystemBackedResource>::provisional(),
            capture,
        }
    }
}

impl ModuleRuntime for ResourceConsumer {
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
        *self.capture.lock().expect("capture lock") = Some(CapturedResourceValue {
            provider: resolved.provider().clone(),
            value: resolved.value().current_value(),
            system_provider: resolved.value().system_provider(),
            system_identity: resolved.value().system_identity(),
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct CapturedClockTick {
    tick: u64,
}

#[derive(Clone)]
struct ClockConsumer {
    module_id: ModuleId,
    requirement: Requires<Clock>,
    capture: Arc<Mutex<Option<CapturedClockTick>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CapturedAdaptedSystem {
    provider: ModuleId,
    identity: ContractIdentity,
    marker: OperationMarker,
    realization_provider: String,
    realization_identity: String,
}

#[derive(Clone)]
struct AdaptedSystemConsumer {
    module_id: ModuleId,
    requirement: SystemRequires<AdaptedOperations>,
    capture: Arc<Mutex<Option<CapturedAdaptedSystem>>>,
}

impl AdaptedSystemConsumer {
    fn new(capture: Arc<Mutex<Option<CapturedAdaptedSystem>>>) -> Self {
        Self {
            module_id: ModuleId::new("adapted.system.consumer").expect("module id"),
            requirement: SystemRequires::<AdaptedOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("static requirement"),
            ),
            capture,
        }
    }
}

impl ModuleRuntime for AdaptedSystemConsumer {
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
        *self.capture.lock().expect("capture lock") = Some(CapturedAdaptedSystem {
            provider: resolved.provider().clone(),
            identity: resolved.identity().clone(),
            marker: resolved.value().current_marker(),
            realization_provider: resolved.value().realization_provider(),
            realization_identity: resolved.value().realization_identity(),
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

impl ClockConsumer {
    fn new(capture: Arc<Mutex<Option<CapturedClockTick>>>) -> Self {
        Self {
            module_id: ModuleId::new("clock.system.consumer").expect("module id"),
            requirement: Requires::<Clock>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("static requirement"),
            ),
            capture,
        }
    }
}

impl ModuleRuntime for ClockConsumer {
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
            .resolve(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.capture.lock().expect("capture lock") = Some(CapturedClockTick {
            tick: resolved
                .current_tick()
                .map_err(|error| ModuleError::new(error.to_string()))?
                .value(),
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

#[test]
fn system_requires_is_anchor_safe_and_versioned() {
    let requirement = SystemRequires::<TestOperations>::versioned(
        ContractVersionRequirement::parse("^1.2").expect("requirement"),
    );

    assert_eq!(
        requirement.as_contract_requirement().id(),
        &operations_contract_id()
    );
    assert_eq!(
        requirement.as_contract_requirement().compatibility(),
        &ContractCompatibilityRequirement::versioned(
            ContractVersionRequirement::parse("^1.2").expect("requirement"),
        )
    );
}

#[test]
fn multiple_systems_can_share_one_rust_module_with_distinct_raw_namespaces() {
    let alpha =
        crate::twins::AlphaOperations::select(crate::twins::AlphaOperationsConfig { value: 10 })
            .expect("alpha selection");
    let beta =
        crate::twins::BetaOperations::select(crate::twins::BetaOperationsConfig { value: 20 })
            .expect("beta selection");

    let alpha_type = ::std::any::TypeId::of::<crate::twins::alpha_operations::raw::ApiContract>();
    let beta_type = ::std::any::TypeId::of::<crate::twins::beta_operations::raw::ApiContract>();

    assert_ne!(alpha_type, beta_type);
    assert_ne!(
        crate::twins::alpha_operations::raw::system_id(),
        crate::twins::beta_operations::raw::system_id(),
    );

    let composition = FabricBuilder::new("fabric.test.system.same-module")
        .expect("builder")
        .block("runtime", |block| block.module(alpha).module(beta))
        .expect("runtime block")
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named("fabric.test.system.same-module.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop();
}

#[test]
fn one_system_occurrence_per_composition_falls_out_of_duplicate_module_ids() {
    let error = FabricBuilder::new("fabric.test.system.duplicate")
        .expect("builder")
        .block("runtime", |block| {
            block
                .module(
                    TestOperations::select(TestOperationsConfig::new(1, 1))
                        .expect("system selection"),
                )
                .module(
                    TestOperations::select(TestOperationsConfig::new(99, 5))
                        .expect("system selection"),
                )
        })
        .expect("runtime block")
        .build()
        .expect_err("duplicate system selection must fail");

    assert!(matches!(error, CompositionError::DuplicateModuleId { .. }));
}

#[test]
fn generic_system_consumer_resolves_typed_contract_and_preserves_provenance() {
    let capture = Arc::new(Mutex::new(None));
    let operations = TestOperations::select(TestOperationsConfig::new(7, 3)).expect("selection");
    let provider_module_id = operations.module_id().clone();

    let composition = FabricBuilder::new("fabric.test.system.consumer")
        .expect("builder")
        .block("runtime", |block| block.module(operations))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(OperationsConsumer::new(
                "system.consumer",
                Arc::clone(&capture),
            ))
        })
        .expect("consumer block")
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named("fabric.test.system.consumer.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(CapturedSystemResolution {
            provider: provider_module_id,
            identity: ContractIdentity::versioned(operations_contract_version()),
            marker: OperationMarker::new(7),
        })
    );
}

#[test]
fn fresh_instances_and_separate_compositions_do_not_share_system_state() {
    let capture_a = Arc::new(Mutex::new(None));
    let capture_b = Arc::new(Mutex::new(None));
    let capture_c = Arc::new(Mutex::new(None));

    let composition = FabricBuilder::new("fabric.test.system.instance-scope")
        .expect("builder")
        .block("runtime", |block| {
            block.module(
                TestOperations::select(TestOperationsConfig::new(11, 2)).expect("system selection"),
            )
        })
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(OperationsConsumer::new(
                "system.consumer.a",
                Arc::clone(&capture_a),
            ))
        })
        .expect("consumer block")
        .build()
        .expect("composition");

    let mut instance_a = composition
        .materialize_named("fabric.test.system.instance-a")
        .expect("instance");
    instance_a.start().expect("start");
    instance_a.stop();

    let mut instance_b = composition
        .materialize_named("fabric.test.system.instance-b")
        .expect("instance");
    instance_b.start().expect("start");
    instance_b.stop();

    let composition_other = FabricBuilder::new("fabric.test.system.other")
        .expect("builder")
        .block("runtime", |block| {
            block.module(
                TestOperations::select(TestOperationsConfig::new(100, 10))
                    .expect("system selection"),
            )
        })
        .expect("runtime block")
        .block("consumer", |block| {
            block
                .module(OperationsConsumer::new(
                    "system.consumer.b",
                    Arc::clone(&capture_b),
                ))
                .module(OperationsConsumer::new(
                    "system.consumer.c",
                    Arc::clone(&capture_c),
                ))
        })
        .expect("consumer block")
        .build()
        .expect("composition");

    let mut instance_c = composition_other
        .materialize_named("fabric.test.system.instance-c")
        .expect("instance");
    instance_c.start().expect("start");
    instance_c.stop();

    assert_eq!(
        capture_a
            .lock()
            .expect("capture")
            .clone()
            .expect("capture")
            .marker,
        OperationMarker::new(11)
    );
    assert_eq!(
        capture_b
            .lock()
            .expect("capture")
            .clone()
            .expect("capture")
            .marker,
        OperationMarker::new(100)
    );
    assert_eq!(
        capture_c
            .lock()
            .expect("capture")
            .clone()
            .expect("capture")
            .marker,
        OperationMarker::new(110)
    );
}

#[test]
fn system_to_system_and_resource_to_system_use_the_same_contract_resolver() {
    let derived_capture = Arc::new(Mutex::new(None));
    let resource_capture = Arc::new(Mutex::new(None));
    let operations = TestOperations::select(TestOperationsConfig::new(5, 1)).expect("selection");
    let operations_provider = operations.module_id().clone();
    let derived =
        DerivedOperations::select(DerivedOperationsConfig { offset: 20 }).expect("selection");
    let derived_provider = derived.module_id().clone();
    let resource =
        SystemBackedResource::select("primary", SystemBackedResourceConfig { multiplier: 10 })
            .expect("resource");
    let resource_provider = resource.module_id().clone();

    let composition = FabricBuilder::new("fabric.test.system.cross-plane")
        .expect("builder")
        .block("runtime", |block| {
            block.module(operations).module(derived).module(resource)
        })
        .expect("runtime block")
        .block("consumer", |block| {
            block
                .module(DerivedConsumer::new(Arc::clone(&derived_capture)))
                .module(ResourceConsumer::new(Arc::clone(&resource_capture)))
        })
        .expect("consumer block")
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named("fabric.test.system.cross-plane.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    assert_eq!(
        derived_capture.lock().expect("capture").clone(),
        Some(CapturedDerivedSystem {
            provider: derived_provider,
            marker: OperationMarker::new(25),
            source_provider: operations_provider.as_str().to_owned(),
            source_identity: "1.4.0".to_owned(),
        })
    );
    assert_eq!(
        resource_capture.lock().expect("capture").clone(),
        Some(CapturedResourceValue {
            provider: resource_provider,
            value: 60,
            system_provider: operations_provider.as_str().to_owned(),
            system_identity: "1.4.0".to_owned(),
        })
    );
}

#[test]
fn resource_target_adapter_can_consume_system_through_typed_contracts_under_the_generic_adapter_trait()
 {
    let capture = Arc::new(Mutex::new(None));
    let adapted = Clock::select("primary", ClockConfig::default())
        .expect("clock selection")
        .using(OperationsDrivenClockAdapter::new(100))
        .expect("adapter");
    let (clock, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.system.adapter-consumer")
        .expect("builder")
        .block("runtime", |block| {
            block
                .module(
                    TestOperations::select(TestOperationsConfig::new(3, 1))
                        .expect("system selection"),
                )
                .module(clock)
                .module(adapter)
        })
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(ClockConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer block")
        .select_provider(selection)
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named_on("fabric.test.system.adapter-consumer.instance", &test_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    assert_eq!(
        capture.lock().expect("capture").clone(),
        Some(CapturedClockTick { tick: 103 })
    );
}

#[derive(Clone)]
struct DishonestSystemConfig;

struct DishonestSystem;

impl SystemDefinition for DishonestSystem {
    type Config = DishonestSystemConfig;

    fn system_id() -> SystemId {
        SystemId::new("fabric.test.system.honest").expect("system id")
    }

    fn schema() -> fabric_system::SystemSchemaDescriptor {
        fabric_system::SystemSchemaDescriptor::provisional(
            SystemId::new("fabric.test.system.foreign").expect("system id"),
        )
    }

    fn declaration(_selection: &SystemSelection<Self>) -> fabric_core::ModuleDeclaration {
        unreachable!("dishonest selection must fail before declaration")
    }

    fn materialize(
        _selection: &SystemSelection<Self>,
    ) -> Option<Box<dyn fabric_core::ModuleRuntime>> {
        unreachable!("dishonest selection must fail before materialization")
    }
}

#[test]
fn system_selection_rejects_dishonest_definition_identity_before_materialization() {
    assert!(matches!(
        DishonestSystem::select(DishonestSystemConfig),
        Err(fabric_system::SystemCompatibilityError::SystemIdentityMismatch { .. })
    ));
}

#[test]
fn adapted_system_realization_uses_core_provider_selection_and_keeps_consumers_realization_blind() {
    let capture = Arc::new(Mutex::new(None));
    let adapted = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
            value: 10,
        }))
        .expect("adapter");
    let system_module_id = adapted.system().module_id().clone();
    let realization_module_id = adapted.adapter().provider_module_id().clone();
    let (system, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.system.adapted")
        .expect("builder")
        .block("runtime", |block| block.module(system).module(adapter))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(AdaptedSystemConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer block")
        .select_provider(selection)
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named_on("fabric.test.system.adapted.instance", &test_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    assert_eq!(
        capture.lock().expect("capture").clone(),
        Some(CapturedAdaptedSystem {
            provider: system_module_id.clone(),
            identity: ContractIdentity::versioned(adapted_operations_contract_version()),
            marker: OperationMarker::new(10),
            realization_provider: realization_module_id.as_str().to_owned(),
            realization_identity: adapted_operations_realization_contract_key()
                .identity()
                .to_string(),
        })
    );
    assert_ne!(system_module_id, realization_module_id);
}

#[test]
fn adapted_system_requires_explicit_host_truth_even_with_empty_adapter_requirement() {
    let adapted = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
            value: 10,
        }))
        .expect("adapter");
    let (system, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.system.host-required")
        .expect("builder")
        .block("runtime", |block| block.module(system).module(adapter))
        .expect("runtime block")
        .select_provider(selection)
        .build()
        .expect("composition");

    let error = composition
        .materialize_named("fabric.test.system.host-required.instance")
        .expect_err("adapter-backed systems should require explicit host truth");

    assert!(matches!(
        error,
        CompositionError::HostDescriptorRequired { module_ids }
            if module_ids.len() == 1
                && module_ids[0].as_str().ends_with(".realization")
    ));
}

#[test]
fn host_bound_system_adapter_evaluates_required_facility_during_materialization() {
    let capture = Arc::new(Mutex::new(None));
    let adapted = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(HostBoundOperationsAdapter::new(
            HostBoundOperationsAdapterConfig { value: 55 },
        ))
        .expect("adapter");
    let expected_provider = adapted.system().module_id().clone();
    let expected_realization_provider = adapted.adapter().provider_module_id().clone();
    let (system, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.system.host-bound")
        .expect("builder")
        .block("runtime", |block| block.module(system).module(adapter))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(AdaptedSystemConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer block")
        .select_provider(selection)
        .build()
        .expect("composition");

    let error = composition
        .materialize_named_on("fabric.test.system.host-bound.missing", &test_host())
        .expect_err("missing host facility should fail");

    assert!(matches!(
        error,
        CompositionError::HostIncompatible {
            module_id,
            source: HostCompatibilityError::MissingRequiredFacilities { missing },
        } if module_id == expected_realization_provider
            && missing == vec![host_bound_operations_facility()]
    ));

    let mut instance = composition
        .materialize_named_on(
            "fabric.test.system.host-bound.instance",
            &test_host().with_facility(host_bound_operations_facility()),
        )
        .expect("host with facility");
    instance.start().expect("start");
    instance.stop();

    assert_eq!(
        capture.lock().expect("capture").clone(),
        Some(CapturedAdaptedSystem {
            provider: expected_provider,
            identity: ContractIdentity::versioned(adapted_operations_contract_version()),
            marker: OperationMarker::new(55),
            realization_provider: expected_realization_provider.as_str().to_owned(),
            realization_identity: adapted_operations_realization_contract_key()
                .identity()
                .to_string(),
        })
    );
}

#[test]
fn adapted_system_realization_provider_slot_is_structural_and_adapter_agnostic() {
    let fixed = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
            value: 1,
        }))
        .expect("adapter");
    let alternate = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(AlternateOperationsAdapter::new(
            AlternateOperationsAdapterConfig { value: 2 },
        ))
        .expect("adapter");
    let expected = format!("{}.realization", fixed.system().module_id().as_str());

    assert_eq!(
        fixed.adapter().provider_module_id().as_str(),
        expected.as_str()
    );
    assert_eq!(
        alternate.adapter().provider_module_id().as_str(),
        expected.as_str()
    );
}

#[test]
fn adapted_system_schema_incompatibility_fails_during_using() {
    let error = match AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(IncompatibleSchemaOperationsAdapter::new(
            IncompatibleSchemaOperationsAdapterConfig { value: 1 },
        )) {
        Ok(_) => panic!("schema-incompatible adapter must fail"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        fabric_system::SystemCompatibilityError::SchemaIncompatible { .. }
    ));
}

#[test]
fn adapted_system_malformed_adapter_missing_realization_contract_fails_through_core() {
    let adapted = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(MissingContractOperationsAdapter)
        .expect("schema-compatible adapter");
    let provider_module_id = adapted.adapter().provider_module_id().clone();
    let consumer_module_id = adapted.system().module_id().clone();
    let (system, adapter, selection) = adapted.into_raw_parts();

    let error = FabricBuilder::new("fabric.test.system.malformed-adapter")
        .expect("builder")
        .block("runtime", |block| block.module(system).module(adapter))
        .expect("runtime block")
        .select_provider(selection)
        .build()
        .expect_err("missing realization contract must fail");

    assert!(matches!(
        error,
        CompositionError::SelectedProviderMissingContract {
            consumer,
            contract_id,
            provider,
        } if consumer == consumer_module_id
            && contract_id == adapted_operations_realization_contract_id()
            && provider == provider_module_id
    ));
}

#[test]
fn adapted_system_schema_and_realization_contract_compatibility_fail_separately() {
    let adapted = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(WrongVersionOperationsAdapter::new(
            WrongVersionOperationsAdapterConfig { value: 99 },
        ))
        .expect("schema-compatible adapter");
    let provider_module_id = adapted.adapter().provider_module_id().clone();
    let consumer_module_id = adapted.system().module_id().clone();
    let (system, adapter, selection) = adapted.into_raw_parts();

    let error = FabricBuilder::new("fabric.test.system.incompatible-realization")
        .expect("builder")
        .block("runtime", |block| block.module(system).module(adapter))
        .expect("runtime block")
        .select_provider(selection)
        .build()
        .expect_err("wrong realization version must fail");

    assert!(matches!(
        error,
        CompositionError::SelectedProviderIncompatible {
            consumer,
            contract_id,
            provider,
            required_compatibility,
            available_identities,
        } if consumer == consumer_module_id
            && contract_id == adapted_operations_realization_contract_id()
            && provider == provider_module_id
            && required_compatibility == ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^1").expect("requirement"),
            )
            && available_identities == vec![ContractIdentity::versioned(
                ContractVersion::parse("2.0.0").expect("version"),
            )]
    ));
}

#[test]
fn adapted_system_instance_state_is_isolated_across_compositions() {
    let capture_a = Arc::new(Mutex::new(None));
    let capture_b = Arc::new(Mutex::new(None));

    let adapted_a = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
            value: 10,
        }))
        .expect("adapter");
    let adapted_b = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
            value: 100,
        }))
        .expect("adapter");
    let (system_a, adapter_a, selection_a) = adapted_a.into_raw_parts();
    let (system_b, adapter_b, selection_b) = adapted_b.into_raw_parts();

    let composition_a = FabricBuilder::new("fabric.test.system.instance-a")
        .expect("builder")
        .block("runtime", |block| block.module(system_a).module(adapter_a))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(AdaptedSystemConsumer::new(Arc::clone(&capture_a)))
        })
        .expect("consumer block")
        .select_provider(selection_a)
        .build()
        .expect("composition");
    let composition_b = FabricBuilder::new("fabric.test.system.instance-b")
        .expect("builder")
        .block("runtime", |block| block.module(system_b).module(adapter_b))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(AdaptedSystemConsumer::new(Arc::clone(&capture_b)))
        })
        .expect("consumer block")
        .select_provider(selection_b)
        .build()
        .expect("composition");

    let mut instance_a = composition_a
        .materialize_named_on("fabric.test.system.instance-a.local", &test_host())
        .expect("instance");
    instance_a.start().expect("start");
    instance_a.stop();

    let mut instance_b = composition_b
        .materialize_named_on("fabric.test.system.instance-b.local", &test_host())
        .expect("instance");
    instance_b.start().expect("start");
    instance_b.stop();

    assert_eq!(
        capture_a
            .lock()
            .expect("capture")
            .clone()
            .expect("capture")
            .marker,
        OperationMarker::new(10)
    );
    assert_eq!(
        capture_b
            .lock()
            .expect("capture")
            .clone()
            .expect("capture")
            .marker,
        OperationMarker::new(100)
    );
}

#[test]
fn adapted_system_adapter_lifecycle_runs_through_the_existing_runtime_model() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let adapted = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(LifecycleCaptureOperationsAdapter::new(
            5,
            Arc::clone(&events),
        ))
        .expect("adapter");
    let realization_module_id = adapted.adapter().provider_module_id().clone();
    let (system, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.system.lifecycle")
        .expect("builder")
        .block("runtime", |block| block.module(system).module(adapter))
        .expect("runtime block")
        .select_provider(selection)
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named_on("fabric.test.system.lifecycle.instance", &test_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    assert_eq!(
        events.lock().expect("events").clone(),
        vec![
            format!("initialize:{}", realization_module_id.as_str()),
            format!("start:{}", realization_module_id.as_str()),
            format!("stop:{}", realization_module_id.as_str()),
        ]
    );
}

#[test]
fn adapted_system_one_occurrence_law_remains_structural() {
    let first = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
            value: 1,
        }))
        .expect("adapter");
    let second = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
            value: 2,
        }))
        .expect("adapter");
    let (system_a, adapter_a, selection_a) = first.into_raw_parts();
    let (system_b, adapter_b, selection_b) = second.into_raw_parts();

    let error = FabricBuilder::new("fabric.test.system.duplicate-adapted")
        .expect("builder")
        .block("runtime", |block| {
            block
                .module(system_a)
                .module(adapter_a)
                .module(system_b)
                .module(adapter_b)
        })
        .expect("runtime block")
        .select_provider(selection_a)
        .select_provider(selection_b)
        .build()
        .expect_err("duplicate adapted systems must fail");

    assert!(matches!(error, CompositionError::DuplicateModuleId { .. }));
}

#[test]
fn declaration_only_system_defines_and_selects_without_runtime_or_composition() {
    assert_eq!(
        PackageOnlySystem::system_id().as_str(),
        "fabric.test.package-only-system"
    );
    assert_eq!(
        PackageOnlySystem::schema().system(),
        &PackageOnlySystem::system_id()
    );

    let selection = PackageOnlySystem::select(PackageOnlySystemConfig {
        endpoint: "https://packages.example/facility".to_owned(),
    })
    .expect("selection");
    assert_eq!(
        selection.config().endpoint,
        "https://packages.example/facility"
    );

    let declaration = selection.declaration();
    assert_eq!(declaration.module_id(), selection.module_id());
    assert_eq!(
        declaration.provided_contracts(),
        &[PackageOnlySystem::primary_contract_key().declaration()]
    );
    assert!(selection.materialize().is_none());
}

#[test]
fn declaration_only_system_source_contains_no_runtime_obligation() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/package_system.rs"
    ))
    .expect("read package-only system witness");
    let code = source
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !(trimmed.starts_with("//!") || trimmed.starts_with("///"))
        })
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "impl SystemDefinition for PackageOnlySystem",
        "impl PrimarySystemContract for PackageOnlySystem",
        "fn system_id()",
        "fn schema()",
        "fn declaration(",
        "PackageOnlySystemConfig",
    ] {
        assert!(
            code.contains(required),
            "package-only system witness must keep declarative anchor {required}"
        );
    }
    for forbidden in [
        "ModuleRuntime",
        "fn materialize",
        "initialize",
        "fn start",
        "fn stop",
        "health",
        "Instance",
        "Composition",
    ] {
        assert!(
            !code.contains(forbidden),
            "package-only system witness must not regain runtime obligation {forbidden}"
        );
    }
}

#[test]
fn declaration_only_system_occurrence_identity_ignores_config() {
    let first = PackageOnlySystem::select(PackageOnlySystemConfig {
        endpoint: "https://packages.example/first".to_owned(),
    })
    .expect("first selection");
    let second = PackageOnlySystem::select(PackageOnlySystemConfig {
        endpoint: "https://packages.example/second".to_owned(),
    })
    .expect("second selection");

    assert_eq!(first.module_id(), second.module_id());

    let error = FabricBuilder::new("fabric.test.system.package-duplicate")
        .expect("builder")
        .block("runtime", |block| block.module(first).module(second))
        .expect("runtime block")
        .build()
        .expect_err("same-SystemId selections must collide declaratively");

    assert!(matches!(error, CompositionError::DuplicateModuleId { .. }));
}

#[test]
fn declaration_only_system_validates_declaratively_and_fails_materialization_explicitly() {
    let selection = PackageOnlySystem::select(PackageOnlySystemConfig {
        endpoint: "https://packages.example/facility".to_owned(),
    })
    .expect("selection");
    let module_id = selection.module_id().clone();

    let composition = FabricBuilder::new("fabric.test.system.package-flow")
        .expect("builder")
        .block("runtime", |block| block.module(selection))
        .expect("runtime block")
        .build()
        .expect("declaration-only composition must validate without runtime materialization");

    let error = composition
        .materialize_named("fabric.test.system.package-flow.instance")
        .expect_err("declaration-only graph must not materialize silently");
    assert!(
        matches!(
            error,
            CompositionError::MissingRuntimeMaterializer { module_id: missing }
                if missing == module_id
        ),
        "materialization must fail through the Core missing-materializer boundary"
    );
}
