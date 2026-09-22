use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use fabric::authoring::*;
use fabric::prelude::*;
use fabric_core::{
    CompositionError, ContractId, ContractKey, ContractRequirement, HostMaterializationRequirement,
    ModuleBindings, ModuleContract, ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_test_adapter_clock_memory::MemoryClock;
use fabric_test_component_greeter::{Greeter, GreeterConfig};
use fabric_test_resource_clock::{Clock, ClockConfig};
use fabric_test_system_operations::{
    AdaptedOperations, AdaptedOperationsConfig, FixedOperationsAdapter,
    FixedOperationsAdapterConfig,
};

use super::support::{NotesConsumer, NotesContract, StaticNotes, VersionedNotesProvider};

const FACILITY: &str = "fabric.test.augmentation.support-integrity";

fn key(id: &str) -> ContractKey<NotesContract> {
    ContractKey::provisional(ContractId::new(id).expect("static contract id"))
}

fn dependency_key() -> ContractKey<NotesContract> {
    key("fabric.test.augmentation.support-integrity.dependency")
}

fn optional_key() -> ContractKey<NotesContract> {
    key("fabric.test.augmentation.support-integrity.optional")
}

fn provided_key() -> ContractKey<NotesContract> {
    key("fabric.test.augmentation.support-integrity.provided")
}

fn facility_requirement(module_id: ModuleId) -> HostMaterializationRequirement {
    HostMaterializationRequirement::new(
        module_id,
        HostRequirement::new().require_facility(HostFacilityId::new(FACILITY).expect("facility")),
    )
}

fn dishonest_module_id() -> ModuleId {
    ModuleId::new("fabric.test.augmentation.support-integrity.dishonest").expect("module id")
}

#[derive(Clone)]
struct ResourceIntegrity;

impl ResourceAugmentationDefinition<Clock> for ResourceIntegrity {
    type Config = ();
    type Contract = NotesContract;

    fn contract_key() -> ContractKey<Self::Contract> {
        key("fabric.test.augmentation.support-integrity.resource")
    }
}

#[derive(Clone)]
struct SystemIntegrity;

impl SystemAugmentationDefinition<AdaptedOperations> for SystemIntegrity {
    type Config = ();
    type Contract = NotesContract;

    fn contract_key() -> ContractKey<Self::Contract> {
        key("fabric.test.augmentation.support-integrity.system")
    }
}

#[derive(Clone)]
struct ComponentIntegrity;

impl ComponentAugmentationDefinition<Greeter> for ComponentIntegrity {
    type Config = ();
    type Contract = NotesContract;

    fn contract_key() -> ContractKey<Self::Contract> {
        key("fabric.test.augmentation.support-integrity.component")
    }
}

struct IntegrityRuntime {
    module_id: ModuleId,
    augmentation: ContractKey<NotesContract>,
    requirements: Vec<ContractRequirement<NotesContract>>,
    bound: Arc<AtomicUsize>,
}

impl ModuleRuntime for IntegrityRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        self.requirements
            .iter()
            .map(|requirement| requirement.declaration().clone())
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![
            ModuleContract::new(
                &self.augmentation,
                Arc::new(NotesContract::new(Arc::new(StaticNotes(
                    "augmentation".to_owned(),
                )))),
            ),
            ModuleContract::new(
                &provided_key(),
                Arc::new(NotesContract::new(Arc::new(StaticNotes(
                    "provided".to_owned(),
                )))),
            ),
        ])
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        for requirement in &self.requirements {
            bindings
                .resolve(requirement)
                .map_err(|error| ModuleError::new(error.to_string()))?;
        }
        self.bound.fetch_add(1, Ordering::SeqCst);
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
struct IntegritySupport(Arc<AtomicUsize>);

