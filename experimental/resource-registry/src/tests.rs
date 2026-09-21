use std::sync::{Arc, Mutex};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionError, CompositionId, ContractId,
    ContractKey, ContractRequirement, Health, InstanceError, InstanceId, ModuleBindings,
    ModuleContract, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_resource::ResourceId;

use crate::{
    ResourceConfiguration, ResourceConfigurationFacet, ResourceConfigurationKind,
    ResourceDescriptor, ResourceInspection, ResourceInspectionEntry, ResourceInspectionFacet,
    ResourceRegistry, ResourceRegistryModule, resource_configuration_facet_id,
    resource_inspection_facet_id,
};

type CapturedResourceRegistry = Arc<Mutex<Option<Arc<ResourceRegistry>>>>;
type ConsumedAlpha = Arc<Mutex<Option<u16>>>;
type ConsumedGamma = Arc<Mutex<Option<String>>>;

#[derive(Debug)]
enum TestStartError {
    #[allow(dead_code)]
    Composition(CompositionError),
    Instance(InstanceError),
}

impl From<CompositionError> for TestStartError {
    fn from(value: CompositionError) -> Self {
        Self::Composition(value)
    }
}

impl From<InstanceError> for TestStartError {
    fn from(value: InstanceError) -> Self {
        Self::Instance(value)
    }
}

fn start_test_instance(
    composition_id: &str,
    composition: fabric_core::Composition,
) -> Result<fabric_core::Instance, TestStartError> {
    let mut instance = composition
        .materialize(InstanceId::new(composition_id.to_owned()).expect("test instance id"))?;
    instance.start()?;
    Ok(instance)
}

#[derive(Clone, Debug)]
struct AlphaConfig {
    shards: u16,
}

#[derive(Clone, Debug)]
struct GammaConfig {
    replicas: Vec<String>,
    durable: bool,
}

#[derive(Clone)]
struct CapabilityCaptureModule {
    module_id: ModuleId,
    requirement: ContractRequirement<ResourceRegistry>,
    captured: CapturedResourceRegistry,
}

impl CapabilityCaptureModule {
    fn new(captured: CapturedResourceRegistry) -> Self {
        Self {
            module_id: ModuleId::new("test.resource.registry.capture").expect("module id"),
            requirement: ContractRequirement::provisional(crate::resource_registry_contract_id()),
            captured,
        }
    }
}

