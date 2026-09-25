use std::sync::{Arc, Mutex};

use fabric::authoring::*;
use fabric::prelude::*;
use fabric::resource::AdapterResourceSchemaSupport;
use fabric::{
    authoring::{CompositionExt, FabricBuilder},
    ids::module,
    resource::ResourceSchemaRequirement,
};
use fabric_core::{
    CompositionError, ContractCompatibilityRequirement, ContractIdentity, ContractKey,
    ContractProviderSelection, ContractVersion, ContractVersionRequirement, Health, ModuleBindings,
    ModuleContract, ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_test_adapter_clock_memory::{
    HostBoundClockAdapter, MemoryClock, host_bound_clock_facility,
};
use fabric_test_resource_clock::{
    BoundClockRealization, Clock, ClockConfig, ClockError, ClockRealization,
    ClockRealizationCapture, ClockRealizationContract, ClockTick, clock_contract_id,
    clock_contract_version, clock_realization_contract_id, clock_realization_contract_key,
};
use fabric_test_resource_counter::{DirectCounter, DirectCounterConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
struct CapturedClockResolution {
    provider: ModuleId,
    identity: ContractIdentity,
    tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CapturedCounterResolution {
    module_id: ModuleId,
    value: u64,
}

#[derive(Clone)]
struct ClockConsumer {
    module_id: ModuleId,
    requirement: Requires<Clock>,
    capture: Arc<Mutex<Option<CapturedClockResolution>>>,
}

fn sdk_host() -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("arch"),
    )
}

impl ClockConsumer {
    fn new(module_id: &str, capture: Arc<Mutex<Option<CapturedClockResolution>>>) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            requirement: Requires::<Clock>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("version requirement"),
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
            .resolve_with_provider(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        let tick = resolved
            .value()
            .current_tick()
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.capture.lock().expect("clock capture") = Some(CapturedClockResolution {
            provider: resolved.provider().clone(),
            identity: resolved.identity().clone(),
            tick: tick.value(),
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
fn sdk_versioned_requires_clock_resolves_primary_contract_and_preserves_provenance() {
    let capture = Arc::new(Mutex::new(None));
    let adapted = Clock::select("primary", ClockConfig::default())
        .expect("clock selection")
        .using(MemoryClock::new(7))
        .expect("clock adapter");
    let expected_provider = adapted.resource().module_id().clone();
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.sdk.extension")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(ClockConsumer::new(
                "sdk.clock.consumer",
                Arc::clone(&capture),
            ))
        })
        .expect("consumer block")
        .select_provider(selection)
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_core_on("fabric.test.sdk.extension.instance", &sdk_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(CapturedClockResolution {
            provider: expected_provider,
            identity: ContractIdentity::versioned(clock_contract_version()),
            tick: 7,
        })
    );
}

#[test]
fn provisional_requires_clock_keeps_the_primary_contract_id_and_remains_provisional() {
    let requirement = Requires::<Clock>::provisional();

    assert_eq!(
        requirement.as_contract_requirement().id(),
        &clock_contract_id()
    );
    assert_eq!(
        requirement.as_contract_requirement().compatibility(),
        &ContractCompatibilityRequirement::provisional()
    );
}

#[test]
fn adapter_backed_clock_requires_explicit_host_truth_even_when_host_requirement_is_empty() {
    let adapted = Clock::select("primary", ClockConfig::default())
        .expect("clock selection")
        .using(MemoryClock::new(7))
        .expect("clock adapter");
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.sdk.host-required")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("runtime block")
        .select_provider(selection)
        .build()
        .expect("composition");

    let error = composition
        .materialize_core("fabric.test.sdk.host-required.instance")
        .expect_err("adapter-backed compositions should require explicit host truth");

    assert!(matches!(
        error,
        CompositionError::HostDescriptorRequired { module_ids }
            if module_ids.len() == 1
                && module_ids[0].as_str().ends_with(".realization")
    ));
}

#[test]
fn host_bound_clock_adapter_evaluates_required_facility_during_materialization() {
    let capture = Arc::new(Mutex::new(None));
    let adapted = Clock::select("primary", ClockConfig::default())
        .expect("clock selection")
        .using(HostBoundClockAdapter::new(17))
        .expect("clock adapter");
    let expected_provider = adapted.resource().module_id().clone();
    let expected_realization_provider = adapted.adapter().provider_module_id().clone();
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.sdk.host-bound")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(ClockConsumer::new(
                "sdk.clock.host-bound.consumer",
                Arc::clone(&capture),
            ))
        })
        .expect("consumer block")
        .select_provider(selection)
        .build()
        .expect("composition");

    let missing_host = sdk_host();
    let error = composition
        .materialize_core_on("fabric.test.sdk.host-bound.missing", &missing_host)
        .expect_err("missing host facility should fail");

    assert!(matches!(
        error,
        CompositionError::HostIncompatible {
            module_id,
            source: HostCompatibilityError::MissingRequiredFacilities { missing },
        } if module_id == expected_realization_provider
            && missing == vec![host_bound_clock_facility()]
    ));

    let mut instance = composition
        .materialize_core_on(
            "fabric.test.sdk.host-bound.instance",
            &sdk_host().with_facility(host_bound_clock_facility()),
        )
        .expect("host with facility");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(CapturedClockResolution {
            provider: expected_provider,
            identity: ContractIdentity::versioned(clock_contract_version()),
            tick: 17,
        })
    );
}

