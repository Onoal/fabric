use std::sync::{Arc, Mutex};

use fabric::{
    authoring::{CompositionExt, FabricBuilder},
    ids::module,
    prelude::{
        AdaptableResourceDefinition, AdapterDefinition, HostArchitecture, HostDescriptor,
        HostOperatingSystem, HostRequirement, PrimaryResourceContract, Requires,
        ResourceDefinition,
    },
};
use fabric_core::{
    CompositionError, ContractCompatibilityRequirement, ContractIdentity,
    ContractProviderSelection, ContractVersion, ContractVersionRequirement, Health, Module,
    ModuleBindings, ModuleContract, ModuleError, ModuleId, ModuleRuntime,
    ProvidedContractDeclaration,
};
use fabric_resource::ResourceCompatibilityError;

use crate::definition::{adapted_counter, derived_counter};
use crate::{
    AdaptedCounter, AdaptedCounterConfig, CounterValue, DerivedCounter, DerivedCounterConfig,
    DirectCounter, DirectCounterConfig, FixedCounterAdapter, FixedCounterAdapterConfig,
    IncompatibleSchemaCounterAdapter, IncompatibleSchemaCounterAdapterConfig,
    MissingContractCounterAdapter, PackageOnlyAdapter, PackageOnlyAdapterConfig, PackageOnlyConfig,
    PackageOnlyResource, WrongVersionCounterAdapter, WrongVersionCounterAdapterConfig,
    counter_resource_id, direct_counter::raw as direct_counter_raw,
};

fabric::resource! {
    pub VersionedCounter {
        id: "fabric.test.counter.versioned";

        schema: "2.0.0";

        config {
            value: u64;
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.versioned";
                version: "1.4.0";

                fn current_value(&self) -> CounterValue;
            }
        }

        runtime {
            fn current_value(&self) -> CounterValue {
                CounterValue::new(self.config.value)
            }
        }
    }
}

fabric::resource! {
    pub VersionedDerivedCounter {
        id: "fabric.test.counter.versioned-derived";

        schema: provisional;

        config {}

        requires {
            source: VersionedCounter(version = "^1.2");
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.versioned-derived";
                version: provisional;

                fn current_value(&self) -> CounterValue;
                fn source_provider(&self) -> String;
                fn source_identity(&self) -> String;
            }
        }

        runtime {
            fn current_value(&self) -> CounterValue {
                CounterValue::new(self.source.current_value().value() + 1)
            }

            fn source_provider(&self) -> String {
                self.source.provider().as_str().to_owned()
            }

            fn source_identity(&self) -> String {
                self.source.identity().to_string()
            }
        }
    }
}

