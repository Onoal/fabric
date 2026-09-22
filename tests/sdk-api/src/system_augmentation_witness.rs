use std::sync::{Arc, Mutex};

use fabric::authoring::*;
use fabric::prelude::*;
use fabric_core::{
    ContractId, ContractKey, ContractRequirement, ModuleBindings, ModuleContract, ModuleError,
    ModuleId, ModuleRuntime,
};
use fabric_test_system_operations::{
    AdaptedOperations, AdaptedOperationsConfig, FixedOperationsAdapter,
    FixedOperationsAdapterConfig, TestOperations, TestOperationsConfig,
};

/// Externally owned semantic with no concrete support implementation.
#[derive(Clone)]
struct DriftObservation;

#[derive(Clone)]
struct DriftObservationService {
    implementation: String,
}

impl DriftObservationService {
    fn implementation(&self) -> &str {
        &self.implementation
    }
}

impl SystemAugmentationDefinition<AdaptedOperations> for DriftObservation {
    type Config = ();
    type Contract = DriftObservationService;

    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.system.drift-observation").expect("contract id"),
        )
    }
}

/// A separate semantic proves that augmentation support composes with a
/// self-realizing System as well as with an Adapter-realized System.
#[derive(Clone)]
struct NativeObservation;

#[derive(Clone)]
struct NativeObservationService;

impl SystemAugmentationDefinition<TestOperations> for NativeObservation {
    type Config = ();
    type Contract = NativeObservationService;

    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.system.native-observation").expect("contract id"),
        )
    }
}

#[derive(Clone)]
struct NativeObservationSupport;

impl SystemAugmentationSupportDefinition<TestOperations, NativeObservation>
    for NativeObservationSupport
{
    fn declaration(&self, provider_module_id: ModuleId) -> fabric_core::ModuleDeclaration {
        fabric_core::ModuleDeclaration::new(provider_module_id)
    }

    fn materialize(
        &self,
        attachment: &SystemAugmentation<TestOperations, NativeObservation>,
        provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(NativeObservationRuntime {
            module_id: provider_module_id,
            base: attachment.base_requirement(),
        }))
    }
}

struct NativeObservationRuntime {
    module_id: ModuleId,
    base: SystemRequires<TestOperations>,
}

impl ModuleRuntime for NativeObservationRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }
    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.base.declaration().clone()]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &NativeObservation::contract_key(),
            Arc::new(NativeObservationService),
        )])
    }
    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.base
            .resolve(bindings)
            .map(|_| ())
            .map_err(|error| ModuleError::new(error.to_string()))
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

/// First independently authored support implementation.
#[derive(Clone)]
struct FirstDriftSupport;

/// Second independently authored support implementation for the same X.
#[derive(Clone)]
struct SecondDriftSupport;

impl SystemAugmentationSupportDefinition<AdaptedOperations, DriftObservation>
    for FirstDriftSupport
{
    fn declaration(&self, provider_module_id: ModuleId) -> fabric_core::ModuleDeclaration {
        fabric_core::ModuleDeclaration::new(provider_module_id)
    }

    fn materialize(
        &self,
        attachment: &SystemAugmentation<AdaptedOperations, DriftObservation>,
        provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        drift_runtime(attachment, provider_module_id, "support-one")
    }
}

impl SystemAugmentationSupportDefinition<AdaptedOperations, DriftObservation>
    for SecondDriftSupport
{
    fn declaration(&self, provider_module_id: ModuleId) -> fabric_core::ModuleDeclaration {
        fabric_core::ModuleDeclaration::new(provider_module_id)
    }

    fn materialize(
        &self,
        attachment: &SystemAugmentation<AdaptedOperations, DriftObservation>,
        provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        drift_runtime(attachment, provider_module_id, "support-two")
    }
}