impl IntegritySupport {
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::new(dishonest_module_id())
            .with_required_contracts(vec![
                ContractRequirement::<NotesContract>::provisional(dependency_key().id().clone())
                    .declaration()
                    .clone(),
            ])
            .with_optional_contracts(vec![
                ContractRequirement::<NotesContract>::provisional(optional_key().id().clone())
                    .declaration()
                    .clone(),
            ])
            .with_provided_contracts(vec![provided_key().declaration()])
            .with_host_requirement(facility_requirement(dishonest_module_id()))
    }

    fn runtime(
        &self,
        provider_module_id: ModuleId,
        augmentation: ContractKey<NotesContract>,
        mut requirements: Vec<ContractRequirement<NotesContract>>,
    ) -> Option<Box<dyn ModuleRuntime>> {
        requirements.push(ContractRequirement::<NotesContract>::provisional(
            dependency_key().id().clone(),
        ));
        Some(Box::new(IntegrityRuntime {
            module_id: provider_module_id,
            augmentation,
            requirements,
            bound: Arc::clone(&self.0),
        }))
    }
}

impl ResourceAugmentationSupportDefinition<Clock, ResourceIntegrity> for IntegritySupport {
    fn declaration(&self, _: ModuleId) -> ModuleDeclaration {
        self.declaration()
    }

    fn materialize(
        &self,
        _: &ResourceAugmentation<Clock, ResourceIntegrity>,
        provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        self.runtime(
            provider_module_id,
            ResourceIntegrity::contract_key(),
            Vec::new(),
        )
    }
}

impl SystemAugmentationSupportDefinition<AdaptedOperations, SystemIntegrity> for IntegritySupport {
    fn declaration(&self, _: ModuleId) -> ModuleDeclaration {
        self.declaration()
    }

    fn materialize(
        &self,
        _: &SystemAugmentation<AdaptedOperations, SystemIntegrity>,
        provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        self.runtime(
            provider_module_id,
            SystemIntegrity::contract_key(),
            Vec::new(),
        )
    }
}

impl ComponentAugmentationSupportDefinition<Greeter, ComponentIntegrity> for IntegritySupport {
    fn declaration(&self, _: ModuleId) -> ModuleDeclaration {
        self.declaration()
    }

    fn materialize(&self, _: &(), provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        self.runtime(
            provider_module_id,
            ComponentIntegrity::contract_key(),
            Vec::new(),
        )
    }

    fn prepare(
        &self,
        _: &(),
        _: &fabric_component::ComponentRuntimeScope,
    ) -> Result<(), ComponentError> {
        Ok(())
    }
}

fn dependency_provider(id: &str) -> VersionedNotesProvider {
    VersionedNotesProvider::new(id, vec![(dependency_key(), "dependency".to_owned())])
}

fn provided_consumer(
    id: &str,
    capture: Arc<std::sync::Mutex<Option<super::support::CapturedResolution>>>,
) -> NotesConsumer {
    NotesConsumer::new(
        id,
        ContractRequirement::provisional(provided_key().id().clone()),
        capture,
    )
}

fn declaration<'a>(
    built: &'a BuiltFabric,
    contract: &ContractKey<NotesContract>,
) -> &'a ModuleDeclaration {
    built
        .manifest()
        .diagnostics()
        .module_declarations()
        .iter()
        .find(|declaration| {
            declaration
                .provided_contracts()
                .contains(&contract.declaration())
        })
        .expect("augmentation support declaration")
}

fn assert_integrity_declaration(
    declaration: &ModuleDeclaration,
    provider_module_id: &ModuleId,
    augmentation: &ContractKey<NotesContract>,
    base: Option<&fabric_core::ContractRequirementDeclaration>,
) {
    assert_eq!(declaration.module_id(), provider_module_id);
    assert_eq!(
        declaration
            .host_requirement()
            .expect("host requirement")
            .module_id(),
        provider_module_id
    );
    assert!(
        declaration.required_contracts().contains(
            &ContractRequirement::<NotesContract>::provisional(dependency_key().id().clone())
                .declaration()
                .clone()
        )
    );
    if let Some(base) = base {
        assert!(declaration.required_contracts().contains(base));
    }
    assert_eq!(
        declaration.optional_contracts(),
        [
            ContractRequirement::<NotesContract>::provisional(optional_key().id().clone())
                .declaration()
                .clone()
        ]
    );
    assert!(
        declaration
            .provided_contracts()
            .contains(&provided_key().declaration())
    );
    assert!(
        declaration
            .provided_contracts()
            .contains(&augmentation.declaration())
    );
}