fabric::resource! {
    pub IncompatibleVersionedDerivedCounter {
        id: "fabric.test.counter.incompatible-versioned-derived";

        schema: provisional;

        config {}

        requires {
            source: VersionedCounter(version = "^2");
        }

        contracts {
            primary Api {
                id: "fabric.test.resource.counter.incompatible-versioned-derived";
                version: provisional;

                fn current_value(&self) -> CounterValue;
            }
        }

        runtime {
            fn current_value(&self) -> CounterValue {
                CounterValue::new(self.source.current_value().value())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DerivedObservation {
    provider: ModuleId,
    value: u64,
    source_provider: String,
    source_identity: String,
    source_contract_pointer: usize,
}

fn test_host() -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("arch"),
    )
}

#[derive(Clone)]
struct DerivedConsumer {
    module_id: ModuleId,
    requirement: Requires<DerivedCounter>,
    capture: Arc<Mutex<Option<DerivedObservation>>>,
}

impl DerivedConsumer {
    fn new(module_id: &str, capture: Arc<Mutex<Option<DerivedObservation>>>) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            requirement: Requires::<DerivedCounter>::provisional(),
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
        *self.capture.lock().expect("capture lock") = Some(DerivedObservation {
            provider: resolved.provider().clone(),
            value: resolved.value().current_value().value(),
            source_provider: resolved.value().source_provider(),
            source_identity: resolved.value().source_identity(),
            source_contract_pointer: resolved.value().source_contract_pointer(),
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

#[derive(Clone)]
struct DerivedPointerConsumer {
    module_id: ModuleId,
    requirement: Requires<DerivedCounter>,
    captures: Arc<Mutex<Vec<usize>>>,
}

impl DerivedPointerConsumer {
    fn new(captures: Arc<Mutex<Vec<usize>>>) -> Self {
        Self {
            module_id: ModuleId::new("derived.pointer.consumer").expect("module id"),
            requirement: Requires::<DerivedCounter>::provisional(),
            captures,
        }
    }
}

impl ModuleRuntime for DerivedPointerConsumer {
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
        self.captures
            .lock()
            .expect("captures")
            .push(resolved.value().source_contract_pointer());
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
struct VersionedDerivedObservation {
    provider: ModuleId,
    value: u64,
    source_provider: String,
    source_identity: String,
}

#[derive(Clone)]
struct VersionedDerivedConsumer {
    module_id: ModuleId,
    requirement: Requires<VersionedDerivedCounter>,
    capture: Arc<Mutex<Option<VersionedDerivedObservation>>>,
}

impl VersionedDerivedConsumer {
    fn new(capture: Arc<Mutex<Option<VersionedDerivedObservation>>>) -> Self {
        Self {
            module_id: ModuleId::new("versioned.derived.consumer").expect("module id"),
            requirement: Requires::<VersionedDerivedCounter>::provisional(),
            capture,
        }
    }
}

impl ModuleRuntime for VersionedDerivedConsumer {
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
        *self.capture.lock().expect("capture lock") = Some(VersionedDerivedObservation {
            provider: resolved.provider().clone(),
            value: resolved.value().current_value().value(),
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
struct AdaptedObservation {
    provider: ModuleId,
    value: u64,
    adapter_provider: String,
    adapter_identity: String,
}

#[derive(Clone)]
struct AdaptedConsumer {
    module_id: ModuleId,
    requirement: Requires<AdaptedCounter>,
    capture: Arc<Mutex<Option<AdaptedObservation>>>,
}

impl AdaptedConsumer {
    fn new(module_id: &str, capture: Arc<Mutex<Option<AdaptedObservation>>>) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            requirement: Requires::<AdaptedCounter>::provisional(),
            capture,
        }
    }
}

impl ModuleRuntime for AdaptedConsumer {
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
        *self.capture.lock().expect("capture lock") = Some(AdaptedObservation {
            provider: resolved.provider().clone(),
            value: resolved.value().current_value().value(),
            adapter_provider: resolved.value().adapter_provider(),
            adapter_identity: resolved.value().adapter_identity(),
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
fn direct_counter_resource_has_no_adapter_ceremony_and_materializes_directly() {
    let module = direct_counter_raw::Runtime::new(
        ModuleId::new("counter.module").expect("module id"),
        DirectCounterConfig { value: 7 },
    );

    assert_eq!(counter_resource_id().as_str(), "fabric.test.counter");
    assert_eq!(
        direct_counter_raw::primary_contract_id().as_str(),
        "fabric.test.resource.counter"
    );
    assert_eq!(
        module.provided_contract_declarations(),
        vec![ProvidedContractDeclaration::provisional(
            direct_counter_raw::primary_contract_id()
        )]
    );
    assert_eq!(DirectCounter::resource_id(), counter_resource_id());
    assert_eq!(
        DirectCounter::primary_contract_key().declaration(),
        ProvidedContractDeclaration::provisional(direct_counter_raw::primary_contract_id())
    );

    let selection =
        DirectCounter::select("primary", DirectCounterConfig { value: 7 }).expect("selection");
    assert_eq!(selection.name().as_str(), "primary");
}

#[test]
fn direct_counter_definition_contains_no_dummy_adapter_workaround() {
    let definition =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/definition.rs"))
            .expect("read definition");

    for forbidden in [
        "type AdapterHandle",
        "DirectResourceDefinition",
        "NoAdapter",
        "NullAdapter",
        "DirectAdapter",
        "impl ResourceDefinition for DirectCounter",
        "impl PrimaryResourceContract for DirectCounter",
        "impl ModuleRuntime for",
        "ContractKey",
        "ContractRequirement",
    ] {
        assert!(
            !definition.contains(forbidden),
            "direct resource fixture must not contain dummy adapter workaround {forbidden}"
        );
    }
}

#[test]
fn macro_generated_resource_keeps_schema_and_contract_versions_distinct() {
    use versioned_counter::raw::ApiService;

    let runtime = versioned_counter::raw::Runtime::new(
        ModuleId::new("versioned.counter.module").expect("module id"),
        VersionedCounterConfig { value: 13 },
    );

    assert_eq!(runtime.current_value().value(), 13);
    assert_eq!(
        VersionedCounter::schema().resource(),
        &VersionedCounter::resource_id()
    );
    assert_eq!(
        VersionedCounter::schema().identity(),
        &fabric_resource::ResourceSchemaIdentity::versioned(
            fabric_resource::ResourceSchemaVersion::parse("2.0.0").expect("schema version"),
        )
    );
    assert_eq!(
        VersionedCounter::primary_contract_key().identity(),
        &fabric_core::ContractIdentity::versioned(
            fabric_core::ContractVersion::parse("1.4.0").expect("contract version"),
        )
    );
}

#[test]
fn incompatible_versioned_resource_runtime_method_remains_reachable() {
    let method = <self::incompatible_versioned_derived_counter::raw::Runtime as self::incompatible_versioned_derived_counter::raw::ApiService>::current_value;
    let _ = method;
}

#[test]
fn multiple_resources_can_share_one_rust_module_with_distinct_raw_namespaces() {
    use crate::direct_counter::raw::ApiService as DirectApiService;

    let direct_runtime = crate::direct_counter::raw::Runtime::new(
        ModuleId::new("direct.counter.module").expect("direct module id"),
        DirectCounterConfig { value: 10 },
    );
    let derived_runtime = derived_counter::raw::Runtime::new(
        ModuleId::new("derived.counter.module").expect("derived module id"),
        DerivedCounterConfig {},
    );

    assert_eq!(direct_runtime.current_value().value(), 10);
    let _ = derived_runtime.id();

    let direct_selection =
        DirectCounter::select("alpha", DirectCounterConfig { value: 10 }).expect("selection");
    let derived_selection =
        DerivedCounter::select("beta", DerivedCounterConfig {}).expect("selection");

    assert_ne!(direct_selection.module_id(), derived_selection.module_id());
    assert_eq!(
        DerivedCounter::primary_contract_key().id(),
        &derived_counter::raw::primary_contract_id()
    );
    assert_eq!(
        AdaptedCounter::primary_contract_key().id(),
        &adapted_counter::raw::primary_contract_id()
    );
}

#[test]
fn derived_counter_requires_direct_counter_and_exposes_typed_runtime_dependency_api() {
    let capture = Arc::new(Mutex::new(None));
    let direct =
        DirectCounter::select("primary", DirectCounterConfig { value: 9 }).expect("selection");
    let derived = DerivedCounter::select("derived", DerivedCounterConfig {}).expect("selection");
    let direct_module_id = direct.module_id().clone();
    let derived_module_id = derived.module_id().clone();

    let composition = FabricBuilder::new("fabric.test.counter.derived-flow")
        .expect("builder")
        .block("runtime", |block| block.module(direct).module(derived))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(DerivedConsumer::new(
                "derived.consumer",
                Arc::clone(&capture),
            ))
        })
        .expect("consumer block")
        .select_provider(ContractProviderSelection::new(
            module("derived.consumer").expect("consumer"),
            DerivedCounter::primary_contract_key().id().clone(),
            derived_module_id.clone(),
        ))
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named("fabric.test.counter.derived-flow.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    let observation = capture
        .lock()
        .expect("capture")
        .clone()
        .expect("observation");
    assert_eq!(observation.provider, derived_module_id);
    assert_eq!(observation.value, 18);
    assert_eq!(observation.source_provider, direct_module_id.as_str());
    assert_eq!(observation.source_identity, "provisional");
    assert_ne!(observation.source_contract_pointer, 0);
}

#[test]
fn derived_counter_dependency_is_ambiguous_without_explicit_provider_selection() {
    let direct_a =
        DirectCounter::select("primary", DirectCounterConfig { value: 9 }).expect("selection");
    let direct_b =
        DirectCounter::select("secondary", DirectCounterConfig { value: 10 }).expect("selection");
    let derived = DerivedCounter::select("derived", DerivedCounterConfig {}).expect("selection");

    let error = FabricBuilder::new("fabric.test.counter.derived-ambiguity")
        .expect("builder")
        .block("runtime", |block| {
            block
                .module(direct_a)
                .module(direct_b)
                .module(derived.clone())
        })
        .expect("runtime block")
        .build()
        .expect_err("ambiguous dependency must fail");

    assert!(matches!(
        error,
        CompositionError::AmbiguousProvider { module_id, contract_id, .. }
            if module_id == derived.module_id().clone()
                && contract_id == DirectCounter::primary_contract_key().id().clone()
    ));
}

#[test]
fn composition_can_explicitly_select_a_dependent_resource_provider() {
    let capture = Arc::new(Mutex::new(None));
    let direct_a =
        DirectCounter::select("primary", DirectCounterConfig { value: 9 }).expect("selection");
    let direct_b =
        DirectCounter::select("secondary", DirectCounterConfig { value: 50 }).expect("selection");
    let derived = DerivedCounter::select("derived", DerivedCounterConfig {}).expect("selection");
    let derived_module_id = derived.module_id().clone();
    let chosen_provider = direct_b.module_id().clone();

    let composition = FabricBuilder::new("fabric.test.counter.derived-selected")
        .expect("builder")
        .block("runtime", |block| {
            block.module(direct_a).module(direct_b).module(derived)
        })
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(DerivedConsumer::new(
                "derived.consumer.selected",
                Arc::clone(&capture),
            ))
        })
        .expect("consumer block")
        .select_provider(ContractProviderSelection::new(
            derived_module_id.clone(),
            DirectCounter::primary_contract_key().id().clone(),
            chosen_provider.clone(),
        ))
        .select_provider(ContractProviderSelection::new(
            module("derived.consumer.selected").expect("consumer"),
            DerivedCounter::primary_contract_key().id().clone(),
            derived_module_id.clone(),
        ))
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named("fabric.test.counter.derived-selected.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    let observation = capture
        .lock()
        .expect("capture")
        .clone()
        .expect("observation");
    assert_eq!(observation.provider, derived_module_id);
    assert_eq!(observation.value, 100);
    assert_eq!(observation.source_provider, chosen_provider.as_str());
}

#[test]
fn versioned_resource_dependency_preserves_compatibility_and_provenance() {
    let capture = Arc::new(Mutex::new(None));
    let versioned = VersionedCounter::select("primary", VersionedCounterConfig { value: 20 })
        .expect("selection");
    let versioned_module_id = versioned.module_id().clone();
    let derived = VersionedDerivedCounter::select("derived", VersionedDerivedCounterConfig {})
        .expect("selection");
    let derived_module_id = derived.module_id().clone();

    let composition = FabricBuilder::new("fabric.test.counter.versioned-derived")
        .expect("builder")
        .block("runtime", |block| block.module(versioned).module(derived))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(VersionedDerivedConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer block")
        .select_provider(ContractProviderSelection::new(
            derived_module_id.clone(),
            VersionedCounter::primary_contract_key().id().clone(),
            versioned_module_id.clone(),
        ))
        .select_provider(ContractProviderSelection::new(
            module("versioned.derived.consumer").expect("consumer"),
            VersionedDerivedCounter::primary_contract_key().id().clone(),
            derived_module_id.clone(),
        ))
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named("fabric.test.counter.versioned-derived.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    assert_eq!(
        capture.lock().expect("capture").clone(),
        Some(VersionedDerivedObservation {
            provider: derived_module_id,
            value: 21,
            source_provider: versioned_module_id.as_str().to_owned(),
            source_identity: "1.4.0".to_owned(),
        })
    );
}

#[test]
fn incompatible_explicit_versioned_dependency_fails_through_core_compatibility() {
    let versioned = VersionedCounter::select("primary", VersionedCounterConfig { value: 20 })
        .expect("selection");
    let versioned_module_id = versioned.module_id().clone();
    let derived = IncompatibleVersionedDerivedCounter::select(
        "derived",
        IncompatibleVersionedDerivedCounterConfig {},
    )
    .expect("selection");
    let derived_module_id = derived.module_id().clone();

    let error = FabricBuilder::new("fabric.test.counter.incompatible-versioned-derived")
        .expect("builder")
        .block("runtime", |block| block.module(versioned).module(derived))
        .expect("runtime block")
        .select_provider(ContractProviderSelection::new(
            derived_module_id.clone(),
            VersionedCounter::primary_contract_key().id().clone(),
            versioned_module_id.clone(),
        ))
        .build()
        .expect_err("incompatible versioned dependency must fail");

    assert!(matches!(
        error,
        CompositionError::SelectedProviderIncompatible {
            consumer,
            contract_id,
            provider,
            required_compatibility,
            available_identities,
        } if consumer == derived_module_id
            && contract_id == VersionedCounter::primary_contract_key().id().clone()
            && provider == versioned_module_id
            && required_compatibility == ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^2").expect("requirement"),
            )
            && available_identities == vec![ContractIdentity::versioned(
                ContractVersion::parse("1.4.0").expect("version"),
            )]
    ));
}

#[test]
fn contract_dependency_state_is_runtime_local_across_materializations() {
    let captures = Arc::new(Mutex::new(Vec::new()));
    let direct =
        DirectCounter::select("primary", DirectCounterConfig { value: 9 }).expect("selection");
    let derived = DerivedCounter::select("derived", DerivedCounterConfig {}).expect("selection");
    let derived_module_id = derived.module_id().clone();

    let composition = FabricBuilder::new("fabric.test.counter.fresh-materialization")
        .expect("builder")
        .block("runtime", |block| block.module(direct).module(derived))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(DerivedPointerConsumer::new(Arc::clone(&captures)))
        })
        .expect("consumer block")
        .select_provider(ContractProviderSelection::new(
            module("derived.pointer.consumer").expect("consumer"),
            DerivedCounter::primary_contract_key().id().clone(),
            derived_module_id.clone(),
        ))
        .build()
        .expect("composition");

    captures.lock().expect("captures").clear();

    let mut first = composition
        .materialize_named_on("fabric.test.counter.first", &test_host())
        .expect("first");
    let mut second = composition
        .materialize_named_on("fabric.test.counter.second", &test_host())
        .expect("second");

    first.start().expect("first start");
    second.start().expect("second start");
    second.stop();
    first.stop();

    let pointers = captures.lock().expect("captures").clone();
    assert_eq!(pointers.len(), 2);
    assert_ne!(pointers[0], pointers[1]);
}

#[test]
fn adapted_counter_implements_adaptable_resource_definition_and_using_works_unchanged() {
    let adapted = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(FixedCounterAdapter::new(FixedCounterAdapterConfig {
            value: 10,
        }))
        .expect("adapter");

    assert_eq!(
        AdaptedCounter::realization_requirement().id(),
        &adapted_counter::realization::raw::contract_id()
    );
    assert_eq!(
        adapted.adapter().provider_module_id().as_str(),
        format!("{}.realization", adapted.resource().module_id().as_str())
    );
    assert!(
        !adapted
            .adapter()
            .provider_module_id()
            .as_str()
            .contains("fixed_counter"),
        "realization provider slot must stay structural rather than adapter-name-based"
    );
}

#[test]
fn adapted_counter_delegates_through_generated_realization_contract() {
    let capture = Arc::new(Mutex::new(None));
    let adapted = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(FixedCounterAdapter::new(FixedCounterAdapterConfig {
            value: 10,
        }))
        .expect("adapter");
    let resource_module_id = adapted.resource().module_id().clone();
    let realization_module_id = adapted.adapter().provider_module_id().clone();
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.counter.adapted")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(AdaptedConsumer::new(
                "adapted.consumer",
                Arc::clone(&capture),
            ))
        })
        .expect("consumer block")
        .select_provider(selection)
        .select_provider(ContractProviderSelection::new(
            module("adapted.consumer").expect("consumer"),
            AdaptedCounter::primary_contract_key().id().clone(),
            resource_module_id.clone(),
        ))
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named_on("fabric.test.counter.adapted.instance", &test_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    assert_eq!(
        capture.lock().expect("capture").clone(),
        Some(AdaptedObservation {
            provider: resource_module_id,
            value: 10,
            adapter_provider: realization_module_id.as_str().to_owned(),
            adapter_identity: "1.0.0".to_owned(),
        })
    );
}

#[test]
fn resource_schema_incompatibility_still_fails_during_using() {
    let error = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(IncompatibleSchemaCounterAdapter::new(
            IncompatibleSchemaCounterAdapterConfig { value: 1 },
        ))
        .err()
        .expect("schema-incompatible adapter must fail");

    assert!(matches!(
        error,
        ResourceCompatibilityError::AdapterSupportIncompatible { .. }
    ));
}

#[test]
fn malformed_adapter_provider_fails_through_core_contract_validation() {
    let adapted = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(MissingContractCounterAdapter)
        .expect("schema-compatible adapter");
    let resource_module_id = adapted.resource().module_id().clone();
    let realization_module_id = adapted.adapter().provider_module_id().clone();
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let error = FabricBuilder::new("fabric.test.counter.adapted.malformed")
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
        } if consumer == resource_module_id
            && contract_id == adapted_counter::realization::raw::contract_id()
            && provider == realization_module_id
    ));
}

