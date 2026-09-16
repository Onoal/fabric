use std::sync::{Arc, Mutex};

use fabric::prelude::*;
use fabric_core::{
    ContractId, ContractKey, ContractRequirement, ModuleBindings, ModuleContract, ModuleError,
    ModuleId, ModuleRuntime,
};
use fabric_test_adapter_clock_memory::MemoryClock;
use fabric_test_resource_clock::{Clock, ClockConfig};

/// An externally owned semantic. It deliberately has no runtime hook and does
/// not name either support implementation below.
#[derive(Clone)]
struct AdditionalReadback;

#[derive(Clone)]
struct AdditionalReadbackService {
    implementation: String,
}

impl AdditionalReadbackService {
    fn implementation(&self) -> &str {
        &self.implementation
    }
}

impl ResourceAugmentationDefinition<Clock> for AdditionalReadback {
    type Config = ();
    type Contract = AdditionalReadbackService;

    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.additional-readback").expect("contract id"),
        )
    }
}

/// Independently authored support. The original `MemoryClock` Adapter and the
/// `AdditionalReadback` semantic both remain unaware of this type.
#[derive(Clone)]
struct FirstReadbackSupport;

#[derive(Clone)]
struct SecondReadbackSupport;

impl ResourceAugmentationSupportDefinition<Clock, AdditionalReadback> for FirstReadbackSupport {
    fn declaration(&self, provider_module_id: ModuleId) -> fabric_core::ModuleDeclaration {
        fabric_core::ModuleDeclaration::new(provider_module_id)
    }

    fn materialize(
        &self,
        attachment: &ResourceAugmentation<Clock, AdditionalReadback>,
        provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        readback_runtime(attachment, provider_module_id, "support-one")
    }
}

impl ResourceAugmentationSupportDefinition<Clock, AdditionalReadback> for SecondReadbackSupport {
    fn declaration(&self, provider_module_id: ModuleId) -> fabric_core::ModuleDeclaration {
        fabric_core::ModuleDeclaration::new(provider_module_id)
    }

    fn materialize(
        &self,
        attachment: &ResourceAugmentation<Clock, AdditionalReadback>,
        provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        readback_runtime(attachment, provider_module_id, "support-two")
    }
}

fn readback_runtime(
    attachment: &ResourceAugmentation<Clock, AdditionalReadback>,
    provider_module_id: ModuleId,
    implementation: &str,
) -> Option<Box<dyn ModuleRuntime>> {
    Some(Box::new(AdditionalReadbackRuntime {
        module_id: provider_module_id,
        implementation: implementation.to_owned(),
        base: attachment.base_requirement(),
    }))
}

struct AdditionalReadbackRuntime {
    module_id: ModuleId,
    implementation: String,
    base: Requires<Clock>,
}

