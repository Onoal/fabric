use std::error::Error;
use std::sync::{Arc, Mutex};

use fabric_core::{
    CompositionError, ContractKey, ContractRequirement, ContractVersion,
    ContractVersionRequirement, Health, Module, ModuleBindings, ModuleContract, ModuleError,
    ModuleId, ModuleRuntime,
};

use crate::SdkAuthoringError;
use crate::authoring::{BlockAuthor, CompositionExt, FabricBuilder};
use crate::authoring::{PrimarySystemContract, SystemDefinition, SystemRequires, SystemSelection};
use crate::contracts::{versioned_provider, versioned_requirement};
use crate::ids::{block, composition, contract, instance};

#[derive(Clone)]
struct EchoContract {
    value: Arc<String>,
}

#[derive(Clone)]
struct SystemEchoContract;

#[derive(Clone)]
struct SystemEchoConfig {
    message: String,
}

struct SystemEcho;

impl SystemEcho {
    fn select(
        config: SystemEchoConfig,
    ) -> Result<SystemSelection<Self>, crate::system::SystemCompatibilityError> {
        <Self as SystemDefinition>::select(config)
    }
}

impl SystemDefinition for SystemEcho {
    type Config = SystemEchoConfig;

    fn system_id() -> crate::system::SystemId {
        crate::system::SystemId::new("fabric.test.sdk.system-echo").expect("system id")
    }

    fn schema() -> crate::system::SystemSchemaDescriptor {
        crate::system::SystemSchemaDescriptor::provisional(Self::system_id())
    }

    fn declaration(selection: &SystemSelection<Self>) -> fabric_core::ModuleDeclaration {
        fabric_core::ModuleDeclaration::new(selection.module_id().clone())
            .with_provided_contracts(vec![Self::primary_contract_key().declaration()])
    }

    fn materialize(
        selection: &SystemSelection<Self>,
    ) -> Option<Box<dyn fabric_core::ModuleRuntime>> {
        Some(Box::new(SystemEchoProvider {
            module_id: selection.module_id().clone(),
        }))
    }
}

impl PrimarySystemContract for SystemEcho {
    type Contract = SystemEchoContract;

    fn primary_contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(contract("fabric.test.sdk.system-echo").expect("contract"))
    }
}

struct SystemEchoProvider {
    module_id: ModuleId,
}

impl ModuleRuntime for SystemEchoProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![SystemEcho::primary_contract_key().declaration()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &SystemEcho::primary_contract_key(),
            Arc::new(SystemEchoContract),
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

impl EchoContract {
    fn read(&self) -> String {
        self.value.as_ref().clone()
    }
}

#[derive(Clone)]
struct EchoProvider {
    module_id: ModuleId,
    key: ContractKey<EchoContract>,
}

impl EchoProvider {
    fn new(module_id: &str, key: ContractKey<EchoContract>) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            key,
        }
    }
}

impl ModuleRuntime for EchoProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![self.key.declaration()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &self.key,
            Arc::new(EchoContract {
                value: Arc::new("hello sdk".to_owned()),
            }),
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
struct EchoConsumer {
    module_id: ModuleId,
    requirement: ContractRequirement<EchoContract>,
    capture: Arc<Mutex<Option<String>>>,
}

impl EchoConsumer {
    fn new(
        module_id: &str,
        requirement: ContractRequirement<EchoContract>,
        capture: Arc<Mutex<Option<String>>>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            requirement,
            capture,
        }
    }
}