#[test]
fn realization_contract_compatibility_remains_core_validation() {
    let adapted = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(WrongVersionCounterAdapter::new(
            WrongVersionCounterAdapterConfig { value: 42 },
        ))
        .expect("schema-compatible adapter");
    let resource_module_id = adapted.resource().module_id().clone();
    let realization_module_id = adapted.adapter().provider_module_id().clone();
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let error = FabricBuilder::new("fabric.test.counter.adapted.incompatible")
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
        } if consumer == resource_module_id
            && contract_id == adapted_counter::realization::raw::contract_id()
            && provider == realization_module_id
            && required_compatibility == ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^1").expect("requirement"),
            )
            && available_identities == vec![ContractIdentity::versioned(
                ContractVersion::parse("2.0.0").expect("version"),
            )]
    ));
}

#[test]
fn two_adapted_resource_instances_bind_distinct_realization_providers() {
    let primary_capture = Arc::new(Mutex::new(None));
    let secondary_capture = Arc::new(Mutex::new(None));
    let primary = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(FixedCounterAdapter::new(FixedCounterAdapterConfig {
            value: 10,
        }))
        .expect("adapter");
    let secondary = AdaptedCounter::select("secondary", AdaptedCounterConfig {})
        .expect("selection")
        .using(FixedCounterAdapter::new(FixedCounterAdapterConfig {
            value: 100,
        }))
        .expect("adapter");

    let primary_resource_module_id = primary.resource().module_id().clone();
    let secondary_resource_module_id = secondary.resource().module_id().clone();
    let primary_realization_module_id = primary.adapter().provider_module_id().clone();
    let secondary_realization_module_id = secondary.adapter().provider_module_id().clone();
    let (primary_resource, primary_adapter, primary_selection) = primary.into_raw_parts();
    let (secondary_resource, secondary_adapter, secondary_selection) = secondary.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.counter.two-adapted")
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
                .module(AdaptedConsumer::new(
                    "adapted.consumer.primary",
                    Arc::clone(&primary_capture),
                ))
                .module(AdaptedConsumer::new(
                    "adapted.consumer.secondary",
                    Arc::clone(&secondary_capture),
                ))
        })
        .expect("consumer block")
        .select_provider(primary_selection)
        .select_provider(secondary_selection)
        .select_provider(ContractProviderSelection::new(
            module("adapted.consumer.primary").expect("consumer"),
            AdaptedCounter::primary_contract_key().id().clone(),
            primary_resource_module_id.clone(),
        ))
        .select_provider(ContractProviderSelection::new(
            module("adapted.consumer.secondary").expect("consumer"),
            AdaptedCounter::primary_contract_key().id().clone(),
            secondary_resource_module_id.clone(),
        ))
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named_on("fabric.test.counter.two-adapted.instance", &test_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    assert_eq!(
        primary_capture.lock().expect("capture").clone(),
        Some(AdaptedObservation {
            provider: primary_resource_module_id,
            value: 10,
            adapter_provider: primary_realization_module_id.as_str().to_owned(),
            adapter_identity: "1.0.0".to_owned(),
        })
    );
    assert_eq!(
        secondary_capture.lock().expect("capture").clone(),
        Some(AdaptedObservation {
            provider: secondary_resource_module_id,
            value: 100,
            adapter_provider: secondary_realization_module_id.as_str().to_owned(),
            adapter_identity: "1.0.0".to_owned(),
        })
    );
}