impl ModuleRuntime for AdditionalReadbackRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.base.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &AdditionalReadback::contract_key(),
            Arc::new(AdditionalReadbackService {
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

    fn stop(&mut self) {}

    fn health(&self) -> Health {
        Health::Healthy
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RichObservation {
    base_provider: ModuleId,
    implementation: String,
}

#[derive(Clone)]
struct RichConsumer {
    module_id: ModuleId,
    requirement: Arc<ResourceAugmentationRequirement<Clock, AdditionalReadback>>,
    capture: Arc<Mutex<Option<RichObservation>>>,
}

impl RichConsumer {
    fn new(
        requirement: ResourceAugmentationRequirement<Clock, AdditionalReadback>,
        capture: Arc<Mutex<Option<RichObservation>>>,
    ) -> Self {
        Self {
            module_id: ModuleId::new("fabric.test.additional-readback.consumer")
                .expect("module id"),
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
        let additional = bindings
            .resolve(self.requirement.augmentation())
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.capture.lock().expect("capture lock") = Some(RichObservation {
            base_provider: base.provider().clone(),
            implementation: additional.implementation().to_owned(),
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
struct MissingAdditionalConsumer {
    module_id: ModuleId,
    requirement: ContractRequirement<AdditionalReadbackService>,
}

impl MissingAdditionalConsumer {
    fn new() -> Self {
        Self {
            module_id: ModuleId::new("fabric.test.additional-readback.missing-consumer")
                .expect("module id"),
            requirement: ContractRequirement::provisional(
                AdditionalReadback::contract_key().id().clone(),
            ),
        }
    }
}

impl ModuleRuntime for MissingAdditionalConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
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

    fn stop(&mut self) {}

    fn health(&self) -> Health {
        Health::Healthy
    }
}

fn selected_clock(name: &str) -> ResourceSelection<Clock> {
    Clock::select(name, ClockConfig::default()).expect("clock selection")
}

fn build_supported_clock<S>(
    composition_id: &str,
    support: S,
) -> (BuiltFabric, ModuleId, Arc<Mutex<Option<RichObservation>>>)
where
    S: ResourceAugmentationSupportDefinition<Clock, AdditionalReadback>,
{
    let one = selected_clock("one");
    let two = selected_clock("two");
    let attachment =
        ResourceAugmentation::<Clock, AdditionalReadback>::attach(&two, ()).expect("attachment");
    let supported = attachment.using(support);
    let requirement = supported.require_from(&two).expect("same occurrence");
    let consumer = RichConsumer::new(requirement, Arc::new(Mutex::new(None)));
    let selections = consumer
        .requirement
        .provider_selections(consumer.module_id.clone());
    let capture = Arc::clone(&consumer.capture);
    let expected_base_provider = two.module_id().clone();

    let built = Fabric::new(composition_id)
        .expect("fabric")
        .resource(one.using(MemoryClock::new(1)).expect("one adapter"))
        .resource(two.using(MemoryClock::new(2)).expect("two adapter"))
        .resource_augmentation(supported)
        .block("consumer", |block| block.module(consumer))
        .expect("consumer block")
        .select_provider(selections[0].clone())
        .select_provider(selections[1].clone())
        .build()
        .expect("build");
    (built, expected_base_provider, capture)
}

#[test]
fn external_support_for_an_adapted_resource_is_occurrence_scoped_and_manifest_truthful() {
    let (built, expected_base_provider, capture) =
        build_supported_clock("fabric.test.resource-augmentation", FirstReadbackSupport);

    let manifest = built.manifest();
    assert_eq!(manifest.resources().len(), 2);
    assert_eq!(manifest.resource_augmentations().len(), 1);
    let entry = &manifest.resource_augmentations()[0];
    assert_eq!(entry.contract_id(), AdditionalReadback::contract_key().id());
    assert_eq!(entry.resource_id(), &Clock::resource_id());
    assert_eq!(entry.resource_name().as_str(), "two");

    let mut instance = built
        .materialize_named_on(
            "fabric.test.resource-augmentation.instance",
            &HostDescriptor::native(),
        )
        .expect("materialize");
    instance.start().expect("start");
    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(RichObservation {
            base_provider: expected_base_provider,
            implementation: "support-one".to_owned(),
        })
    );
    instance.stop();
}

#[test]
fn attachment_rejects_a_requirement_for_another_resource_occurrence() {
    let one = selected_clock("one");
    let two = selected_clock("two");
    let supported = ResourceAugmentation::<Clock, AdditionalReadback>::attach(&two, ())
        .expect("attachment")
        .using(FirstReadbackSupport);

    let error = match supported.require_from(&one) {
        Ok(_) => panic!("wrong occurrence must not form a requirement"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        ResourceAugmentationError::TargetMismatch {
            expected_resource_name,
            actual_resource_name,
            ..
        } if expected_resource_name.as_str() == "two" && actual_resource_name.as_str() == "one"
    ));
}

#[test]
fn semantic_attachment_without_support_cannot_satisfy_the_additional_contract() {
    let two = selected_clock("two");
    let attachment =
        ResourceAugmentation::<Clock, AdditionalReadback>::attach(&two, ()).expect("attachment");
    let error = Fabric::new("fabric.test.resource-augmentation.missing")
        .expect("fabric")
        .resource(two.using(MemoryClock::new(2)).expect("adapter"))
        .resource_augmentation(attachment)
        .block("missing", |block| {
            block.module(MissingAdditionalConsumer::new())
        })
        .expect("missing block")
        .build()
        .expect_err("semantic attachment alone does not provide X");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::MissingProvider { .. })
    ));
}

#[test]
fn alternate_supports_preserve_x_identity_and_change_only_implementation() {
    let (first, first_provider, first_capture) = build_supported_clock(
        "fabric.test.resource-augmentation.first",
        FirstReadbackSupport,
    );
    let (second, second_provider, second_capture) = build_supported_clock(
        "fabric.test.resource-augmentation.second",
        SecondReadbackSupport,
    );
    assert_eq!(
        first.manifest().resource_augmentations()[0].contract_id(),
        second.manifest().resource_augmentations()[0].contract_id()
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
        Some(RichObservation {
            base_provider: first_provider,
            implementation: "support-one".to_owned(),
        })
    );
    assert_eq!(
        second_capture.lock().expect("second capture").clone(),
        Some(RichObservation {
            base_provider: second_provider,
            implementation: "support-two".to_owned(),
        })
    );
    first_instance.stop();
    second_instance.stop();
}

#[test]
fn base_only_adapted_resource_authoring_remains_valid() {
    let built = Fabric::new("fabric.test.resource-augmentation.base-only")
        .expect("fabric")
        .resource(
            selected_clock("one")
                .using(MemoryClock::new(1))
                .expect("adapter"),
        )
        .build()
        .expect("base-only build");
    assert!(built.manifest().resource_augmentations().is_empty());
    let mut instance = built
        .materialize_named_on("base-only", &HostDescriptor::native())
        .expect("materialize");
    instance.start().expect("start");
    instance.stop();
}