#[test]
fn resource_support_declaration_is_merged_bound_and_host_checked() {
    let bound = Arc::new(AtomicUsize::new(0));
    let capture = Arc::new(std::sync::Mutex::new(None));
    let clock = Clock::select("integrity", ClockConfig::default()).expect("clock");
    let attachment =
        ResourceAugmentation::<Clock, ResourceIntegrity>::attach(&clock, ()).expect("attachment");
    let base_requirement = attachment.base_requirement().declaration().clone();
    let supported = attachment.using(IntegritySupport(Arc::clone(&bound)));
    let provider_module_id = supported.provider_module_id().clone();
    let built = Fabric::new("fabric.test.resource-support-integrity")
        .expect("fabric")
        .resource(clock.clone().using(MemoryClock::new(1)).expect("adapter"))
        .resource_augmentation(supported)
        .block("dependency", |block| {
            block.module(dependency_provider("integrity-resource-dependency"))
        })
        .expect("dependency block")
        .block("provided", |block| {
            block.module(provided_consumer(
                "integrity-resource-provided",
                Arc::clone(&capture),
            ))
        })
        .expect("provided block")
        .build()
        .expect("build");
    assert_integrity_declaration(
        declaration(&built, &ResourceIntegrity::contract_key()),
        &provider_module_id,
        &ResourceIntegrity::contract_key(),
        Some(&base_requirement),
    );
    assert!(matches!(
        built.materialize_named_on("resource-missing-host", &HostDescriptor::native()),
        Err(CompositionError::HostIncompatible { .. })
    ));
    let mut instance = built
        .materialize_named_on(
            "resource-compatible-host",
            &HostDescriptor::native()
                .with_facility(HostFacilityId::new(FACILITY).expect("facility")),
        )
        .expect("materialize");
    instance.start().expect("start");
    assert_eq!(bound.load(Ordering::SeqCst), 1);
    assert_eq!(
        capture
            .lock()
            .expect("capture")
            .as_ref()
            .expect("bound")
            .value,
        "provided"
    );
}

#[test]
fn resource_support_dependency_is_a_normal_missing_provider_error() {
    let clock = Clock::select("missing", ClockConfig::default()).expect("clock");
    let error = Fabric::new("fabric.test.resource-support-integrity.missing")
        .expect("fabric")
        .resource(clock.clone().using(MemoryClock::new(1)).expect("adapter"))
        .resource_augmentation(
            ResourceAugmentation::<Clock, ResourceIntegrity>::attach(&clock, ())
                .expect("attachment")
                .using(IntegritySupport(Arc::new(AtomicUsize::new(0)))),
        )
        .build()
        .expect_err("missing support dependency");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::MissingProvider { .. })
    ));
}