#[derive(Clone)]
struct CounterConsumer {
    module_id: ModuleId,
    requirement: Requires<DirectCounter>,
    capture: Arc<Mutex<Option<CapturedCounterResolution>>>,
}

impl CounterConsumer {
    fn new(capture: Arc<Mutex<Option<CapturedCounterResolution>>>) -> Self {
        Self {
            module_id: ModuleId::new("sdk.counter.consumer").expect("module id"),
            requirement: Requires::<DirectCounter>::provisional(),
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
        *self.capture.lock().expect("counter capture") = Some(CapturedCounterResolution {
            module_id: resolved.provider().clone(),
            value: resolved.value().current_value().value(),
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
fn direct_resource_realization_works_without_any_adapter_object() {
    let capture = Arc::new(Mutex::new(None));
    let selection =
        DirectCounter::select("primary", DirectCounterConfig { value: 29 }).expect("selection");
    let provider_module_id = selection.module_id().clone();
    let requirement = Requires::<DirectCounter>::provisional();

    assert_eq!(
        requirement.as_contract_requirement().id(),
        DirectCounter::primary_contract_key().id()
    );

    let composition = FabricBuilder::new("fabric.test.sdk.direct-resource")
        .expect("builder")
        .block("runtime", |block| block.module(selection.clone()))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(CounterConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer block")
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_core("fabric.test.sdk.direct-resource.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(CapturedCounterResolution {
            module_id: provider_module_id,
            value: 29,
        })
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CapturedRequirementMismatch {
    module_id: ModuleId,
    contract_id: fabric_core::ContractId,
    declared_compatibility: ContractCompatibilityRequirement,
    requested_compatibility: ContractCompatibilityRequirement,
}

#[derive(Clone)]
struct MismatchedClockConsumer {
    module_id: ModuleId,
    declared_requirement: Requires<Clock>,
    bind_requirement: Requires<Clock>,
    capture: Arc<Mutex<Option<CapturedRequirementMismatch>>>,
}

impl MismatchedClockConsumer {
    fn new(capture: Arc<Mutex<Option<CapturedRequirementMismatch>>>) -> Self {
        Self {
            module_id: ModuleId::new("sdk.clock.mismatch").expect("module id"),
            declared_requirement: Requires::<Clock>::versioned(
                ContractVersionRequirement::parse("^1").expect("version requirement"),
            ),
            bind_requirement: Requires::<Clock>::versioned(
                ContractVersionRequirement::parse("^2").expect("version requirement"),
            ),
            capture,
        }
    }
}

impl ModuleRuntime for MismatchedClockConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.declared_requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let error = match self.bind_requirement.resolve(bindings) {
            Ok(_) => panic!("mismatched clock requirement must fail"),
            Err(error) => error,
        };
        let CompositionError::RequirementDeclarationMismatch {
            module_id,
            contract_id,
            declared_compatibility,
            requested_compatibility,
        } = error
        else {
            panic!("expected requirement declaration mismatch")
        };
        *self.capture.lock().expect("capture lock") = Some(CapturedRequirementMismatch {
            module_id,
            contract_id,
            declared_compatibility,
            requested_compatibility,
        });
        Err(ModuleError::new("typed requirement declaration mismatch"))
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
fn typed_requires_clock_still_preserves_fx5_requirement_declaration_integrity() {
    let capture = Arc::new(Mutex::new(None));
    let adapted = Clock::select("primary", ClockConfig::default())
        .expect("clock selection")
        .using(MemoryClock::new(13))
        .expect("clock adapter");
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.sdk.clock-mismatch")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(MismatchedClockConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer block")
        .select_provider(selection)
        .build()
        .expect("declaration graph validates without executing runtime bind");

    let error = composition
        .materialize_core_on("fabric.test.sdk.clock-mismatch.instance", &sdk_host())
        .expect_err("runtime bind must reject the undeclared requirement");

    assert!(matches!(
        error,
        CompositionError::ModuleFailure { phase: "bind", .. }
    ));
    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(CapturedRequirementMismatch {
            module_id: module("sdk.clock.mismatch").expect("module id"),
            contract_id: clock_contract_id(),
            declared_compatibility: ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^1").expect("version requirement"),
            ),
            requested_compatibility: ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^2").expect("version requirement"),
            ),
        })
    );
}

#[derive(Clone)]
struct IncompatibleClockSchema;

impl AdapterDefinition for IncompatibleClockSchema {
    fn adapter_definition_id(&self) -> fabric::authoring::AdapterDefinitionId {
        fabric::authoring::AdapterDefinitionId::new("test.manual.incompatible-clock-schema")
            .expect("static test Adapter definition ID is valid")
    }

    type Target = Clock;
    type Compatibility = AdapterResourceSchemaSupport;

    fn compatibility(&self) -> AdapterResourceSchemaSupport {
        AdapterResourceSchemaSupport::versioned(
            Clock::resource_id(),
            ResourceSchemaRequirement::parse("^2").expect("schema requirement"),
        )
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(provider_module_id)
    }

    fn materialize_provider(
        &self,
        _provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        panic!("schema compatibility must fail before provider materialization")
    }
}

#[test]
fn sdk_rejects_incompatible_adapter_schema_support_before_materialization() {
    let error = match Clock::select("primary", ClockConfig::default())
        .expect("selection")
        .using(IncompatibleClockSchema)
    {
        Ok(_) => panic!("incompatible adapter must be rejected"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        ResourceCompatibilityError::AdapterSupportIncompatible { .. }
    ));
}

#[derive(Clone)]
struct AlternateClockAdapter {
    start: u64,
}

impl AlternateClockAdapter {
    fn new(start: u64) -> Self {
        Self { start }
    }
}

impl AdapterDefinition for AlternateClockAdapter {
    fn adapter_definition_id(&self) -> fabric::authoring::AdapterDefinitionId {
        fabric::authoring::AdapterDefinitionId::new("test.manual.alternate-clock-adapter")
            .expect("static test Adapter definition ID is valid")
    }

    type Target = Clock;
    type Compatibility = AdapterResourceSchemaSupport;

    fn compatibility(&self) -> AdapterResourceSchemaSupport {
        AdapterResourceSchemaSupport::versioned(
            Clock::resource_id(),
            ResourceSchemaRequirement::parse("^1").expect("schema requirement"),
        )
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(provider_module_id)
            .with_provided_contracts(vec![clock_realization_contract_key().declaration()])
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(FixedClockProvider::new(
            provider_module_id,
            clock_realization_contract_key(),
            self.start,
        )))
    }
}

#[derive(Clone)]
struct WrongVersionClockAdapter;

impl AdapterDefinition for WrongVersionClockAdapter {
    fn adapter_definition_id(&self) -> fabric::authoring::AdapterDefinitionId {
        fabric::authoring::AdapterDefinitionId::new("test.manual.wrong-version-clock-adapter")
            .expect("static test Adapter definition ID is valid")
    }