#[test]
fn declaration_only_resource_defines_and_selects_without_runtime_or_composition() {
    assert_eq!(
        PackageOnlyResource::resource_id().as_str(),
        "fabric.test.package-only"
    );
    assert_eq!(
        PackageOnlyResource::schema().resource(),
        &PackageOnlyResource::resource_id()
    );

    let selection = PackageOnlyResource::select(
        "primary",
        PackageOnlyConfig {
            endpoint: "https://packages.example/capability".to_owned(),
        },
    )
    .expect("selection");
    assert_eq!(selection.name().as_str(), "primary");
    assert_eq!(
        selection.config().endpoint,
        "https://packages.example/capability"
    );

    let declaration = selection.declaration();
    assert_eq!(declaration.module_id(), selection.module_id());
    assert_eq!(
        declaration.provided_contracts(),
        &[PackageOnlyResource::primary_contract_key().declaration()]
    );
    assert!(selection.materialize().is_none());
}

#[test]
fn declaration_only_resource_source_contains_no_runtime_obligation() {
    let source =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/package_only.rs"))
            .expect("read package-only witness");
    let code = source
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !(trimmed.starts_with("//!") || trimmed.starts_with("///"))
        })
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "impl ResourceDefinition for PackageOnlyResource",
        "impl PrimaryResourceContract for PackageOnlyResource",
        "fn resource_id()",
        "fn schema()",
        "fn declaration(",
        "PackageOnlyConfig",
    ] {
        assert!(
            code.contains(required),
            "package-only witness must keep declarative anchor {required}"
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
            "package-only witness must not regain runtime obligation {forbidden}"
        );
    }
}