#[test]
fn system_and_component_support_declarations_preserve_integrity() {
    let system_bound = Arc::new(AtomicUsize::new(0));
    let system = AdaptedOperations::select(AdaptedOperationsConfig::default()).expect("system");
    let system_attachment =
        SystemAugmentation::<AdaptedOperations, SystemIntegrity>::attach(&system, ())
            .expect("attachment");
    let system_base_requirement = system_attachment.base_requirement().declaration().clone();
    let system_support = system_attachment.using(IntegritySupport(Arc::clone(&system_bound)));
    let system_provider = system_support.provider_module_id().clone();
    let system_built = Fabric::new("fabric.test.system-support-integrity")
        .expect("fabric")
        .system(
            system
                .clone()
                .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
                    value: 1,
                }))
                .expect("adapter"),
        )
        .system_augmentation(system_support)
        .block("dependency", |block| {
            block.module(dependency_provider("integrity-system-dependency"))
        })
        .expect("dependency")
        .build()
        .expect("build");
    assert_integrity_declaration(
        declaration(&system_built, &SystemIntegrity::contract_key()),
        &system_provider,
        &SystemIntegrity::contract_key(),
        Some(&system_base_requirement),
    );
    assert!(matches!(
        system_built.materialize_named_on("system-missing-host", &HostDescriptor::native()),
        Err(CompositionError::HostIncompatible { .. })
    ));
    let mut system_instance = system_built
        .materialize_named_on(
            "system-compatible-host",
            &HostDescriptor::native()
                .with_facility(HostFacilityId::new(FACILITY).expect("facility")),
        )
        .expect("materialize");
    system_instance.start().expect("start");
    assert_eq!(system_bound.load(Ordering::SeqCst), 1);

    let component_bound = Arc::new(AtomicUsize::new(0));
    let component_inspection = Greeter::define(GreeterConfig {})
        .augment::<ComponentIntegrity>(())
        .expect("attachment")
        .using(IntegritySupport(Arc::clone(&component_bound)));
    let component_provider = component_inspection
        .requirement()
        .provider_selection(ModuleId::new("integrity-component-consumer").expect("id"))
        .provider()
        .clone();
    let (_, component_provider_module) = component_inspection.into_parts();
    assert_integrity_declaration(
        &component_provider_module.declaration(),
        &component_provider,
        &ComponentIntegrity::contract_key(),
        None,
    );
    let component_support = Greeter::define(GreeterConfig {})
        .augment::<ComponentIntegrity>(())
        .expect("attachment")
        .using(IntegritySupport(Arc::clone(&component_bound)));
    let component_built = Fabric::new("fabric.test.component-support-integrity")
        .expect("fabric")
        .component(component_support)
        .block("dependency", |block| {
            block.module(dependency_provider("integrity-component-dependency"))
        })
        .expect("dependency")
        .build()
        .expect("build");
    assert!(matches!(
        component_built.materialize_named_on("component-missing-host", &HostDescriptor::native()),
        Err(CompositionError::HostIncompatible { .. })
    ));
    let mut component_instance = component_built
        .materialize_named_on(
            "component-compatible-host",
            &HostDescriptor::native()
                .with_facility(HostFacilityId::new(FACILITY).expect("facility")),
        )
        .expect("materialize");
    component_instance.start().expect("start");
    assert_eq!(component_bound.load(Ordering::SeqCst), 1);
}

#[derive(Clone)]
struct DuplicateXSupport;

impl ResourceAugmentationSupportDefinition<Clock, ResourceIntegrity> for DuplicateXSupport {
    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(provider_module_id)
            .with_provided_contracts(vec![ResourceIntegrity::contract_key().declaration()])
    }

    fn materialize(
        &self,
        _: &ResourceAugmentation<Clock, ResourceIntegrity>,
        _: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        None
    }
}

#[test]
fn support_cannot_redeclare_fabric_owned_x() {
    let clock = Clock::select("duplicate", ClockConfig::default()).expect("clock");
    let error = Fabric::new("fabric.test.resource-support-integrity.duplicate-x")
        .expect("fabric")
        .resource(clock.clone().using(MemoryClock::new(1)).expect("adapter"))
        .resource_augmentation(
            ResourceAugmentation::<Clock, ResourceIntegrity>::attach(&clock, ())
                .expect("attachment")
                .using(DuplicateXSupport),
        )
        .build()
        .expect_err("duplicate X declaration");
    assert!(matches!(
        error,
        FabricBuildError::Composition(
            CompositionError::DuplicateProvidedContractDeclaration { .. }
        )
    ));
}