fn drift_runtime(
    attachment: &SystemAugmentation<AdaptedOperations, DriftObservation>,
    provider_module_id: ModuleId,
    implementation: &str,
) -> Option<Box<dyn ModuleRuntime>> {
    Some(Box::new(DriftSupportRuntime {
        module_id: provider_module_id,
        implementation: implementation.to_owned(),
        base: attachment.base_requirement(),
    }))
}

struct DriftSupportRuntime {
    module_id: ModuleId,
    implementation: String,
    base: SystemRequires<AdaptedOperations>,
}

impl ModuleRuntime for DriftSupportRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }
    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.base.declaration().clone()]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &DriftObservation::contract_key(),
            Arc::new(DriftObservationService {
                implementation: self.implementation.clone(),
            }),
        )])
    }
    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.base
            .resolve(bindings)
            .map(|_| ())
            .map_err(|error| ModuleError::new(error.to_string()))
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
struct Observation {
    base_provider: ModuleId,
    implementation: String,
}

#[derive(Clone)]
struct RichConsumer {
    module_id: ModuleId,
    requirement: Arc<SystemAugmentationRequirement<AdaptedOperations, DriftObservation>>,
    capture: Arc<Mutex<Option<Observation>>>,
}

impl RichConsumer {
    fn new(
        requirement: SystemAugmentationRequirement<AdaptedOperations, DriftObservation>,
        capture: Arc<Mutex<Option<Observation>>>,
    ) -> Self {
        Self {
            module_id: ModuleId::new("fabric.test.system-drift.consumer").expect("module id"),
            requirement: Arc::new(requirement),
            capture,
        }
    }
}

impl ModuleRuntime for RichConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }
    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![
            self.requirement.base().declaration().clone(),
            self.requirement.augmentation().declaration().clone(),
        ]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }
    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let base = self
            .requirement
            .base()
            .resolve_with_provider(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        let augmentation = bindings
            .resolve(self.requirement.augmentation())
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.capture.lock().expect("capture lock") = Some(Observation {
            base_provider: base.provider().clone(),
            implementation: augmentation.implementation().to_owned(),
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

#[derive(Clone)]
struct MissingSupportConsumer {
    module_id: ModuleId,
    requirement: ContractRequirement<DriftObservationService>,
}

impl MissingSupportConsumer {
    fn new() -> Self {
        Self {
            module_id: ModuleId::new("fabric.test.system-drift.missing").expect("module id"),
            requirement: ContractRequirement::provisional(
                DriftObservation::contract_key().id().clone(),
            ),
        }
    }
}

impl ModuleRuntime for MissingSupportConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }
    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }
    fn bind(&mut self, _: &ModuleBindings) -> Result<(), ModuleError> {
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

fn selected_system() -> SystemSelection<AdaptedOperations> {
    AdaptedOperations::select(AdaptedOperationsConfig::default()).expect("system")
}

fn build_supported_system<P>(
    composition_id: &str,
    support: P,
) -> (BuiltFabric, ModuleId, Arc<Mutex<Option<Observation>>>)
where
    P: SystemAugmentationSupportDefinition<AdaptedOperations, DriftObservation>,
{
    let system = selected_system();
    let expected_base_provider = system.module_id().clone();
    let attachment = SystemAugmentation::<AdaptedOperations, DriftObservation>::attach(&system, ())
        .expect("attachment");
    let supported = attachment.using(support);
    let requirement = supported
        .require_from(&system)
        .expect("same System occurrence");
    let consumer = RichConsumer::new(requirement, Arc::new(Mutex::new(None)));
    let selections = consumer
        .requirement
        .provider_selections(consumer.module_id.clone());
    let capture = Arc::clone(&consumer.capture);
    let built = Fabric::new(composition_id)
        .expect("fabric")
        .system(
            system
                .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
                    value: 7,
                }))
                .expect("adapter"),
        )
        .system_augmentation(supported)
        .block("consumer", |block| block.module(consumer))
        .expect("consumer")
        .select_provider(selections[0].clone())
        .select_provider(selections[1].clone())
        .build()
        .expect("build");
    (built, expected_base_provider, capture)
}