impl ModuleRuntime for CapabilityCaptureModule {
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
        vec![self.requirement.id().clone()]
            .into_iter()
            .map(fabric_core::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let resource_registry = bindings
            .resolve(&self.requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.captured.lock().expect("capture lock") = Some(resource_registry.clone());
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
struct SyntheticProviderModule {
    module_id: ModuleId,
    provided_contracts: Vec<ContractId>,
}

impl SyntheticProviderModule {
    fn new(module_id: &str, provided_contracts: Vec<ContractId>) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            provided_contracts,
        }
    }
}

impl ModuleRuntime for SyntheticProviderModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        self.provided_contracts
            .clone()
            .into_iter()
            .map(fabric_core::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        Vec::new()
            .into_iter()
            .map(fabric_core::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(self
            .provided_contracts
            .iter()
            .map(|contract_id| {
                let key = ContractKey::<()>::provisional(contract_id.clone());
                ModuleContract::new(&key, Arc::new(()))
            })
            .collect())
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
struct SyntheticCapabilityModule {
    module_id: ModuleId,
    provided_contracts: Vec<ContractId>,
    required_contracts: Vec<ContractId>,
    optional_core_contracts: Vec<ContractId>,
    descriptor: ResourceDescriptor,
    requirement: ContractRequirement<ResourceRegistry>,
    resource_registry: Option<ResourceRegistry>,
}

impl SyntheticCapabilityModule {
    fn new(
        module_id: &str,
        _resource_id: ResourceId,
        provided_contracts: Vec<ContractId>,
        required_contracts: Vec<ContractId>,
        optional_core_contracts: Vec<ContractId>,
        descriptor: ResourceDescriptor,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            provided_contracts,
            required_contracts,
            optional_core_contracts,
            descriptor,
            requirement: ContractRequirement::provisional(crate::resource_registry_contract_id()),
            resource_registry: None,
        }
    }
}

impl ModuleRuntime for SyntheticCapabilityModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        self.provided_contracts
            .clone()
            .into_iter()
            .map(fabric_core::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        self.required_contracts
            .clone()
            .into_iter()
            .map(fabric_core::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn optional_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        let mut contracts = self.optional_core_contracts.clone();
        contracts.push(self.requirement.id().clone());
        contracts
            .into_iter()
            .map(fabric_core::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(self
            .provided_contracts
            .iter()
            .cloned()
            .map(|contract_id| {
                let key = ContractKey::<()>::provisional(contract_id);
                ModuleContract::new(&key, Arc::new(()))
            })
            .collect())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.resource_registry = bindings
            .resolve_optional(&self.requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?
            .as_deref()
            .cloned();
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        if let Some(resource_registry) = &self.resource_registry {
            resource_registry
                .register(self, self.descriptor.clone())
                .map_err(|error| ModuleError::new(error.to_string()))?;
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        if let Some(resource_registry) = &self.resource_registry {
            let _ = resource_registry.unregister(&self.module_id);
        }
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

#[test]
fn synthetic_capabilities_participate_without_closed_catalog() {
    let capture = Arc::new(Mutex::new(None));
    let composition = CompositionBuilder::new(
        CompositionId::new("test.resource.registry.synthetic".to_owned()).expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("resource-registry".to_owned()).expect("block id"))
            .register_module(ResourceRegistryModule::new())
            .register_module(SyntheticProviderModule::new(
                "test.shared.provider",
                vec![
                    ContractId::new("test.shared.interface").expect("contract id"),
                    ContractId::new("test.gamma.requirement").expect("contract id"),
                ],
            ))
            .register_module(SyntheticCapabilityModule::new(
                "test.alpha.module",
                ResourceId::new("alpha").expect("resource id"),
                vec![ContractId::new("test.alpha.interface").expect("contract id")],
                Vec::new(),
                Vec::new(),
                ResourceDescriptor::new(ResourceId::new("alpha").expect("resource id")),
            ))
            .register_module(SyntheticCapabilityModule::new(
                "test.beta.module",
                ResourceId::new("beta").expect("resource id"),
                Vec::new(),
                vec![ContractId::new("test.shared.interface").expect("contract id")],
                Vec::new(),
                ResourceDescriptor::new(ResourceId::new("beta").expect("resource id")),
            ))
            .register_module(SyntheticCapabilityModule::new(
                "test.gamma.module",
                ResourceId::new("gamma").expect("resource id"),
                Vec::new(),
                Vec::new(),
                vec![ContractId::new("test.shared.optional").expect("contract id")],
                ResourceDescriptor::new(ResourceId::new("gamma").expect("resource id")),
            ))
            .register_module(CapabilityCaptureModule::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");

    let _instance = start_test_instance("test.resource.registry.synthetic", composition)
        .expect("start instance");
    let shell = capture_resource_registry(&capture);
    let capabilities = shell.resources();
    assert_eq!(capabilities.len(), 3);
    assert_eq!(capabilities[0].resource_id().as_str(), "alpha");
    assert_eq!(capabilities[1].resource_id().as_str(), "beta");
    assert_eq!(capabilities[2].resource_id().as_str(), "gamma");
    assert_eq!(
        capabilities[0].provided_contracts()[0].as_str(),
        "test.alpha.interface"
    );
    assert_eq!(
        capabilities[1].required_contracts()[0].as_str(),
        "test.shared.interface"
    );
    assert_eq!(
        capabilities[2].optional_contracts()[0].as_str(),
        "test.shared.optional"
    );
}

#[test]
fn duplicate_capability_participation_is_rejected_deterministically() {
    let composition = CompositionBuilder::new(
        CompositionId::new("test.resource.registry.duplicate".to_owned()).expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("resource-registry".to_owned()).expect("block id"))
            .register_module(ResourceRegistryModule::new())
            .register_module(SyntheticCapabilityModule::new(
                "test.alpha.one",
                ResourceId::new("alpha").expect("resource id"),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                ResourceDescriptor::new(ResourceId::new("alpha").expect("resource id")),
            ))
            .register_module(SyntheticCapabilityModule::new(
                "test.alpha.two",
                ResourceId::new("alpha").expect("resource id"),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                ResourceDescriptor::new(ResourceId::new("alpha").expect("resource id")),
            ))
            .build(),
    )
    .build()
    .expect("composition");
    let error = start_test_instance("test.resource.registry.duplicate", composition)
        .expect_err("duplicate capability participation must fail");

    assert!(matches!(
        error,
        TestStartError::Instance(InstanceError::ModuleFailure { ref module_id, ref source, .. })
            if module_id.as_str() == "test.alpha.two"
                && source
                    .to_string()
                    .contains("resource `alpha` is already participating through module `test.alpha.one`")
    ));
}

#[test]
fn resource_interface_claim_cannot_diverge_from_core_module_contract_truth() {
    let composition = CompositionBuilder::new(
        CompositionId::new("test.resource.registry.interface-mismatch".to_owned())
            .expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("resource-registry".to_owned()).expect("block id"))
            .register_module(ResourceRegistryModule::new())
            .register_module(SyntheticCapabilityModule::new(
                "test.alpha.module",
                ResourceId::new("alpha").expect("resource id"),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                ResourceDescriptor::new(ResourceId::new("alpha").expect("resource id"))
                    .provides(ContractId::new("test.false.provided").expect("contract id")),
            ))
            .build(),
    )
    .build()
    .expect("composition");
    let error = start_test_instance("test.resource.registry.interface-mismatch", composition)
        .expect_err("divergent resource interface claim must fail");

    assert!(matches!(
        error,
        TestStartError::Instance(InstanceError::ModuleFailure { ref module_id, ref source, .. })
            if module_id.as_str() == "test.alpha.module"
                && source
                    .to_string()
                    .contains("claimed interface `test.false.provided` with role `provides`")
    ));
}

#[test]
fn heterogeneous_capabilities_consume_configuration_through_one_resource_registry_boundary() {
    let capture = Arc::new(Mutex::new(None));
    let alpha_id = ResourceId::new("alpha").expect("resource id");
    let gamma_id = ResourceId::new("gamma").expect("resource id");
    let alpha_consumed: ConsumedAlpha = Arc::new(Mutex::new(None));
    let gamma_consumed: ConsumedGamma = Arc::new(Mutex::new(None));
    let composition = CompositionBuilder::new(
        CompositionId::new("test.resource.registry.configuration".to_owned())
            .expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("resource-registry".to_owned()).expect("block id"))
            .register_module(ResourceRegistryModule::new())
            .register_module(SyntheticProviderModule::new(
                "test.shared.provider",
                vec![ContractId::new("test.gamma.requirement").expect("contract id")],
            ))
            .register_module(SyntheticCapabilityModule::new(
                "test.alpha.module",
                alpha_id.clone(),
                vec![ContractId::new("test.alpha.interface").expect("contract id")],
                Vec::new(),
                Vec::new(),
                ResourceDescriptor::new(alpha_id.clone())
                    .with_configuration({
                        let alpha_consumed = Arc::clone(&alpha_consumed);
                        ResourceConfigurationFacet::typed_with::<AlphaConfig>(
                            ResourceConfigurationKind::new("alpha.box")
                                .expect("configuration kind"),
                            move |config| {
                                if config.shards == 0 {
                                    return Err(
                                        crate::ResourceRegistryError::ConfigurationRejected {
                                            message: "alpha shards must be non-zero".to_owned(),
                                        },
                                    );
                                }
                                *alpha_consumed.lock().expect("alpha consumed") =
                                    Some(config.shards);
                                Ok(())
                            },
                        )
                    })
                    .with_inspection({
                        let alpha_consumed = Arc::clone(&alpha_consumed);
                        ResourceInspectionFacet::new(move || {
                            let shards = alpha_consumed
                                .lock()
                                .expect("alpha consumed")
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "unconsumed".to_owned());
                            ResourceInspection::new(vec![
                                ResourceInspectionEntry::public("mode", "alpha")?,
                                ResourceInspectionEntry::public("consumed_shards", shards)?,
                            ])
                        })
                    }),
            ))
            .register_module(SyntheticCapabilityModule::new(
                "test.gamma.module",
                gamma_id.clone(),
                Vec::new(),
                vec![ContractId::new("test.gamma.requirement").expect("contract id")],
                Vec::new(),
                ResourceDescriptor::new(gamma_id.clone())
                    .with_configuration({
                        let gamma_consumed = Arc::clone(&gamma_consumed);
                        ResourceConfigurationFacet::typed_with::<GammaConfig>(
                            ResourceConfigurationKind::new("gamma.box")
                                .expect("configuration kind"),
                            move |config| {
                                if config.replicas.is_empty() {
                                    return Err(
                                        crate::ResourceRegistryError::ConfigurationRejected {
                                            message: "gamma replicas must not be empty".to_owned(),
                                        },
                                    );
                                }
                                if !config.durable {
                                    return Err(
                                        crate::ResourceRegistryError::ConfigurationRejected {
                                            message: "gamma must participate durably".to_owned(),
                                        },
                                    );
                                }
                                *gamma_consumed.lock().expect("gamma consumed") = Some(format!(
                                    "{}:{}",
                                    config.replicas.len(),
                                    if config.durable {
                                        "durable"
                                    } else {
                                        "ephemeral"
                                    }
                                ));
                                Ok(())
                            },
                        )
                    })
                    .with_inspection({
                        let gamma_consumed = Arc::clone(&gamma_consumed);
                        ResourceInspectionFacet::new(move || {
                            let state = gamma_consumed
                                .lock()
                                .expect("gamma consumed")
                                .clone()
                                .unwrap_or_else(|| "unconsumed".to_owned());
                            ResourceInspection::new(vec![
                                ResourceInspectionEntry::public("mode", "gamma")?,
                                ResourceInspectionEntry::public("consumed_topology", state)?,
                            ])
                        })
                    }),
            ))
            .register_module(CapabilityCaptureModule::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");

    let _instance = start_test_instance("test.resource.registry.configuration", composition)
        .expect("start instance");
    let shell = capture_resource_registry(&capture);
    let alpha = shell.resource(&alpha_id).expect("alpha capability");
    let gamma = shell.resource(&gamma_id).expect("gamma capability");
    assert!(alpha.supports_facet(&resource_configuration_facet_id()));
    assert!(alpha.supports_facet(&resource_inspection_facet_id()));
    assert!(gamma.supports_facet(&resource_configuration_facet_id()));
    assert!(gamma.supports_facet(&resource_inspection_facet_id()));

    shell
        .consume_configuration(
            &alpha_id,
            &ResourceConfiguration::new(
                ResourceConfigurationKind::new("alpha.box").expect("configuration kind"),
                AlphaConfig { shards: 3 },
            ),
        )
        .expect("alpha configuration consumption");
    shell
        .consume_configuration(
            &gamma_id,
            &ResourceConfiguration::new(
                ResourceConfigurationKind::new("gamma.box").expect("configuration kind"),
                GammaConfig {
                    replicas: vec!["eu".to_owned(), "us".to_owned()],
                    durable: true,
                },
            ),
        )
        .expect("gamma configuration consumption");

    assert_eq!(*alpha_consumed.lock().expect("alpha consumed"), Some(3));
    assert_eq!(
        gamma_consumed.lock().expect("gamma consumed").as_deref(),
        Some("2:durable")
    );

    let mismatch = shell
        .consume_configuration(
            &alpha_id,
            &ResourceConfiguration::new(
                ResourceConfigurationKind::new("gamma.box").expect("configuration kind"),
                GammaConfig {
                    replicas: vec!["eu".to_owned()],
                    durable: true,
                },
            ),
        )
        .expect_err("mismatched resource configuration must fail");
    assert!(
        mismatch
            .to_string()
            .contains("does not match declared envelope `alpha.box`")
    );

    let alpha_inspection = shell.inspect(&alpha_id).expect("alpha inspection");
    let gamma_inspection = shell.inspect(&gamma_id).expect("gamma inspection");
    assert_eq!(alpha_inspection.entries()[1].public_value(), Some("alpha"));
    assert_eq!(alpha_inspection.entries()[0].public_value(), Some("3"));
    assert_eq!(
        gamma_inspection.entries()[0].public_value(),
        Some("2:durable")
    );
    assert_eq!(gamma_inspection.entries()[1].public_value(), Some("gamma"));
}

fn capture_resource_registry(capture: &CapturedResourceRegistry) -> Arc<ResourceRegistry> {
    capture
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured resource registry")
}

#[test]
fn configuration_debug_exposes_only_kind() {
    let configuration = ResourceConfiguration::new(
        ResourceConfigurationKind::new("alpha.box").expect("configuration kind"),
        "SUPER_SECRET_CONFIGURATION_VALUE".to_owned(),
    );

    let debug = format!("{configuration:?}");
    assert!(debug.contains("alpha.box"));
    assert!(!debug.contains("SUPER_SECRET_CONFIGURATION_VALUE"));
}

#[test]
fn successful_configuration_consumption_invokes_resource_consumer() {
    let resource_id = ResourceId::new("alpha").expect("resource id");
    let capture = Arc::new(Mutex::new(None));
    let consume_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let effective_state = Arc::new(Mutex::new(None));
    let composition = CompositionBuilder::new(
        CompositionId::new("test.resource.registry.config-success".to_owned())
            .expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("resource-registry".to_owned()).expect("block id"))
            .register_module(ResourceRegistryModule::new())
            .register_module(SyntheticCapabilityModule::new(
                "test.alpha.module",
                resource_id.clone(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                ResourceDescriptor::new(resource_id.clone()).with_configuration({
                    let consume_calls = Arc::clone(&consume_calls);
                    let effective_state = Arc::clone(&effective_state);
                    ResourceConfigurationFacet::typed_with::<AlphaConfig>(
                        ResourceConfigurationKind::new("alpha.box").expect("configuration kind"),
                        move |config| {
                            consume_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            *effective_state.lock().expect("effective state") = Some(config.shards);
                            Ok(())
                        },
                    )
                }),
            ))
            .register_module(CapabilityCaptureModule::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");

    let _instance = start_test_instance("test.resource.registry.config-success", composition)
        .expect("start instance");
    let shell = capture_resource_registry(&capture);
    shell
        .consume_configuration(
            &resource_id,
            &ResourceConfiguration::new(
                ResourceConfigurationKind::new("alpha.box").expect("configuration kind"),
                AlphaConfig { shards: 3 },
            ),
        )
        .expect("valid config");

    assert_eq!(consume_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(*effective_state.lock().expect("effective state"), Some(3));
}

#[test]
fn wrong_kind_and_wrong_payload_do_not_invoke_resource_consumer() {
    let resource_id = ResourceId::new("alpha").expect("resource id");
    let capture = Arc::new(Mutex::new(None));
    let consume_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let composition = CompositionBuilder::new(
        CompositionId::new("test.resource.registry.wrong-config".to_owned())
            .expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("resource-registry".to_owned()).expect("block id"))
            .register_module(ResourceRegistryModule::new())
            .register_module(SyntheticCapabilityModule::new(
                "test.alpha.module",
                resource_id.clone(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                ResourceDescriptor::new(resource_id.clone()).with_configuration({
                    let consume_calls = Arc::clone(&consume_calls);
                    ResourceConfigurationFacet::typed_with::<AlphaConfig>(
                        ResourceConfigurationKind::new("alpha.box").expect("configuration kind"),
                        move |_config| {
                            consume_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            Ok(())
                        },
                    )
                }),
            ))
            .register_module(CapabilityCaptureModule::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");

    let _instance = start_test_instance("test.resource.registry.wrong-config", composition)
        .expect("start instance");
    let shell = capture_resource_registry(&capture);

    let wrong_kind = shell
        .consume_configuration(
            &resource_id,
            &ResourceConfiguration::new(
                ResourceConfigurationKind::new("gamma.box").expect("configuration kind"),
                AlphaConfig { shards: 3 },
            ),
        )
        .expect_err("wrong kind must fail");
    assert!(matches!(
        wrong_kind,
        crate::ResourceRegistryError::ConfigurationRejected { .. }
    ));

    let wrong_payload = shell
        .consume_configuration(
            &resource_id,
            &ResourceConfiguration::new(
                ResourceConfigurationKind::new("alpha.box").expect("configuration kind"),
                GammaConfig {
                    replicas: vec!["eu".to_owned()],
                    durable: true,
                },
            ),
        )
        .expect_err("wrong payload must fail");
    assert!(matches!(
        wrong_payload,
        crate::ResourceRegistryError::ConfigurationRejected { .. }
    ));

    assert_eq!(consume_calls.load(std::sync::atomic::Ordering::SeqCst), 0);
}

#[test]
fn configuration_rejection_is_atomic_and_replay_is_idempotent() {
    let resource_id = ResourceId::new("alpha").expect("resource id");
    let capture = Arc::new(Mutex::new(None));
    let effective_state: Arc<Mutex<Option<u16>>> = Arc::new(Mutex::new(None));
    let composition = CompositionBuilder::new(
        CompositionId::new("test.resource.registry.atomic-config".to_owned())
            .expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("resource-registry".to_owned()).expect("block id"))
            .register_module(ResourceRegistryModule::new())
            .register_module(SyntheticCapabilityModule::new(
                "test.alpha.module",
                resource_id.clone(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                ResourceDescriptor::new(resource_id.clone()).with_configuration({
                    let effective_state = Arc::clone(&effective_state);
                    ResourceConfigurationFacet::typed_with::<AlphaConfig>(
                        ResourceConfigurationKind::new("alpha.box").expect("configuration kind"),
                        move |config| {
                            let mut state = effective_state.lock().expect("effective state");
                            match *state {
                                Some(existing) if existing != config.shards => {
                                    Err(crate::ResourceRegistryError::ConfigurationRejected {
                                        message: "alpha configuration change is not supported"
                                            .to_owned(),
                                    })
                                }
                                _ => {
                                    *state = Some(config.shards);
                                    Ok(())
                                }
                            }
                        },
                    )
                }),
            ))
            .register_module(CapabilityCaptureModule::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");

    let _instance = start_test_instance("test.resource.registry.atomic-config", composition)
        .expect("start instance");
    let shell = capture_resource_registry(&capture);
    let config_a = ResourceConfiguration::new(
        ResourceConfigurationKind::new("alpha.box").expect("configuration kind"),
        AlphaConfig { shards: 3 },
    );

    shell
        .consume_configuration(&resource_id, &config_a)
        .expect("initial config");
    assert_eq!(*effective_state.lock().expect("effective state"), Some(3));

    shell
        .consume_configuration(&resource_id, &config_a)
        .expect("replay config");
    assert_eq!(*effective_state.lock().expect("effective state"), Some(3));

    shell
        .consume_configuration(
            &resource_id,
            &ResourceConfiguration::new(
                ResourceConfigurationKind::new("alpha.box").expect("configuration kind"),
                AlphaConfig { shards: 7 },
            ),
        )
        .expect_err("mutating config must fail");
    assert_eq!(*effective_state.lock().expect("effective state"), Some(3));
}

#[test]
fn inspection_is_sorted_redaction_safe_and_rejects_duplicate_keys() {
    let inspection = ResourceInspection::new(vec![
        ResourceInspectionEntry::public("zeta", "visible").expect("public entry"),
        ResourceInspectionEntry::redacted("secret").expect("redacted entry"),
        ResourceInspectionEntry::public("alpha", "first").expect("public entry"),
    ])
    .expect("inspection");

    assert_eq!(inspection.entries()[0].key(), "alpha");
    assert_eq!(inspection.entries()[1].key(), "secret");
    assert!(inspection.entries()[1].is_redacted());
    assert_eq!(inspection.entries()[1].public_value(), None);
    assert_eq!(inspection.entries()[2].key(), "zeta");
    assert!(!format!("{inspection:?}").contains("SUPER_SECRET_INSPECTION_VALUE"));

    let duplicate = ResourceInspection::new(vec![
        ResourceInspectionEntry::public("alpha", "one").expect("public entry"),
        ResourceInspectionEntry::public("alpha", "two").expect("public entry"),
    ])
    .expect_err("duplicate inspection keys must fail");
    assert!(
        duplicate
            .to_string()
            .contains("duplicate resource inspection key")
    );
}

#[test]
fn redacted_inspection_values_do_not_expose_secret_text() {
    let sentinel = "SUPER_SECRET_INSPECTION_VALUE";
    let inspection = ResourceInspection::new(vec![
        ResourceInspectionEntry::redacted("secret").expect("redacted entry"),
        ResourceInspectionEntry::public("summary", "public").expect("public entry"),
    ])
    .expect("inspection");

    let debug = format!("{inspection:?}");
    assert!(!debug.contains(sentinel));
    assert!(inspection.entries()[0].is_redacted());
    assert_eq!(inspection.entries()[0].public_value(), None);
    assert_eq!(inspection.entries()[1].public_value(), Some("public"));
}

#[test]
fn shell_inspection_callbacks_run_without_holding_registry_locks() {
    let resource_id = ResourceId::new("alpha").expect("resource id");
    let capture = Arc::new(Mutex::new(None));
    let composition = CompositionBuilder::new(
        CompositionId::new("test.resource.registry.inspect-reentrant".to_owned())
            .expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("resource-registry".to_owned()).expect("block id"))
            .register_module(ResourceRegistryModule::new())
            .register_module(SyntheticCapabilityModule::new(
                "test.alpha.module",
                resource_id.clone(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                ResourceDescriptor::new(resource_id.clone()).with_inspection({
                    let capture = Arc::clone(&capture);
                    let resource_id = resource_id.clone();
                    ResourceInspectionFacet::new(move || {
                        let shell = capture_resource_registry(&capture);
                        let descriptor = shell.resource(&resource_id)?;
                        ResourceInspection::new(vec![ResourceInspectionEntry::public(
                            "module",
                            descriptor.module_id().as_str(),
                        )?])
                    })
                }),
            ))
            .register_module(CapabilityCaptureModule::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");

    let _instance = start_test_instance("test.resource.registry.inspect-reentrant", composition)
        .expect("start instance");
    let shell = capture_resource_registry(&capture);
    let inspection = shell.inspect(&resource_id).expect("inspection");
    assert_eq!(
        inspection.entries()[0].public_value(),
        Some("test.alpha.module")
    );
}

#[test]
fn inspection_failure_does_not_unregister_resource_or_poison_shell() {
    let resource_id = ResourceId::new("alpha").expect("resource id");
    let capture = Arc::new(Mutex::new(None));
    let composition = CompositionBuilder::new(
        CompositionId::new("test.resource.registry.inspect-failure".to_owned())
            .expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("resource-registry".to_owned()).expect("block id"))
            .register_module(ResourceRegistryModule::new())
            .register_module(SyntheticCapabilityModule::new(
                "test.alpha.module",
                resource_id.clone(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                ResourceDescriptor::new(resource_id.clone()).with_inspection(
                    ResourceInspectionFacet::new(move || {
                        Err(crate::ResourceRegistryError::InspectionFailed {
                            message: "inspection failed safely".to_owned(),
                        })
                    }),
                ),
            ))
            .register_module(CapabilityCaptureModule::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");

    let _instance = start_test_instance("test.resource.registry.inspect-failure", composition)
        .expect("start instance");
    let shell = capture_resource_registry(&capture);
    let error = shell
        .inspect(&resource_id)
        .expect_err("inspection must fail");
    assert_eq!(
        error,
        crate::ResourceRegistryError::InspectionFailed {
            message: "inspection failed safely".to_owned(),
        }
    );
    assert_eq!(shell.resources().len(), 1);
    let descriptor = shell.resource(&resource_id).expect("resource remains");
    assert_eq!(descriptor.resource_id(), &resource_id);
}