    type Target = Clock;
    type Compatibility = AdapterResourceSchemaSupport;

    fn compatibility(&self) -> AdapterResourceSchemaSupport {
        AdapterResourceSchemaSupport::versioned(
            Clock::resource_id(),
            ResourceSchemaRequirement::parse("^1").expect("schema requirement"),
        )
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        let key: ContractKey<ClockRealizationContract> = ContractKey::versioned(
            clock_realization_contract_id(),
            ContractVersion::parse("2.0.0").expect("version"),
        );
        ModuleDeclaration::new(provider_module_id).with_provided_contracts(vec![key.declaration()])
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(FixedClockProvider::new(
            provider_module_id,
            ContractKey::versioned(
                clock_realization_contract_id(),
                ContractVersion::parse("2.0.0").expect("version"),
            ),
            99,
        )))
    }
}

#[derive(Clone)]
struct MissingContractClockAdapter;

impl AdapterDefinition for MissingContractClockAdapter {
    fn adapter_definition_id(&self) -> fabric::authoring::AdapterDefinitionId {
        fabric::authoring::AdapterDefinitionId::new("test.manual.missing-contract-clock-adapter")
            .expect("static test Adapter definition ID is valid")
    }

    type Target = Clock;
    type Compatibility = AdapterResourceSchemaSupport;