#[test]
fn external_support_for_an_adapted_system_is_semantic_and_manifest_truthful() {
    let (built, expected_base_provider, capture) =
        build_supported_system("fabric.test.system-augmentation", FirstDriftSupport);
    let manifest = built.manifest();
    assert_eq!(manifest.systems().len(), 1);
    assert_eq!(manifest.system_augmentations().len(), 1);
    let entry = &manifest.system_augmentations()[0];
    assert_eq!(entry.contract_id(), DriftObservation::contract_key().id());
    assert_eq!(entry.system_id(), &AdaptedOperations::system_id());

    let mut instance = built
        .materialize_named_on("system-augmentation", &HostDescriptor::native())
        .expect("materialize");
    instance.start().expect("start");
    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(Observation {
            base_provider: expected_base_provider,
            implementation: "support-one".to_owned(),
        })
    );
    instance.stop().expect("stop instance");
}

#[test]
fn semantic_attachment_without_support_cannot_satisfy_x() {
    let system = selected_system();
    let attachment = SystemAugmentation::<AdaptedOperations, DriftObservation>::attach(&system, ())
        .expect("attachment");
    let error = Fabric::new("fabric.test.system-augmentation.missing")
        .expect("fabric")
        .system(
            system
                .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
                    value: 7,
                }))
                .expect("adapter"),
        )
        .system_augmentation(attachment)
        .block("missing", |block| {
            block.module(MissingSupportConsumer::new())
        })
        .expect("missing")
        .build()
        .expect_err("attachment does not provide X");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::MissingProvider { .. })
    ));
}

#[test]
fn alternate_supports_preserve_x_identity_and_change_only_implementation() {
    let (first, first_provider, first_capture) =
        build_supported_system("fabric.test.system-augmentation.first", FirstDriftSupport);
    let (second, second_provider, second_capture) =
        build_supported_system("fabric.test.system-augmentation.second", SecondDriftSupport);
    assert_eq!(
        first.manifest().system_augmentations()[0].contract_id(),
        second.manifest().system_augmentations()[0].contract_id()
    );
    let host = HostDescriptor::native();
    let mut first_instance = first
        .materialize_named_on("first", &host)
        .expect("first materialize");
    let mut second_instance = second
        .materialize_named_on("second", &host)
        .expect("second materialize");
    first_instance.start().expect("first start");
    second_instance.start().expect("second start");
    assert_eq!(
        first_capture.lock().expect("first capture").clone(),
        Some(Observation {
            base_provider: first_provider,
            implementation: "support-one".to_owned()
        })
    );
    assert_eq!(
        second_capture.lock().expect("second capture").clone(),
        Some(Observation {
            base_provider: second_provider,
            implementation: "support-two".to_owned()
        })
    );
    first_instance.stop().expect("stop instance");
    second_instance.stop().expect("stop instance");
}

#[test]
fn base_only_adapted_system_authoring_remains_valid() {
    let built = Fabric::new("fabric.test.system-augmentation.base-only")
        .expect("fabric")
        .system(
            selected_system()
                .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
                    value: 7,
                }))
                .expect("adapter"),
        )
        .build()
        .expect("base-only build");
    assert!(built.manifest().system_augmentations().is_empty());
    let mut instance = built
        .materialize_named_on("base-only", &HostDescriptor::native())
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop instance");
}

#[test]
fn self_realizing_system_remains_compatible_with_external_augmentation_support() {
    let system = TestOperations::select(TestOperationsConfig::new(1, 1)).expect("system");
    let augmentation = SystemAugmentation::<TestOperations, NativeObservation>::attach(&system, ())
        .expect("attachment")
        .using(NativeObservationSupport);
    let built = Fabric::new("fabric.test.system-augmentation.native")
        .expect("fabric")
        .system(system)
        .system_augmentation(augmentation)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_named_on("native", &HostDescriptor::native())
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop instance");
}