#[test]
fn declaration_only_resource_validates_declaratively_and_fails_materialization_explicitly() {
    let selection = PackageOnlyResource::select(
        "primary",
        PackageOnlyConfig {
            endpoint: "https://packages.example/capability".to_owned(),
        },
    )
    .expect("selection");
    let module_id = selection.module_id().clone();

    let composition = FabricBuilder::new("fabric.test.package-only.flow")
        .expect("builder")
        .block("runtime", |block| block.module(selection))
        .expect("runtime block")
        .build()
        .expect("declaration-only composition must validate without runtime materialization");

    let error = composition
        .materialize_named("fabric.test.package-only.instance")
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

#[test]
fn declaration_only_adapter_defines_target_schema_and_realization_without_runtime() {
    let adapter = PackageOnlyAdapter::new(PackageOnlyAdapterConfig {
        label: "package-counter-notes".to_owned(),
    });

    assert_eq!(adapter.config().label, "package-counter-notes");
    assert_eq!(
        adapter.schema_support().resource(),
        &AdaptedCounter::resource_id()
    );
    adapter
        .schema_support()
        .accepts_schema(&AdaptedCounter::schema())
        .expect("adapter schema support must accept the target schema");
    assert_eq!(adapter.host_requirement(), HostRequirement::new());

    let provider_id = module("package.adapter.provider").expect("module id");
    let declaration = adapter.declaration(provider_id.clone());
    assert_eq!(declaration.module_id(), &provider_id);
    assert_eq!(
        declaration.provided_contracts(),
        &[adapted_counter::realization::raw::contract_key(
            fabric_core::ContractVersion::parse("1.0.0").expect("version"),
        )
        .declaration()]
    );
    assert!(adapter.materialize_provider(provider_id).is_none());
}

#[test]
fn declaration_only_adapter_source_contains_no_runtime_obligation() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/package_adapter.rs"
    ))
    .expect("read package-only adapter witness");
    let code = source
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !(trimmed.starts_with("//!") || trimmed.starts_with("///"))
        })
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "impl AdapterDefinition for PackageOnlyAdapter",
        "type Target = AdaptedCounter",
        "type SchemaSupport",
        "fn schema_support(",
        "fn declaration(",
        "PackageOnlyAdapterConfig",
    ] {
        assert!(
            code.contains(required),
            "package-only adapter witness must keep declarative anchor {required}"
        );
    }
    for forbidden in [
        "ModuleRuntime",
        "materialize_provider",
        "initialize",
        "fn start",
        "fn stop",
        "health",
        "Instance",
        "Composition",
        "AdapterId",
        "Component",
        "Registry",
    ] {
        assert!(
            !code.contains(forbidden),
            "package-only adapter witness must not regain runtime obligation {forbidden}"
        );
    }
}