    fn compatibility(&self) -> AdapterResourceSchemaSupport {
        AdapterResourceSchemaSupport::versioned(
            Clock::resource_id(),
            ResourceSchemaRequirement::parse("^1").expect("schema requirement"),
        )
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(provider_module_id)
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(EmptyClockProvider::new(provider_module_id)))
    }
}

#[derive(Clone)]
struct FixedRealization(u64);

impl ClockRealization for FixedRealization {
    fn current_tick(&self) -> Result<ClockTick, ClockError> {
        Ok(ClockTick::new(self.0))
    }
}

#[derive(Clone)]
struct FixedClockProvider {
    module_id: ModuleId,
    key: ContractKey<ClockRealizationContract>,
    tick: u64,
}

impl FixedClockProvider {
    fn new(module_id: ModuleId, key: ContractKey<ClockRealizationContract>, tick: u64) -> Self {
        Self {
            module_id,
            key,
            tick,
        }
    }
}

impl ModuleRuntime for FixedClockProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![self.key.declaration()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &self.key,
            Arc::new(ClockRealizationContract::new(Arc::new(FixedRealization(
                self.tick,
            )))),
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
struct EmptyClockProvider {
    module_id: ModuleId,
}

impl EmptyClockProvider {
    fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }
}

impl ModuleRuntime for EmptyClockProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
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

#[test]
fn realization_slot_module_id_is_structural_and_independent_of_adapter_name() {
    let memory = Clock::select("primary", ClockConfig::default())
        .expect("selection")
        .using(MemoryClock::new(1))
        .expect("adapter");
    let alternate = Clock::select("primary", ClockConfig::default())
        .expect("selection")
        .using(AlternateClockAdapter::new(1))
        .expect("adapter");
    let expected = format!("{}.realization", memory.resource().module_id().as_str());

    assert_eq!(
        memory.adapter().provider_module_id().as_str(),
        expected.as_str()
    );
    assert_eq!(
        alternate.adapter().provider_module_id().as_str(),
        expected.as_str()
    );
    assert!(
        !memory
            .adapter()
            .provider_module_id()
            .as_str()
            .contains("memory"),
        "structural provider slot must not encode adapter implementation names"
    );
}

#[test]
fn malformed_adapter_provider_missing_realization_contract_fails_through_core() {
    let adapted = Clock::select("primary", ClockConfig::default())
        .expect("selection")
        .using(MissingContractClockAdapter)
        .expect("schema-compatible adapter");
    let provider_module_id = adapted.adapter().provider_module_id().clone();
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let error = FabricBuilder::new("fabric.test.sdk.malformed-adapter")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
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
        } if consumer == module("fabric.test.clock.selection.7072696d617279").expect("consumer")
            && contract_id == clock_realization_contract_id()
            && provider == provider_module_id
    ));
}

