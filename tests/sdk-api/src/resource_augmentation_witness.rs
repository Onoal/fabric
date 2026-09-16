use std::sync::{Arc, Mutex};

use fabric::prelude::*;
use fabric_core::{
    ContractId, ContractKey, ContractRequirement, ModuleBindings, ModuleContract, ModuleError,
    ModuleId, ModuleRuntime,
};
use fabric_test_resource_counter::{DirectCounter, DirectCounterConfig};

#[derive(Clone)]
struct AdditionalReadback;

#[derive(Clone)]
struct AdditionalReadbackService {
    target: String,
}

impl AdditionalReadbackService {
    fn target(&self) -> &str {
        &self.target
    }
}

impl ResourceAugmentationDefinition<DirectCounter> for AdditionalReadback {
    type Config = String;
    type Contract = AdditionalReadbackService;

    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.additional-readback").expect("contract id"),
        )
    }

    fn materialize(
        attachment: &ResourceAugmentation<DirectCounter, Self>,
    ) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(AdditionalReadbackRuntime {
            module_id: attachment.module_id().clone(),
            target: attachment.config().clone(),
            base: Requires::provisional(),
        }))
    }
}

struct AdditionalReadbackRuntime {
    module_id: ModuleId,
    target: String,
    base: Requires<DirectCounter>,
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
                target: self.target.clone(),
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
    additional_target: String,
}

#[derive(Clone)]
struct RichConsumer {
    module_id: ModuleId,
    requirement: Arc<ResourceAugmentationRequirement<DirectCounter, AdditionalReadback>>,
    capture: Arc<Mutex<Option<RichObservation>>>,
}

impl RichConsumer {
    fn new(
        requirement: ResourceAugmentationRequirement<DirectCounter, AdditionalReadback>,
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
            additional_target: additional.target().to_owned(),
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
    requirement: fabric_core::ContractRequirement<AdditionalReadbackService>,
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

#[test]
fn external_semantic_attachment_is_occurrence_scoped_and_manifest_truthful() {
    let one = DirectCounter::select("one", DirectCounterConfig { value: 1 }).expect("one");
    let two = DirectCounter::select("two", DirectCounterConfig { value: 2 }).expect("two");
    let attachment =
        ResourceAugmentation::<DirectCounter, AdditionalReadback>::attach(&two, "two".to_owned())
            .expect("attachment");
    let requirement = attachment.require_from(&two).expect("same occurrence");
    let consumer = RichConsumer::new(requirement, Arc::new(Mutex::new(None)));
    let selections = consumer
        .requirement
        .provider_selections(consumer.module_id.clone());
    let capture = Arc::clone(&consumer.capture);
    let expected_base_provider = two.module_id().clone();

    let built = Fabric::new("fabric.test.resource-augmentation")
        .expect("fabric")
        .resource(one)
        .resource(two)
        .resource_augmentation(attachment)
        .block("consumer", |block| block.module(consumer))
        .expect("consumer block")
        .select_provider(selections[0].clone())
        .select_provider(selections[1].clone())
        .build()
        .expect("build");

    let manifest = built.manifest();
    assert_eq!(manifest.resources().len(), 2);
    assert_eq!(manifest.resource_augmentations().len(), 1);
    let entry = &manifest.resource_augmentations()[0];
    assert_eq!(entry.contract_id(), AdditionalReadback::contract_key().id());
    assert_eq!(entry.resource_id(), &DirectCounter::resource_id());
    assert_eq!(entry.resource_name().as_str(), "two");

    let mut instance = built
        .materialize_named("fabric.test.resource-augmentation.instance")
        .expect("materialize");
    instance.start().expect("start");
    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(RichObservation {
            base_provider: expected_base_provider,
            additional_target: "two".to_owned(),
        })
    );
    instance.stop();
}

#[test]
fn attachment_rejects_a_requirement_for_another_resource_occurrence() {
    let one = DirectCounter::select("one", DirectCounterConfig { value: 1 }).expect("one");
    let two = DirectCounter::select("two", DirectCounterConfig { value: 2 }).expect("two");
    let attachment =
        ResourceAugmentation::<DirectCounter, AdditionalReadback>::attach(&two, "two".to_owned())
            .expect("attachment");

    let error = match attachment.require_from(&one) {
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
fn base_only_resource_authoring_and_missing_additional_contract_remain_distinct() {
    let base_only = Fabric::new("fabric.test.resource-augmentation.base-only")
        .expect("fabric")
        .resource(DirectCounter::select("one", DirectCounterConfig { value: 1 }).expect("one"))
        .resource(DirectCounter::select("two", DirectCounterConfig { value: 2 }).expect("two"))
        .build()
        .expect("base-only build");
    assert!(base_only.manifest().resource_augmentations().is_empty());
    let mut instance = base_only
        .materialize_named("fabric.test.resource-augmentation.base-only.instance")
        .expect("base-only materializes");
    instance.start().expect("base-only starts");
    instance.stop();

    let error = Fabric::new("fabric.test.resource-augmentation.missing")
        .expect("fabric")
        .resource(DirectCounter::select("two", DirectCounterConfig { value: 2 }).expect("two"))
        .block("missing", |block| {
            block.module(MissingAdditionalConsumer::new())
        })
        .expect("missing block")
        .build()
        .expect_err("base does not provide external semantic");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::MissingProvider { .. })
    ));
}

#[test]
fn duplicate_attachment_contract_for_one_occurrence_is_rejected_by_existing_core_rules() {
    let target = DirectCounter::select("two", DirectCounterConfig { value: 2 }).expect("target");
    let first = ResourceAugmentation::<DirectCounter, AdditionalReadback>::attach(
        &target,
        "first".to_owned(),
    )
    .expect("first");
    let second = ResourceAugmentation::<DirectCounter, AdditionalReadback>::attach(
        &target,
        "second".to_owned(),
    )
    .expect("second");

    let error = Fabric::new("fabric.test.resource-augmentation.duplicate")
        .expect("fabric")
        .resource(target)
        .resource_augmentation(first)
        .resource_augmentation(second)
        .build()
        .expect_err("duplicate attachment must remain structurally invalid");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::DuplicateModuleId { .. })
    ));
}