#[test]
fn resource_plus_adapter_form_a_declaration_only_partial_slice() {
    let adapted = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(PackageOnlyAdapter::new(PackageOnlyAdapterConfig {
            label: "package-counter-notes".to_owned(),
        }))
        .expect("adapter");
    let resource_module_id = adapted.resource().module_id().clone();
    let provider_module_id = adapted.adapter().provider_module_id().clone();

    let resource_declaration = adapted.resource().declaration();
    let provider_declaration = adapted.adapter().declaration();
    assert_eq!(resource_declaration.module_id(), &resource_module_id);
    assert_eq!(provider_declaration.module_id(), &provider_module_id);
    assert!(provider_declaration.host_requirement().is_some());
    assert_eq!(adapted.provider_selection().consumer(), &resource_module_id);
    assert_eq!(adapted.provider_selection().provider(), &provider_module_id);

    assert!(adapted.resource().materialize().is_some());
    assert!(adapted.adapter().materialize().is_none());
}

#[test]
fn declaration_only_adapter_provider_validates_then_fails_materialization_explicitly() {
    let adapted = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(PackageOnlyAdapter::new(PackageOnlyAdapterConfig {
            label: "package-counter-notes".to_owned(),
        }))
        .expect("adapter");
    let provider_module_id = adapted.adapter().provider_module_id().clone();
    let (resource, adapter, selection) = adapted.into_raw_parts();

    let composition = FabricBuilder::new("fabric.test.counter.package-adapter")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("runtime block")
        .select_provider(selection)
        .build()
        .expect("declaration-only realization graph must validate without runtime materialization");

    let error = composition
        .materialize_named_on("fabric.test.counter.package-adapter.instance", &test_host())
        .expect_err("adapter provider without runtime must not materialize silently");
    assert!(
        matches!(
            error,
            CompositionError::MissingRuntimeMaterializer { module_id: missing }
                if missing == provider_module_id
        ),
        "materialization must fail through the Core missing-materializer boundary"
    );
}