#[test]
fn realization_contract_compatibility_is_distinct_from_schema_compatibility() {
    let adapted = Clock::select("primary", ClockConfig::default())
        .expect("selection")
        .using(WrongVersionClockAdapter)
        .expect("schema-compatible adapter");
    let provider_module_id = adapted.adapter().provider_module_id().clone();
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let error = FabricBuilder::new("fabric.test.sdk.incompatible-realization")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("runtime block")
        .select_provider(selection)
        .build()
        .expect_err("incompatible realization contract must fail");

    assert!(matches!(
        error,
        CompositionError::SelectedProviderIncompatible {
            consumer,
            contract_id,
            provider,
            required_compatibility,
            available_identities,
        } if consumer == module("fabric.test.clock.selection.7072696d617279").expect("consumer")
            && contract_id == clock_realization_contract_id()
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
fn two_clock_instances_bind_distinct_realization_providers_and_public_consumers() {
    let primary_realization_capture: ClockRealizationCapture = Arc::new(Mutex::new(None));
    let secondary_realization_capture: ClockRealizationCapture = Arc::new(Mutex::new(None));
    let primary = Clock::select(
        "primary",
        ClockConfig::default().with_realization_capture(Arc::clone(&primary_realization_capture)),
    )
    .expect("primary selection")
    .using(MemoryClock::new(10))
    .expect("primary adapter");
    let secondary = Clock::select(
        "secondary",
        ClockConfig::default().with_realization_capture(Arc::clone(&secondary_realization_capture)),
    )
    .expect("secondary selection")
    .using(MemoryClock::new(100))
    .expect("secondary adapter");
    let primary_provider_capture = Arc::new(Mutex::new(None));
    let secondary_provider_capture = Arc::new(Mutex::new(None));
    let primary_resource_module_id = primary.resource().module_id().clone();
    let secondary_resource_module_id = secondary.resource().module_id().clone();
    let primary_realization_module_id = primary.adapter().provider_module_id().clone();
    let secondary_realization_module_id = secondary.adapter().provider_module_id().clone();
    let (primary_resource, primary_adapter, primary_selection) = primary.into_raw_parts();
    let (secondary_resource, secondary_adapter, secondary_selection) = secondary.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.sdk.two-clocks")
        .expect("builder")
        .block("runtime", |block| {
            block
                .module(primary_resource)
                .module(primary_adapter)
                .module(secondary_resource)
                .module(secondary_adapter)
        })
        .expect("runtime block")
        .block("consumer", |block| {
            block
                .module(ClockConsumer::new(
                    "sdk.clock.consumer.primary",
                    Arc::clone(&primary_provider_capture),
                ))
                .module(ClockConsumer::new(
                    "sdk.clock.consumer.secondary",
                    Arc::clone(&secondary_provider_capture),
                ))
        })
        .expect("consumer block")
        .select_provider(primary_selection)
        .select_provider(secondary_selection)
        .select_provider(ContractProviderSelection::new(
            module("sdk.clock.consumer.primary").expect("consumer"),
            clock_contract_id(),
            primary_resource_module_id.clone(),
        ))
        .select_provider(ContractProviderSelection::new(
            module("sdk.clock.consumer.secondary").expect("consumer"),
            clock_contract_id(),
            secondary_resource_module_id.clone(),
        ))
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_core_on("fabric.test.sdk.two-clocks.instance", &sdk_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(
        primary_provider_capture
            .lock()
            .expect("capture lock")
            .clone(),
        Some(CapturedClockResolution {
            provider: primary_resource_module_id,
            identity: ContractIdentity::versioned(clock_contract_version()),
            tick: 10,
        })
    );
    assert_eq!(
        secondary_provider_capture
            .lock()
            .expect("capture lock")
            .clone(),
        Some(CapturedClockResolution {
            provider: secondary_resource_module_id,
            identity: ContractIdentity::versioned(clock_contract_version()),
            tick: 100,
        })
    );
    assert_eq!(
        primary_realization_capture
            .lock()
            .expect("realization capture")
            .clone(),
        Some(BoundClockRealization {
            provider: primary_realization_module_id,
            identity: clock_realization_contract_key().identity().clone(),
        })
    );
    assert_eq!(
        secondary_realization_capture
            .lock()
            .expect("realization capture")
            .clone(),
        Some(BoundClockRealization {
            provider: secondary_realization_module_id,
            identity: clock_realization_contract_key().identity().clone(),
        })
    );
}

#[test]
fn realization_dependency_order_is_driven_by_the_contract_graph() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let adapted = Clock::select(
        "primary",
        ClockConfig::default().with_lifecycle_capture(Arc::clone(&events)),
    )
    .expect("selection")
    .using(MemoryClock::new(5).with_lifecycle_capture(Arc::clone(&events)))
    .expect("adapter");
    let resource_module_id = adapted.resource().module_id().clone();
    let realization_module_id = adapted.adapter().provider_module_id().clone();
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.sdk.lifecycle")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("runtime")
        .select_provider(selection)
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_core_on("fabric.test.sdk.lifecycle.instance", &sdk_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(
        events.lock().expect("events").clone(),
        vec![
            format!("initialize:{}", realization_module_id.as_str()),
            format!("initialize:{}", resource_module_id.as_str()),
            format!("start:{}", realization_module_id.as_str()),
            format!("start:{}", resource_module_id.as_str()),
            format!("stop:{}", resource_module_id.as_str()),
            format!("stop:{}", realization_module_id.as_str()),
        ]
    );
}