impl ModuleRuntime for EchoConsumer {
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
        let contract = bindings
            .resolve(&self.requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.capture.lock().expect("capture lock") = Some(contract.read());
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
fn fabric_builder_builds_raw_composition_and_materializes_raw_instance() {
    let capture = Arc::new(Mutex::new(None));
    let key = versioned_provider::<EchoContract>("fabric.test.sdk.echo", "1.4.0").expect("key");
    let requirement =
        versioned_requirement::<EchoContract>("fabric.test.sdk.echo", "^1.2").expect("req");
    let composition = FabricBuilder::new("fabric.test.sdk")
        .expect("builder")
        .block("runtime", |block| {
            block.module(EchoProvider::new("provider", key))
        })
        .expect("runtime block")
        .block("consumer", |block| {
            block.module(EchoConsumer::new(
                "consumer",
                requirement,
                Arc::clone(&capture),
            ))
        })
        .expect("consumer block")
        .build()
        .expect("composition");

    takes_raw_composition(&composition);
    let mut instance = composition
        .materialize_named("fabric.test.sdk.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some("hello sdk".to_owned())
    );
}

#[test]
fn readme_flow_builds_materializes_starts_and_stops_with_public_sdk_apis() {
    let composition = FabricBuilder::new("example")
        .expect("builder")
        .block("runtime", |block| block)
        .expect("runtime block")
        .build()
        .expect("composition");

    takes_raw_composition(&composition);

    let mut instance = composition
        .materialize_named("example.local")
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");
}

#[test]
fn id_helpers_preserve_strong_raw_id_types() {
    assert_eq!(
        composition("fabric.test.sdk")
            .expect("composition")
            .as_str(),
        "fabric.test.sdk"
    );
    assert_eq!(block("runtime").expect("block").as_str(), "runtime");
    assert_eq!(
        instance("fabric.test.sdk.instance")
            .expect("instance")
            .as_str(),
        "fabric.test.sdk.instance"
    );
    assert_eq!(
        contract("fabric.test.sdk.notes")
            .expect("contract")
            .as_str(),
        "fabric.test.sdk.notes"
    );
}

#[test]
fn invalid_string_authoring_fails_before_raw_composition_construction() {
    let error = match FabricBuilder::new("invalid id") {
        Ok(_) => panic!("invalid identifier must fail"),
        Err(error) => error,
    };
    assert!(matches!(error, CompositionError::InvalidIdentifier { .. }));
}

#[test]
fn block_author_remains_grouping_only_convenience() {
    let block = BlockAuthor::new("runtime")
        .expect("block author")
        .module(EchoProvider::new(
            "provider",
            ContractKey::versioned(
                contract("fabric.test.sdk.echo").expect("contract"),
                ContractVersion::parse("1.4.0").expect("version"),
            ),
        ))
        .build();

    assert_eq!(block.id().as_str(), "runtime");
}

#[test]
fn version_helpers_return_canonical_core_version_types() {
    assert_eq!(
        crate::versions::contract_version("1.4.0")
            .expect("version")
            .to_string(),
        ContractVersion::parse("1.4.0")
            .expect("version")
            .to_string()
    );
    assert_eq!(
        crate::versions::contract_requirement("^1.2")
            .expect("requirement")
            .to_string(),
        ContractVersionRequirement::parse("^1.2")
            .expect("requirement")
            .to_string()
    );
}

#[test]
fn invalid_contract_version_preserves_original_parser_error_source() {
    let error = crate::versions::contract_version("not-a-version").expect_err("invalid version");
    match &error {
        SdkAuthoringError::InvalidContractVersion { value, .. } => {
            assert_eq!(value, "not-a-version");
        }
        other => panic!("unexpected error: {other}"),
    }
    let source = error.source().expect("version parse source");
    assert!(
        error
            .to_string()
            .contains("invalid contract version `not-a-version`"),
        "display must include the invalid input"
    );
    assert_eq!(
        source.to_string(),
        "unexpected character 'n' while parsing major version number"
    );
}

#[test]
fn invalid_contract_version_requirement_preserves_original_parser_error_source() {
    let error =
        crate::versions::contract_requirement("not-a-requirement").expect_err("invalid req");
    match &error {
        SdkAuthoringError::InvalidContractVersionRequirement { value, .. } => {
            assert_eq!(value, "not-a-requirement");
        }
        other => panic!("unexpected error: {other}"),
    }
    let source = error.source().expect("requirement parse source");
    assert!(
        error
            .to_string()
            .contains("invalid contract version requirement `not-a-requirement`"),
        "display must include the invalid input"
    );
    assert!(
        !source.to_string().is_empty(),
        "parser source should stay available"
    );
}

#[test]
fn invalid_identifier_flowing_through_versioned_contract_helper_preserves_composition_source() {
    let error = match versioned_provider::<EchoContract>("invalid id", "1.4.0") {
        Ok(_) => panic!("invalid identifier must fail"),
        Err(error) => error,
    };
    match &error {
        SdkAuthoringError::Composition(CompositionError::InvalidIdentifier { .. }) => {}
        other => panic!("unexpected error: {other}"),
    }
    assert!(
        error.source().is_some(),
        "composition source must stay available"
    );
}

#[test]
fn system_requires_public_apis_always_target_the_primary_contract_id() {
    let provisional = SystemRequires::<SystemEcho>::provisional();
    let versioned = SystemRequires::<SystemEcho>::versioned(
        ContractVersionRequirement::parse("^1").expect("requirement"),
    );

    assert_eq!(
        provisional.as_contract_requirement().id(),
        SystemEcho::primary_contract_key().id()
    );
    assert_eq!(
        versioned.as_contract_requirement().id(),
        SystemEcho::primary_contract_key().id()
    );
}

#[test]
fn system_selection_module_ids_depend_only_on_system_id() {
    let first = SystemEcho::select(SystemEchoConfig {
        message: "first".to_owned(),
    })
    .expect("system selection");
    let second = SystemEcho::select(SystemEchoConfig {
        message: "second".to_owned(),
    })
    .expect("system selection");

    assert_eq!(first.module_id(), second.module_id());
    assert!(first.module_id().as_str().starts_with("fabric.system."));
    assert_eq!(first.config().message, "first");
    assert_eq!(second.config().message, "second");
}

#[test]
fn system_definition_materializes_raw_modules_without_a_second_runtime_model() {
    let selection = SystemEcho::select(SystemEchoConfig {
        message: "hello system".to_owned(),
    })
    .expect("system selection");
    let runtime = selection
        .materialize()
        .expect("system selections are runtime-capable");
    let declaration = runtime
        .provided_contract_declarations()
        .into_iter()
        .next()
        .expect("contract");

    assert_eq!(runtime.id(), selection.module_id());
    assert_eq!(declaration.id(), SystemEcho::primary_contract_key().id());
    assert_eq!(
        declaration.identity(),
        SystemEcho::primary_contract_key().identity()
    );
}

fn takes_raw_composition(_composition: &fabric_core::Composition) {}
