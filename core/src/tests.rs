use std::sync::{Arc, Mutex};

use crate::block::BlockBuilder;
use crate::composition::ModuleBindings;
use crate::contract::{
    ContractIdentity, ContractKey, ContractRequirement, ContractRequirementDeclaration,
    ContractVersion, ContractVersionRequirement, ModuleContract, ProvidedContractDeclaration,
};
use crate::error::{CompositionError, InstanceError, ModuleError};
use crate::health::Health;
use crate::identifiers::{BlockId, ContractId, InstanceId, ModuleId};
use crate::lifecycle::LifecycleState;
use crate::module::Module;
use crate::module_runtime::ModuleRuntime;
use crate::{Composition, CompositionBuilder, CompositionExport, CompositionId, Instance};

#[derive(Clone)]
struct TestContract {
    greeter: Arc<dyn GreetingPort>,
}

impl TestContract {
    fn greet(&self) -> String {
        self.greeter.greet()
    }
}

trait GreetingPort: Send + Sync {
    fn greet(&self) -> String;
}

struct Recorder {
    events: Mutex<Vec<String>>,
}

fn composition_for(block: crate::Block) -> Result<Composition, CompositionError> {
    CompositionBuilder::new(CompositionId::new("test.composition".to_owned()).expect("composition"))
        .register_block(block)
        .build()
}

fn materialize_test_instance(
    composition: &Composition,
    instance_id: &str,
) -> Result<Instance, CompositionError> {
    composition.materialize(InstanceId::new(instance_id.to_owned()).expect("instance id"))
}

struct DeclarationOnlyProvider {
    module_id: ModuleId,
    key: ContractKey<TestContract>,
}

impl DeclarationOnlyProvider {
    fn new(module_id: &str, key: ContractKey<TestContract>) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            key,
        }
    }
}

impl Module for DeclarationOnlyProvider {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::new(self.module_id.clone())
            .with_provided_contracts(vec![self.key.declaration()])
    }
}

struct DeclarationOnlyConsumer {
    module_id: ModuleId,
    requirement: ContractRequirement<TestContract>,
}

impl DeclarationOnlyConsumer {
    fn new(module_id: &str, requirement: ContractRequirement<TestContract>) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            requirement,
        }
    }
}

impl Module for DeclarationOnlyConsumer {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::new(self.module_id.clone())
            .with_required_contracts(vec![self.requirement.declaration().clone()])
    }
}

struct BuildPanicRuntimeModule {
    module_id: ModuleId,
}

impl BuildPanicRuntimeModule {
    fn new(module_id: &str) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
        }
    }
}

impl Module for BuildPanicRuntimeModule {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::new(self.module_id.clone())
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        panic!("CompositionBuilder::build must not materialize runtime modules")
    }
}

#[test]
fn declaration_only_modules_validate_without_runtime_materialization() {
    let contract_id = ContractId::new("fabric.test.declaration-only").expect("contract id");
    let provider = DeclarationOnlyProvider::new(
        "fabric.test.declaration-only.provider",
        ContractKey::provisional(contract_id.clone()),
    );
    let consumer = DeclarationOnlyConsumer::new(
        "fabric.test.declaration-only.consumer",
        ContractRequirement::provisional(contract_id),
    );
    let composition = CompositionBuilder::new(
        CompositionId::new("fabric.test.declaration-only".to_owned()).expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("declarations".to_owned()).expect("block id"))
            .register_module(provider)
            .register_module(consumer)
            .register_module(BuildPanicRuntimeModule::new(
                "fabric.test.declaration-only.panic-runtime",
            ))
            .build(),
    )
    .build()
    .expect("declarations should validate without runtime materialization");

    assert!(matches!(
        composition.materialize(InstanceId::new("declaration-only".to_owned()).expect("instance id")),
        Err(CompositionError::MissingRuntimeMaterializer { module_id })
            if module_id.as_str() == "fabric.test.declaration-only.provider"
    ));
}

impl Recorder {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            events: Mutex::new(Vec::new()),
        })
    }

    fn push(&self, value: impl Into<String>) {
        self.events.lock().expect("events lock").push(value.into());
    }

    fn snapshot(&self) -> Vec<String> {
        self.events.lock().expect("events lock").clone()
    }
}

#[derive(Clone)]
struct TestProvider {
    module_id: ModuleId,
    contract_key: ContractKey<TestContract>,
    recorder: Arc<Recorder>,
    health: Health,
    greeting: String,
}

impl TestProvider {
    fn new(
        module_id: &str,
        contract_key: ContractKey<TestContract>,
        recorder: Arc<Recorder>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            contract_key,
            recorder,
            health: Health::Healthy,
            greeting: "hello from provider".to_owned(),
        }
    }

    fn with_greeting(mut self, greeting: impl Into<String>) -> Self {
        self.greeting = greeting.into();
        self
    }
}

#[derive(Clone)]
struct MultiVersionProvider {
    module_id: ModuleId,
    contracts: Vec<(ContractKey<TestContract>, String)>,
}

impl MultiVersionProvider {
    fn new(module_id: &str, contracts: Vec<(ContractKey<TestContract>, String)>) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            contracts,
        }
    }
}

impl ModuleRuntime for TestProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<ProvidedContractDeclaration> {
        vec![self.contract_key.declaration()]
    }

    fn required_contract_declarations(&self) -> Vec<crate::ContractRequirementDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &self.contract_key,
            Arc::new(TestContract {
                greeter: Arc::new(TestProviderHandle(self.greeting.clone())),
            }),
        )])
    }

    fn bind(&mut self, _bindings: &ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("initialize:{}", self.module_id.as_str()));
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("start:{}", self.module_id.as_str()));
        Ok(())
    }

    fn stop(&mut self) {
        self.recorder
            .push(format!("stop:{}", self.module_id.as_str()));
    }

    fn health(&self) -> Health {
        self.health
    }
}

impl ModuleRuntime for MultiVersionProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<ProvidedContractDeclaration> {
        self.contracts
            .iter()
            .map(|(key, _)| key.declaration())
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<crate::ContractRequirementDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(self
            .contracts
            .iter()
            .map(|(key, greeting)| {
                ModuleContract::new(
                    key,
                    Arc::new(TestContract {
                        greeter: Arc::new(TestProviderHandle(greeting.clone())),
                    }),
                )
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

    fn stop(&mut self) {}

    fn health(&self) -> Health {
        Health::Healthy
    }
}

struct TestProviderHandle(String);

impl GreetingPort for TestProviderHandle {
    fn greet(&self) -> String {
        self.0.clone()
    }
}

#[derive(Clone)]
struct ResolutionObserver {
    module_id: ModuleId,
    requirement: ContractRequirement<TestContract>,
    observed_provider: Arc<Mutex<Option<ModuleId>>>,
    observed_identity: Arc<Mutex<Option<ContractIdentity>>>,
    observed_greeting: Arc<Mutex<Option<String>>>,
}

impl ResolutionObserver {
    fn new(
        module_id: &str,
        requirement: ContractRequirement<TestContract>,
        observed_provider: Arc<Mutex<Option<ModuleId>>>,
        observed_identity: Arc<Mutex<Option<ContractIdentity>>>,
        observed_greeting: Arc<Mutex<Option<String>>>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            requirement,
            observed_provider,
            observed_identity,
            observed_greeting,
        }
    }
}

impl ModuleRuntime for ResolutionObserver {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let resolved = bindings
            .resolve_with_provider(&self.requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.observed_provider.lock().expect("provider lock") = Some(resolved.provider().clone());
        *self.observed_identity.lock().expect("identity lock") = Some(resolved.identity().clone());
        *self.observed_greeting.lock().expect("greeting lock") = Some(resolved.value().greet());
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

struct TestConsumer {
    module_id: ModuleId,
    requirement: ContractRequirement<TestContract>,
    recorder: Arc<Recorder>,
    contract: Option<Arc<TestContract>>,
    health: Health,
}

impl TestConsumer {
    fn new(
        module_id: &str,
        requirement: ContractRequirement<TestContract>,
        recorder: Arc<Recorder>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            requirement,
            recorder,
            contract: None,
            health: Health::Degraded,
        }
    }

    fn used_greeting(&self) -> String {
        self.contract.as_ref().expect("contract bound").greet()
    }
}

impl ModuleRuntime for TestConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.contract = Some(
            bindings
                .resolve(&self.requirement)
                .map_err(|error| ModuleError::new(error.to_string()))?,
        );
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("initialize:{}", self.module_id.as_str()));
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("start:{}", self.module_id.as_str()));
        self.recorder.push(format!("use:{}", self.used_greeting()));
        self.health = Health::Healthy;
        Ok(())
    }

    fn stop(&mut self) {
        self.recorder
            .push(format!("stop:{}", self.module_id.as_str()));
    }

    fn health(&self) -> Health {
        self.health
    }
}

impl Module for TestConsumer {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self::new(
            self.module_id.as_str(),
            self.requirement.clone(),
            Arc::clone(&self.recorder),
        )))
    }
}

struct FailingConsumer {
    inner: TestConsumer,
}

impl FailingConsumer {
    fn new(inner: TestConsumer) -> Self {
        Self { inner }
    }
}

impl ModuleRuntime for FailingConsumer {
    fn id(&self) -> &ModuleId {
        self.inner.id()
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        self.inner.provided_contract_declarations()
    }

    fn required_contract_declarations(&self) -> Vec<crate::ContractRequirementDeclaration> {
        self.inner.required_contract_declarations()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        self.inner.export_contracts()
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.inner.bind(bindings)
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.inner.initialize()
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.inner.start()?;
        Err(ModuleError::new("forced startup failure"))
    }

    fn stop(&mut self) {
        self.inner.stop();
    }

    fn health(&self) -> Health {
        self.inner.health()
    }
}

impl Module for FailingConsumer {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self::new(TestConsumer::new(
            self.inner.module_id.as_str(),
            self.inner.requirement.clone(),
            Arc::clone(&self.inner.recorder),
        ))))
    }
}

struct CycleModule {
    module_id: ModuleId,
    provides: ContractId,
    requires: ContractId,
}

impl CycleModule {
    fn new(module_id: &str, provides: ContractId, requires: ContractId) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            provides,
            requires,
        }
    }
}

impl ModuleRuntime for CycleModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        vec![self.provides.clone()]
            .into_iter()
            .map(crate::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<crate::ContractRequirementDeclaration> {
        vec![self.requires.clone()]
            .into_iter()
            .map(crate::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        let key = ContractKey::<String>::provisional(self.provides.clone());
        Ok(vec![ModuleContract::new(
            &key,
            Arc::new(self.module_id.as_str().to_owned()),
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

    fn stop(&mut self) {}

    fn health(&self) -> Health {
        Health::Healthy
    }
}

impl Module for CycleModule {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self::new(
            self.module_id.as_str(),
            self.provides.clone(),
            self.requires.clone(),
        )))
    }
}

#[test]
fn successful_single_block_instance_binds_typed_contract_and_stops_in_reverse_order() {
    let recorder = Recorder::new();
    let key: ContractKey<TestContract> =
        ContractKey::provisional(ContractId::new("test.greeting".to_owned()).expect("contract"));
    let requirement = ContractRequirement::provisional(key.id().clone());

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(TestProvider::new("provider", key, Arc::clone(&recorder)))
        .register_module(TestConsumer::new(
            "consumer",
            requirement,
            Arc::clone(&recorder),
        ))
        .build();
    let composition = composition_for(block).expect("composition");
    let mut instance = materialize_test_instance(&composition, "test.block").expect("instance");

    assert_eq!(instance.lifecycle(), LifecycleState::Ready);

    instance.start().expect("start");
    let report = instance.report();
    assert_eq!(report.lifecycle, LifecycleState::Running);
    assert_eq!(report.health, Health::Healthy);
    assert_eq!(report.blocks.len(), 1);
    assert_eq!(report.blocks[0].block_id.as_str(), "test.block");
    assert_eq!(report.blocks[0].lifecycle, LifecycleState::Running);

    instance.stop();
    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
    assert_eq!(
        recorder.snapshot(),
        vec![
            "initialize:provider",
            "initialize:consumer",
            "start:provider",
            "start:consumer",
            "use:hello from provider",
            "stop:consumer",
            "stop:provider",
        ]
    );
}

#[test]
fn explicit_composition_export_retains_only_the_declared_runtime_contract() {
    let recorder = Recorder::new();
    let key = ContractKey::provisional(ContractId::new("test.export.contract").expect("contract"));
    let export: CompositionExport<TestContract> = CompositionExport::new(
        ContractId::new("test.export.operator").expect("export id"),
        ContractRequirement::provisional(key.id().clone()),
    );
    let undeclared = CompositionExport::new(
        ContractId::new("test.export.undeclared").expect("export id"),
        ContractRequirement::<TestContract>::provisional(key.id().clone()),
    );
    let block = BlockBuilder::new(BlockId::new("test.export.block").expect("block"))
        .register_module(TestProvider::new("provider", key, recorder))
        .build();
    let composition = CompositionBuilder::new(
        CompositionId::new("test.export.composition").expect("composition"),
    )
    .register_block(block)
    .export(export.clone())
    .build()
    .expect("composition");
    let instance =
        materialize_test_instance(&composition, "test.export.instance").expect("instance");

    assert_eq!(
        instance.export(&export).expect("declared export").greet(),
        "hello from provider"
    );
    assert!(instance.export(&undeclared).is_none());
}

#[test]
fn cross_block_contract_resolution_orders_lifecycle_and_aggregates_health() {
    let recorder = Recorder::new();
    let key =
        ContractKey::provisional(ContractId::new("test.cross-block".to_owned()).expect("contract"));
    let provider = BlockBuilder::new(BlockId::new("test.provider".to_owned()).expect("block"))
        .register_module(TestProvider::new(
            "provider",
            key.clone(),
            Arc::clone(&recorder),
        ))
        .build();
    let consumer = BlockBuilder::new(BlockId::new("test.consumer".to_owned()).expect("block"))
        .register_module(TestConsumer::new(
            "consumer",
            ContractRequirement::provisional(key.id().clone()),
            Arc::clone(&recorder),
        ))
        .build();
    let composition =
        CompositionBuilder::new(CompositionId::new("test.cloud".to_owned()).expect("composition"))
            .register_block(provider)
            .register_block(consumer)
            .build()
            .expect("composition");
    let mut instance = materialize_test_instance(&composition, "test.cloud").expect("instance");
    instance.start().expect("start");
    assert_eq!(instance.report().health, Health::Healthy);
    instance.stop();
    assert_eq!(
        recorder.snapshot(),
        vec![
            "initialize:provider",
            "initialize:consumer",
            "start:provider",
            "start:consumer",
            "use:hello from provider",
            "stop:consumer",
            "stop:provider"
        ]
    );
}

#[test]
fn cross_block_start_failure_unwinds_each_initialized_module_once() {
    let recorder = Recorder::new();
    let key = ContractKey::provisional(
        ContractId::new("test.cross-block-failure".to_owned()).expect("contract"),
    );
    let provider = BlockBuilder::new(BlockId::new("test.provider".to_owned()).expect("block"))
        .register_module(TestProvider::new(
            "provider",
            key.clone(),
            Arc::clone(&recorder),
        ))
        .build();
    let consumer = BlockBuilder::new(BlockId::new("test.consumer".to_owned()).expect("block"))
        .register_module(FailingConsumer::new(TestConsumer::new(
            "consumer",
            ContractRequirement::provisional(key.id().clone()),
            Arc::clone(&recorder),
        )))
        .build();
    let composition = CompositionBuilder::new(
        CompositionId::new("test.cross-block-failure".to_owned()).expect("composition"),
    )
    .register_block(provider)
    .register_block(consumer)
    .build()
    .expect("composition");
    let mut instance =
        materialize_test_instance(&composition, "test.cross-block-failure").expect("instance");

    let error = instance.start().expect_err("startup failure");

    assert!(matches!(
        error,
        InstanceError::ModuleFailure { phase: "start", .. }
    ));
    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
    assert_eq!(
        recorder.snapshot(),
        vec![
            "initialize:provider",
            "initialize:consumer",
            "start:provider",
            "start:consumer",
            "use:hello from provider",
            "stop:consumer",
            "stop:provider",
        ]
    );
}

#[test]
fn block_is_structural_input_and_instance_owns_runtime_lifecycle() {
    let recorder = Recorder::new();
    let key = ContractKey::provisional(
        ContractId::new("test.instance-block".to_owned()).expect("contract"),
    );
    let block = BlockBuilder::new(BlockId::new("test.instance-block".to_owned()).expect("block"))
        .register_module(TestProvider::new("provider", key, Arc::clone(&recorder)))
        .build();
    let composition = CompositionBuilder::new(
        CompositionId::new("test.instance-block-composition".to_owned()).expect("composition"),
    )
    .register_block(block)
    .build()
    .expect("composition");
    let mut instance =
        materialize_test_instance(&composition, "test.instance-block").expect("instance");

    assert_eq!(instance.lifecycle(), LifecycleState::Ready);
    instance.start().expect("start");
    assert_eq!(
        instance.report().blocks[0].lifecycle,
        LifecycleState::Running
    );
    instance.stop();
    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
    assert_eq!(
        instance.report().blocks[0].lifecycle,
        LifecycleState::Stopped
    );
    assert_eq!(
        recorder.snapshot(),
        vec!["initialize:provider", "start:provider", "stop:provider"]
    );
}

#[test]
fn ready_instance_stop_invokes_no_module_stop_hooks() {
    let recorder = Recorder::new();
    let key = ContractKey::provisional(
        ContractId::new("test.ready-instance-stop".to_owned()).expect("contract"),
    );
    let requirement = ContractRequirement::provisional(key.id().clone());
    let block =
        BlockBuilder::new(BlockId::new("test.ready-instance-stop".to_owned()).expect("block"))
            .register_module(TestProvider::new("provider", key, Arc::clone(&recorder)))
            .register_module(TestConsumer::new(
                "consumer",
                requirement,
                Arc::clone(&recorder),
            ))
            .build();
    let composition = composition_for(block).expect("composition");
    let mut instance =
        materialize_test_instance(&composition, "test.ready-instance-stop").expect("instance");

    instance.stop();

    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
    assert!(recorder.snapshot().is_empty());
}

#[test]
fn cross_block_duplicate_ids_and_cycles_fail_before_startup() {
    let key_a = ContractId::new("test.a".to_owned()).expect("contract");
    let key_b = ContractId::new("test.b".to_owned()).expect("contract");
    let first =
        BlockBuilder::new(BlockId::new("test.duplicate".to_owned()).expect("block")).build();
    let second =
        BlockBuilder::new(BlockId::new("test.duplicate".to_owned()).expect("block")).build();
    assert!(matches!(
        CompositionBuilder::new(
            CompositionId::new("test.duplicate.cloud".to_owned()).expect("composition")
        )
        .register_block(first)
        .register_block(second)
        .build(),
        Err(CompositionError::DuplicateBlockId { .. })
    ));
    let left = BlockBuilder::new(BlockId::new("test.left".to_owned()).expect("block"))
        .register_module(CycleModule::new("left", key_a.clone(), key_b.clone()))
        .build();
    let right = BlockBuilder::new(BlockId::new("test.right".to_owned()).expect("block"))
        .register_module(CycleModule::new("right", key_b, key_a))
        .build();
    assert!(matches!(
        CompositionBuilder::new(
            CompositionId::new("test.cycle.cloud".to_owned()).expect("composition")
        )
        .register_block(left)
        .register_block(right)
        .build(),
        Err(CompositionError::DependencyCycle { .. })
    ));
}

#[test]
fn missing_dependency_fails_before_lifecycle_side_effects() {
    let recorder = Recorder::new();
    let key: ContractKey<TestContract> =
        ContractKey::provisional(ContractId::new("test.greeting".to_owned()).expect("contract"));
    let requirement = ContractRequirement::provisional(key.id().clone());

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(TestConsumer::new(
            "consumer",
            requirement,
            Arc::clone(&recorder),
        ))
        .build();
    let error = composition_for(block).expect_err("expected missing provider");

    assert!(matches!(error, CompositionError::MissingProvider { .. }));
    assert!(recorder.snapshot().is_empty());
}

#[test]
fn duplicate_module_identity_is_rejected() {
    let recorder = Recorder::new();
    let key =
        ContractKey::provisional(ContractId::new("test.greeting".to_owned()).expect("contract"));

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(TestProvider::new(
            "duplicate",
            key.clone(),
            Arc::clone(&recorder),
        ))
        .register_module(TestProvider::new("duplicate", key, Arc::clone(&recorder)))
        .build();
    let error = composition_for(block).expect_err("expected duplicate module id");

    assert!(matches!(error, CompositionError::DuplicateModuleId { .. }));
}

#[test]
fn dependency_cycle_fails_before_startup() {
    let cycle_a = ContractId::new("test.cycle.a".to_owned()).expect("contract");
    let cycle_b = ContractId::new("test.cycle.b".to_owned()).expect("contract");

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(CycleModule::new(
            "module-a",
            cycle_a.clone(),
            cycle_b.clone(),
        ))
        .register_module(CycleModule::new("module-b", cycle_b, cycle_a))
        .build();
    let error = composition_for(block).expect_err("expected dependency cycle");

    assert!(matches!(error, CompositionError::DependencyCycle { .. }));
}

#[test]
fn ambiguous_provider_fails_deterministically() {
    let recorder = Recorder::new();
    let key =
        ContractKey::provisional(ContractId::new("test.greeting".to_owned()).expect("contract"));
    let requirement = ContractRequirement::provisional(key.id().clone());

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(TestProvider::new(
            "provider-a",
            key.clone(),
            Arc::clone(&recorder),
        ))
        .register_module(TestProvider::new("provider-b", key, Arc::clone(&recorder)))
        .register_module(TestConsumer::new(
            "consumer",
            requirement,
            Arc::clone(&recorder),
        ))
        .build();
    let error = composition_for(block).expect_err("expected ambiguous provider");

    assert!(matches!(error, CompositionError::AmbiguousProvider { .. }));
    assert!(recorder.snapshot().is_empty());
}

#[test]
fn startup_failure_unwinds_started_dependencies() {
    let recorder = Recorder::new();
    let key =
        ContractKey::provisional(ContractId::new("test.greeting".to_owned()).expect("contract"));
    let requirement = ContractRequirement::provisional(key.id().clone());

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(TestProvider::new("provider", key, Arc::clone(&recorder)))
        .register_module(FailingConsumer::new(TestConsumer::new(
            "consumer",
            requirement,
            Arc::clone(&recorder),
        )))
        .build();
    let composition = composition_for(block).expect("composition");
    let mut instance = materialize_test_instance(&composition, "test.block").expect("instance");

    let error = instance.start().expect_err("startup failure");
    assert!(matches!(
        error,
        InstanceError::ModuleFailure { phase: "start", .. }
    ));
    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
    assert_eq!(
        recorder.snapshot(),
        vec![
            "initialize:provider",
            "initialize:consumer",
            "start:provider",
            "start:consumer",
            "use:hello from provider",
            "stop:consumer",
            "stop:provider",
        ]
    );
}

#[test]
fn same_composition_materializes_fresh_instances_with_shared_identity_and_fresh_generation() {
    let recorder = Recorder::new();
    let key = ContractKey::provisional(
        ContractId::new("test.reusable.greeting".to_owned()).expect("contract"),
    );
    let requirement = ContractRequirement::provisional(key.id().clone());
    let composition = composition_for(
        BlockBuilder::new(BlockId::new("test.reusable.block".to_owned()).expect("block"))
            .register_module(TestProvider::new("provider", key, Arc::clone(&recorder)))
            .register_module(TestConsumer::new(
                "consumer",
                requirement,
                Arc::clone(&recorder),
            ))
            .build(),
    )
    .expect("composition");
    let instance_id = InstanceId::new("test.reusable.instance".to_owned()).expect("instance id");

    let mut instance_one = composition
        .materialize(instance_id.clone())
        .expect("materialize first instance");
    instance_one.start().expect("start first instance");
    instance_one.stop();

    let first_report = instance_one.report();
    let mut instance_two = composition
        .materialize(instance_id.clone())
        .expect("materialize second instance");
    let second_report_before_start = instance_two.report();
    instance_two.start().expect("start second instance");
    instance_two.stop();

    assert_eq!(instance_one.instance_id(), &instance_id);
    assert_eq!(instance_two.instance_id(), &instance_id);
    assert_ne!(instance_one.generation(), instance_two.generation());
    assert_eq!(first_report.lifecycle, LifecycleState::Stopped);
    assert_eq!(second_report_before_start.lifecycle, LifecycleState::Ready);
    assert_eq!(second_report_before_start.health, Health::Degraded);
    assert_eq!(
        recorder.snapshot(),
        vec![
            "initialize:provider",
            "initialize:consumer",
            "start:provider",
            "start:consumer",
            "use:hello from provider",
            "stop:consumer",
            "stop:provider",
            "initialize:provider",
            "initialize:consumer",
            "start:provider",
            "start:consumer",
            "use:hello from provider",
            "stop:consumer",
            "stop:provider",
        ]
    );
}

struct UndeclaredResolver {
    module_id: ModuleId,
    hidden_requirement: ContractRequirement<TestContract>,
    recorder: Arc<Recorder>,
}

impl UndeclaredResolver {
    fn new(
        module_id: &str,
        hidden_requirement: ContractRequirement<TestContract>,
        recorder: Arc<Recorder>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            hidden_requirement,
            recorder,
        }
    }
}

impl ModuleRuntime for UndeclaredResolver {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<crate::ContractRequirementDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let error = match bindings.resolve(&self.hidden_requirement) {
            Ok(_) => panic!("undeclared contract must not resolve"),
            Err(error) => error,
        };
        self.recorder.push(format!("bind-error:{error}"));
        Err(ModuleError::new(error.to_string()))
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("initialize:{}", self.module_id.as_str()));
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("start:{}", self.module_id.as_str()));
        Ok(())
    }

    fn stop(&mut self) {
        self.recorder
            .push(format!("stop:{}", self.module_id.as_str()));
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

impl Module for UndeclaredResolver {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self::new(
            self.module_id.as_str(),
            self.hidden_requirement.clone(),
            Arc::clone(&self.recorder),
        )))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedRequirementMismatch {
    module_id: ModuleId,
    contract_id: ContractId,
    declared_compatibility: crate::ContractCompatibilityRequirement,
    requested_compatibility: crate::ContractCompatibilityRequirement,
}

struct MismatchedRequirementResolver {
    module_id: ModuleId,
    declared_requirement: ContractRequirement<TestContract>,
    bind_requirement: ContractRequirement<TestContract>,
    optional: bool,
    observed: Arc<Mutex<Option<ObservedRequirementMismatch>>>,
}

impl MismatchedRequirementResolver {
    fn new(
        module_id: &str,
        declared_requirement: ContractRequirement<TestContract>,
        bind_requirement: ContractRequirement<TestContract>,
        optional: bool,
        observed: Arc<Mutex<Option<ObservedRequirementMismatch>>>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            declared_requirement,
            bind_requirement,
            optional,
            observed,
        }
    }
}

impl ModuleRuntime for MismatchedRequirementResolver {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        Vec::new()
    }

    fn required_contract_declarations(&self) -> Vec<crate::ContractRequirementDeclaration> {
        if self.optional {
            Vec::new()
        } else {
            vec![self.declared_requirement.declaration().clone()]
        }
    }

    fn optional_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        if self.optional {
            vec![self.declared_requirement.declaration().clone()]
        } else {
            Vec::new()
        }
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let error = if self.optional {
            match bindings.resolve_optional(&self.bind_requirement) {
                Ok(_) => panic!("mismatched optional requirement must fail"),
                Err(error) => error,
            }
        } else {
            match bindings.resolve(&self.bind_requirement) {
                Ok(_) => panic!("mismatched requirement must fail"),
                Err(error) => error,
            }
        };
        let CompositionError::RequirementDeclarationMismatch {
            module_id,
            contract_id,
            declared_compatibility,
            requested_compatibility,
        } = error
        else {
            panic!("expected declaration mismatch")
        };
        *self.observed.lock().expect("observed lock") = Some(ObservedRequirementMismatch {
            module_id,
            contract_id,
            declared_compatibility,
            requested_compatibility,
        });
        Err(ModuleError::new(
            "requirement declaration mismatch observed",
        ))
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

impl Module for MismatchedRequirementResolver {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self::new(
            self.module_id.as_str(),
            self.declared_requirement.clone(),
            self.bind_requirement.clone(),
            self.optional,
            Arc::clone(&self.observed),
        )))
    }
}

struct ChainModule {
    module_id: ModuleId,
    provides: Option<ContractKey<String>>,
    requires: Vec<ContractId>,
    recorder: Arc<Recorder>,
    fail_on_start: bool,
}

impl ChainModule {
    fn new(
        module_id: &str,
        provides: Option<ContractKey<String>>,
        requires: Vec<ContractId>,
        recorder: Arc<Recorder>,
        fail_on_start: bool,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            provides,
            requires,
            recorder,
            fail_on_start,
        }
    }
}

impl ModuleRuntime for ChainModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        self.provides
            .as_ref()
            .map(|key| vec![key.id().clone()])
            .unwrap_or_default()
            .into_iter()
            .map(crate::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<crate::ContractRequirementDeclaration> {
        self.requires
            .clone()
            .into_iter()
            .map(crate::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        match &self.provides {
            Some(key) => Ok(vec![ModuleContract::new(
                key,
                Arc::new(self.module_id.as_str().to_owned()),
            )]),
            None => Ok(Vec::new()),
        }
    }

    fn bind(&mut self, _bindings: &ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("initialize:{}", self.module_id.as_str()));
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("start:{}", self.module_id.as_str()));
        if self.fail_on_start {
            Err(ModuleError::new("chain startup failure"))
        } else {
            Ok(())
        }
    }

    fn stop(&mut self) {
        self.recorder
            .push(format!("stop:{}", self.module_id.as_str()));
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

impl Module for ChainModule {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self::new(
            self.module_id.as_str(),
            self.provides.clone(),
            self.requires.clone(),
            Arc::clone(&self.recorder),
            self.fail_on_start,
        )))
    }
}

struct OptionalObserver {
    module_id: ModuleId,
    requirement: ContractRequirement<TestContract>,
    observed: Arc<Mutex<Option<String>>>,
}

impl OptionalObserver {
    fn new(
        module_id: &str,
        requirement: ContractRequirement<TestContract>,
        observed: Arc<Mutex<Option<String>>>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            requirement,
            observed,
        }
    }
}

impl ModuleRuntime for OptionalObserver {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<crate::ContractRequirementDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn optional_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let observed = bindings
            .resolve_optional(&self.requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?
            .map(|contract| contract.greet());
        *self.observed.lock().expect("observed lock") = observed;
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

impl Module for OptionalObserver {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self::new(
            self.module_id.as_str(),
            self.requirement.clone(),
            Arc::clone(&self.observed),
        )))
    }
}

#[derive(Clone)]
struct ProvenanceObserver {
    module_id: ModuleId,
    requirement: ContractRequirement<TestContract>,
    observed_provider: Arc<Mutex<Option<ModuleId>>>,
    observed_identity: Arc<Mutex<Option<ContractIdentity>>>,
}

impl ProvenanceObserver {
    fn new(
        module_id: &str,
        requirement: ContractRequirement<TestContract>,
        observed_provider: Arc<Mutex<Option<ModuleId>>>,
        observed_identity: Arc<Mutex<Option<ContractIdentity>>>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            requirement,
            observed_provider,
            observed_identity,
        }
    }
}

impl ModuleRuntime for ProvenanceObserver {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let resolved = bindings
            .resolve_with_provider(&self.requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.observed_provider.lock().expect("provider lock") = Some(resolved.provider().clone());
        *self.observed_identity.lock().expect("identity lock") = Some(resolved.identity().clone());
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

struct WrongTypeConsumer {
    module_id: ModuleId,
    requirement: ContractRequirement<String>,
}

impl WrongTypeConsumer {
    fn new(module_id: &str, requirement: ContractRequirement<String>) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            requirement,
        }
    }
}

impl ModuleRuntime for WrongTypeConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let _ = bindings
            .resolve(&self.requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
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

impl Module for WrongTypeConsumer {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self::new(
            self.module_id.as_str(),
            self.requirement.clone(),
        )))
    }
}

#[derive(Clone)]
struct DuplicateRequirementConsumer {
    module_id: ModuleId,
    required: Vec<ContractRequirementDeclaration>,
    optional: Vec<ContractRequirementDeclaration>,
}

impl DuplicateRequirementConsumer {
    fn new(
        module_id: &str,
        required: Vec<ContractRequirementDeclaration>,
        optional: Vec<ContractRequirementDeclaration>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            required,
            optional,
        }
    }
}

impl ModuleRuntime for DuplicateRequirementConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        Vec::new()
            .into_iter()
            .map(crate::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        self.required.clone()
    }

    fn optional_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        self.optional.clone()
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
fn undeclared_contract_cannot_be_resolved_during_bind() {
    let recorder = Recorder::new();
    let key: ContractKey<TestContract> =
        ContractKey::provisional(ContractId::new("test.greeting".to_owned()).expect("contract"));
    let hidden_requirement = ContractRequirement::provisional(key.id().clone());

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(TestProvider::new("provider", key, Arc::clone(&recorder)))
        .register_module(UndeclaredResolver::new(
            "consumer",
            hidden_requirement,
            Arc::clone(&recorder),
        ))
        .build();
    let composition = composition_for(block).expect("declarations should validate");
    let error = materialize_test_instance(&composition, "undeclared-bind")
        .expect_err("expected undeclared bind failure");

    assert!(matches!(
        error,
        CompositionError::ModuleFailure { phase: "bind", .. }
    ));
    assert_eq!(
        recorder.snapshot(),
        vec![
            "bind-error:module `consumer` attempted to resolve undeclared or unbound contract `test.greeting`",
        ]
    );
}

#[test]
fn provisional_declaration_rejects_versioned_bind_requirement() {
    let observed = Arc::new(Mutex::new(None));
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let declared = ContractRequirement::provisional(contract_id.clone());
    let bound = ContractRequirement::versioned(
        contract_id.clone(),
        ContractVersionRequirement::parse("^1").expect("requirement"),
    );
    let provider_key = ContractKey::provisional(contract_id.clone());

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(TestProvider::new("provider", provider_key, Recorder::new()))
        .register_module(MismatchedRequirementResolver::new(
            "consumer",
            declared,
            bound,
            false,
            Arc::clone(&observed),
        ))
        .build();
    let composition = composition_for(block).expect("declarations should validate");
    let error = materialize_test_instance(&composition, "provisional-versioned-bind")
        .expect_err("expected bind failure");

    assert!(matches!(
        error,
        CompositionError::ModuleFailure { phase: "bind", .. }
    ));
    assert_eq!(
        observed.lock().expect("observed lock").clone(),
        Some(ObservedRequirementMismatch {
            module_id: ModuleId::new("consumer".to_owned()).expect("module id"),
            contract_id,
            declared_compatibility: crate::ContractCompatibilityRequirement::provisional(),
            requested_compatibility: crate::ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^1").expect("requirement"),
            ),
        })
    );
}

#[test]
fn versioned_declaration_rejects_provisional_bind_requirement() {
    let observed = Arc::new(Mutex::new(None));
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let declared = ContractRequirement::versioned(
        contract_id.clone(),
        ContractVersionRequirement::parse("^1").expect("requirement"),
    );
    let bound = ContractRequirement::provisional(contract_id.clone());
    let provider_key = ContractKey::versioned(
        contract_id.clone(),
        ContractVersion::parse("1.4.0").expect("version"),
    );

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(TestProvider::new("provider", provider_key, Recorder::new()))
        .register_module(MismatchedRequirementResolver::new(
            "consumer",
            declared,
            bound,
            false,
            Arc::clone(&observed),
        ))
        .build();
    let composition = composition_for(block).expect("declarations should validate");
    let error = materialize_test_instance(&composition, "versioned-provisional-bind")
        .expect_err("expected bind failure");

    assert!(matches!(
        error,
        CompositionError::ModuleFailure { phase: "bind", .. }
    ));
    assert_eq!(
        observed.lock().expect("observed lock").clone(),
        Some(ObservedRequirementMismatch {
            module_id: ModuleId::new("consumer".to_owned()).expect("module id"),
            contract_id,
            declared_compatibility: crate::ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^1").expect("requirement"),
            ),
            requested_compatibility: crate::ContractCompatibilityRequirement::provisional(),
        })
    );
}

#[test]
fn versioned_declaration_rejects_different_bind_range() {
    let observed = Arc::new(Mutex::new(None));
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let declared = ContractRequirement::versioned(
        contract_id.clone(),
        ContractVersionRequirement::parse("^1").expect("requirement"),
    );
    let bound = ContractRequirement::versioned(
        contract_id.clone(),
        ContractVersionRequirement::parse("^2").expect("requirement"),
    );
    let provider_key = ContractKey::versioned(
        contract_id.clone(),
        ContractVersion::parse("1.4.0").expect("version"),
    );

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(TestProvider::new("provider", provider_key, Recorder::new()))
        .register_module(MismatchedRequirementResolver::new(
            "consumer",
            declared,
            bound,
            false,
            Arc::clone(&observed),
        ))
        .build();
    let composition = composition_for(block).expect("declarations should validate");
    let error = materialize_test_instance(&composition, "versioned-range-bind")
        .expect_err("expected bind failure");

    assert!(matches!(
        error,
        CompositionError::ModuleFailure { phase: "bind", .. }
    ));
    assert_eq!(
        observed.lock().expect("observed lock").clone(),
        Some(ObservedRequirementMismatch {
            module_id: ModuleId::new("consumer".to_owned()).expect("module id"),
            contract_id,
            declared_compatibility: crate::ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^1").expect("requirement"),
            ),
            requested_compatibility: crate::ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^2").expect("requirement"),
            ),
        })
    );
}

#[test]
fn optional_requirement_mismatch_is_rejected_before_absence() {
    let observed = Arc::new(Mutex::new(None));
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let declared = ContractRequirement::provisional(contract_id.clone());
    let bound = ContractRequirement::versioned(
        contract_id.clone(),
        ContractVersionRequirement::parse("^1").expect("requirement"),
    );

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(MismatchedRequirementResolver::new(
            "consumer",
            declared,
            bound,
            true,
            Arc::clone(&observed),
        ))
        .build();
    let composition = composition_for(block).expect("declarations should validate");
    let error = materialize_test_instance(&composition, "optional-bind")
        .expect_err("expected bind failure");

    assert!(matches!(
        error,
        CompositionError::ModuleFailure { phase: "bind", .. }
    ));
    assert_eq!(
        observed.lock().expect("observed lock").clone(),
        Some(ObservedRequirementMismatch {
            module_id: ModuleId::new("consumer".to_owned()).expect("module id"),
            contract_id,
            declared_compatibility: crate::ContractCompatibilityRequirement::provisional(),
            requested_compatibility: crate::ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^1").expect("requirement"),
            ),
        })
    );
}

#[test]
fn startup_failure_unwinds_dependency_chain_in_exact_reverse_order() {
    let recorder = Recorder::new();
    let contract_a =
        ContractKey::provisional(ContractId::new("test.chain.a".to_owned()).expect("contract"));
    let contract_b =
        ContractKey::provisional(ContractId::new("test.chain.b".to_owned()).expect("contract"));

    let block = BlockBuilder::new(BlockId::new("test.block".to_owned()).expect("block"))
        .register_module(ChainModule::new(
            "module-a",
            Some(contract_a.clone()),
            Vec::new(),
            Arc::clone(&recorder),
            false,
        ))
        .register_module(ChainModule::new(
            "module-b",
            Some(contract_b),
            vec![contract_a.id().clone()],
            Arc::clone(&recorder),
            false,
        ))
        .register_module(ChainModule::new(
            "module-c",
            None,
            vec![ContractId::new("test.chain.b".to_owned()).expect("contract")],
            Arc::clone(&recorder),
            true,
        ))
        .build();
    let composition = composition_for(block).expect("composition");
    let mut instance = materialize_test_instance(&composition, "test.block").expect("instance");

    let error = instance.start().expect_err("startup failure");
    assert!(matches!(
        error,
        InstanceError::ModuleFailure { phase: "start", .. }
    ));
    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
    assert_eq!(
        recorder.snapshot(),
        vec![
            "initialize:module-a",
            "initialize:module-b",
            "initialize:module-c",
            "start:module-a",
            "start:module-b",
            "start:module-c",
            "stop:module-c",
            "stop:module-b",
            "stop:module-a",
        ]
    );
}

#[test]
fn versioned_provider_resolves_when_requirement_accepts_it() {
    let recorder = Recorder::new();
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let provider_key = ContractKey::versioned(
        contract_id.clone(),
        ContractVersion::parse("1.2.0").expect("version"),
    );
    let requirement = ContractRequirement::versioned(
        contract_id,
        ContractVersionRequirement::parse("^1.1").expect("requirement"),
    );

    let block = BlockBuilder::new(BlockId::new("test.versioned".to_owned()).expect("block"))
        .register_module(TestProvider::new(
            "provider",
            provider_key,
            Arc::clone(&recorder),
        ))
        .register_module(TestConsumer::new(
            "consumer",
            requirement,
            Arc::clone(&recorder),
        ))
        .build();
    let composition = composition_for(block).expect("composition");
    let mut instance = materialize_test_instance(&composition, "test.versioned").expect("instance");

    instance.start().expect("start");
    assert_eq!(instance.report().health, Health::Healthy);
}

#[test]
fn incompatible_provider_is_reported_separately_from_missing_provider() {
    let recorder = Recorder::new();
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let provider_key = ContractKey::versioned(
        contract_id.clone(),
        ContractVersion::parse("2.0.0").expect("version"),
    );
    let requirement = ContractRequirement::versioned(
        contract_id.clone(),
        ContractVersionRequirement::parse("^1.1").expect("requirement"),
    );

    let block = BlockBuilder::new(BlockId::new("test.incompatible".to_owned()).expect("block"))
        .register_module(TestProvider::new(
            "provider",
            provider_key,
            Arc::clone(&recorder),
        ))
        .register_module(TestConsumer::new(
            "consumer",
            requirement,
            Arc::clone(&recorder),
        ))
        .build();

    let error = composition_for(block).expect_err("expected incompatible provider");
    match error {
        CompositionError::IncompatibleProvider {
            module_id,
            contract_id,
            required_compatibility,
            available_identities,
        } => {
            assert_eq!(module_id.as_str(), "consumer");
            assert_eq!(contract_id.as_str(), "fabric.test.notes");
            assert_eq!(required_compatibility.to_string(), "^1.1");
            assert_eq!(available_identities.len(), 1);
            assert_eq!(available_identities[0].to_string(), "2.0.0");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn two_compatible_versioned_providers_remain_ambiguous() {
    let recorder = Recorder::new();
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let requirement = ContractRequirement::versioned(
        contract_id.clone(),
        ContractVersionRequirement::parse("^1").expect("requirement"),
    );

    let block =
        BlockBuilder::new(BlockId::new("test.ambiguous.versioned".to_owned()).expect("block"))
            .register_module(TestProvider::new(
                "provider-a",
                ContractKey::versioned(
                    contract_id.clone(),
                    ContractVersion::parse("1.2.0").expect("version"),
                ),
                Arc::clone(&recorder),
            ))
            .register_module(TestProvider::new(
                "provider-b",
                ContractKey::versioned(
                    contract_id,
                    ContractVersion::parse("1.4.0").expect("version"),
                ),
                Arc::clone(&recorder),
            ))
            .register_module(TestConsumer::new(
                "consumer",
                requirement,
                Arc::clone(&recorder),
            ))
            .build();

    let error = composition_for(block).expect_err("expected ambiguity");
    assert!(matches!(error, CompositionError::AmbiguousProvider { .. }));
}

#[test]
fn compatible_requirement_resolves_the_only_matching_provider() {
    let recorder = Recorder::new();
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let requirement = ContractRequirement::versioned(
        contract_id.clone(),
        ContractVersionRequirement::parse("^1.1").expect("requirement"),
    );
    let observed_provider = Arc::new(Mutex::new(None));
    let observed_identity = Arc::new(Mutex::new(None));

    let block = BlockBuilder::new(BlockId::new("test.compatible".to_owned()).expect("block"))
        .register_module(TestProvider::new(
            "provider-a",
            ContractKey::versioned(
                contract_id.clone(),
                ContractVersion::parse("2.0.0").expect("version"),
            ),
            Arc::clone(&recorder),
        ))
        .register_module(TestProvider::new(
            "provider-b",
            ContractKey::versioned(
                contract_id.clone(),
                ContractVersion::parse("1.2.0").expect("version"),
            ),
            Arc::clone(&recorder),
        ))
        .register_module(ProvenanceObserver::new(
            "consumer",
            requirement,
            Arc::clone(&observed_provider),
            Arc::clone(&observed_identity),
        ))
        .build();
    let composition = composition_for(block).expect("composition");
    let mut instance =
        materialize_test_instance(&composition, "test.compatible").expect("instance");
    instance.start().expect("start");

    assert_eq!(
        observed_provider
            .lock()
            .expect("provider lock")
            .as_ref()
            .expect("provider")
            .as_str(),
        "provider-b"
    );
    assert_eq!(
        observed_identity
            .lock()
            .expect("identity lock")
            .as_ref()
            .expect("identity")
            .to_string(),
        "1.2.0"
    );
}

#[test]
fn same_module_multi_version_exports_bind_the_exact_selected_declaration() {
    fn assert_export_order(contracts: Vec<(&str, &str)>) {
        let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
        let provider_seen_v1 = Arc::new(Mutex::new(None));
        let identity_seen_v1 = Arc::new(Mutex::new(None));
        let greeting_seen_v1 = Arc::new(Mutex::new(None));
        let provider_seen_v2 = Arc::new(Mutex::new(None));
        let identity_seen_v2 = Arc::new(Mutex::new(None));
        let greeting_seen_v2 = Arc::new(Mutex::new(None));

        let provider = MultiVersionProvider::new(
            "provider",
            contracts
                .into_iter()
                .map(|(version, greeting)| {
                    (
                        ContractKey::versioned(
                            contract_id.clone(),
                            ContractVersion::parse(version).expect("version"),
                        ),
                        greeting.to_owned(),
                    )
                })
                .collect(),
        );

        let block = BlockBuilder::new(
            BlockId::new("test.same-module.multiversion".to_owned()).expect("block"),
        )
        .register_module(provider)
        .register_module(ResolutionObserver::new(
            "consumer-v1",
            ContractRequirement::versioned(
                contract_id.clone(),
                ContractVersionRequirement::parse("^1").expect("requirement"),
            ),
            Arc::clone(&provider_seen_v1),
            Arc::clone(&identity_seen_v1),
            Arc::clone(&greeting_seen_v1),
        ))
        .register_module(ResolutionObserver::new(
            "consumer-v2",
            ContractRequirement::versioned(
                contract_id,
                ContractVersionRequirement::parse("^2").expect("requirement"),
            ),
            Arc::clone(&provider_seen_v2),
            Arc::clone(&identity_seen_v2),
            Arc::clone(&greeting_seen_v2),
        ))
        .build();
        let composition = composition_for(block).expect("composition");
        let mut instance = materialize_test_instance(&composition, "test.same-module.multiversion")
            .expect("instance");
        instance.start().expect("start");

        assert_eq!(
            provider_seen_v1
                .lock()
                .expect("provider lock")
                .as_ref()
                .expect("provider")
                .as_str(),
            "provider"
        );
        assert_eq!(
            identity_seen_v1
                .lock()
                .expect("identity lock")
                .as_ref()
                .expect("identity")
                .to_string(),
            "1.5.0"
        );
        assert_eq!(
            greeting_seen_v1
                .lock()
                .expect("greeting lock")
                .as_ref()
                .expect("greeting"),
            "v1"
        );
        assert_eq!(
            provider_seen_v2
                .lock()
                .expect("provider lock")
                .as_ref()
                .expect("provider")
                .as_str(),
            "provider"
        );
        assert_eq!(
            identity_seen_v2
                .lock()
                .expect("identity lock")
                .as_ref()
                .expect("identity")
                .to_string(),
            "2.4.0"
        );
        assert_eq!(
            greeting_seen_v2
                .lock()
                .expect("greeting lock")
                .as_ref()
                .expect("greeting"),
            "v2"
        );
    }

    assert_export_order(vec![("1.5.0", "v1"), ("2.4.0", "v2")]);
    assert_export_order(vec![("2.4.0", "v2"), ("1.5.0", "v1")]);
}

#[test]
fn same_module_multi_version_broad_requirement_remains_ambiguous() {
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let block = BlockBuilder::new(
        BlockId::new("test.same-module.multiversion.ambiguous".to_owned()).expect("block"),
    )
    .register_module(MultiVersionProvider::new(
        "provider",
        vec![
            (
                ContractKey::versioned(
                    contract_id.clone(),
                    ContractVersion::parse("1.5.0").expect("version"),
                ),
                "v1".to_owned(),
            ),
            (
                ContractKey::versioned(
                    contract_id.clone(),
                    ContractVersion::parse("1.8.0").expect("version"),
                ),
                "v1.8".to_owned(),
            ),
        ],
    ))
    .register_module(TestConsumer::new(
        "consumer",
        ContractRequirement::versioned(
            contract_id,
            ContractVersionRequirement::parse("^1").expect("requirement"),
        ),
        Recorder::new(),
    ))
    .build();

    assert!(matches!(
        composition_for(block),
        Err(CompositionError::AmbiguousProvider { .. })
    ));
}

#[test]
fn provisional_and_versioned_contract_rules_are_explicit() {
    let recorder = Recorder::new();
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");

    let provisional_block =
        BlockBuilder::new(BlockId::new("test.provisional".to_owned()).expect("block"))
            .register_module(TestProvider::new(
                "provider",
                ContractKey::provisional(contract_id.clone()),
                Arc::clone(&recorder),
            ))
            .register_module(TestConsumer::new(
                "consumer",
                ContractRequirement::provisional(contract_id.clone()),
                Arc::clone(&recorder),
            ))
            .build();
    let composition = composition_for(provisional_block).expect("composition");
    let mut instance =
        materialize_test_instance(&composition, "test.provisional").expect("instance");
    instance.start().expect("start");

    let versioned_requirement = BlockBuilder::new(
        BlockId::new("test.provisional.incompatible.a".to_owned()).expect("block"),
    )
    .register_module(TestProvider::new(
        "provider",
        ContractKey::provisional(contract_id.clone()),
        Arc::clone(&recorder),
    ))
    .register_module(TestConsumer::new(
        "consumer",
        ContractRequirement::versioned(
            contract_id.clone(),
            ContractVersionRequirement::parse("^1").expect("requirement"),
        ),
        Arc::clone(&recorder),
    ))
    .build();
    assert!(matches!(
        composition_for(versioned_requirement),
        Err(CompositionError::IncompatibleProvider { .. })
    ));

    let provisional_requirement = BlockBuilder::new(
        BlockId::new("test.provisional.incompatible.b".to_owned()).expect("block"),
    )
    .register_module(TestProvider::new(
        "provider",
        ContractKey::versioned(
            contract_id,
            ContractVersion::parse("1.0.0").expect("version"),
        ),
        Arc::clone(&recorder),
    ))
    .register_module(TestConsumer::new(
        "consumer",
        ContractRequirement::provisional(
            ContractId::new("fabric.test.notes".to_owned()).expect("contract"),
        ),
        Arc::clone(&recorder),
    ))
    .build();
    assert!(matches!(
        composition_for(provisional_requirement),
        Err(CompositionError::IncompatibleProvider { .. })
    ));
}

#[test]
fn optional_incompatible_provider_resolves_as_absent() {
    let observed = Arc::new(Mutex::new(Some("seed".to_owned())));
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let requirement = ContractRequirement::versioned(
        contract_id.clone(),
        ContractVersionRequirement::parse("^1").expect("requirement"),
    );

    let block = BlockBuilder::new(BlockId::new("test.optional".to_owned()).expect("block"))
        .register_module(TestProvider::new(
            "provider",
            ContractKey::versioned(
                contract_id,
                ContractVersion::parse("2.0.0").expect("version"),
            ),
            Recorder::new(),
        ))
        .register_module(OptionalObserver::new(
            "observer",
            requirement,
            Arc::clone(&observed),
        ))
        .build();
    let composition = composition_for(block).expect("composition");
    let mut instance = materialize_test_instance(&composition, "test.optional").expect("instance");
    instance.start().expect("start");

    assert_eq!(*observed.lock().expect("observed lock"), None);
}

#[test]
fn different_consumers_resolve_the_same_contract_id_to_different_compatible_providers() {
    fn assert_order(order: [&str; 4]) {
        let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
        let recorder = Recorder::new();
        let a_provider = TestProvider::new(
            "provider-v1",
            ContractKey::versioned(
                contract_id.clone(),
                ContractVersion::parse("1.5.0").expect("version"),
            ),
            Arc::clone(&recorder),
        )
        .with_greeting("v1");
        let b_provider = TestProvider::new(
            "provider-v2",
            ContractKey::versioned(
                contract_id.clone(),
                ContractVersion::parse("2.4.0").expect("version"),
            ),
            Arc::clone(&recorder),
        )
        .with_greeting("v2");
        let a_provider_seen = Arc::new(Mutex::new(None));
        let a_identity_seen = Arc::new(Mutex::new(None));
        let b_provider_seen = Arc::new(Mutex::new(None));
        let b_identity_seen = Arc::new(Mutex::new(None));

        let consumer_v1 = ProvenanceObserver::new(
            "consumer-v1",
            ContractRequirement::versioned(
                contract_id.clone(),
                ContractVersionRequirement::parse("^1").expect("requirement"),
            ),
            Arc::clone(&a_provider_seen),
            Arc::clone(&a_identity_seen),
        );
        let consumer_v2 = ProvenanceObserver::new(
            "consumer-v2",
            ContractRequirement::versioned(
                contract_id,
                ContractVersionRequirement::parse("^2").expect("requirement"),
            ),
            Arc::clone(&b_provider_seen),
            Arc::clone(&b_identity_seen),
        );

        let mut builder =
            BlockBuilder::new(BlockId::new("test.consumer-scoped".to_owned()).expect("block"));
        for item in order {
            builder = match item {
                "provider-v1" => builder.register_module(a_provider.clone()),
                "provider-v2" => builder.register_module(b_provider.clone()),
                "consumer-v1" => builder.register_module(consumer_v1.clone()),
                "consumer-v2" => builder.register_module(consumer_v2.clone()),
                other => panic!("unexpected order item: {other}"),
            };
        }
        let block = builder.build();

        let composition = composition_for(block).expect("composition");
        let mut instance =
            materialize_test_instance(&composition, "test.consumer-scoped").expect("instance");
        instance.start().expect("start");

        assert_eq!(
            a_provider_seen
                .lock()
                .expect("provider lock")
                .as_ref()
                .expect("provider")
                .as_str(),
            "provider-v1"
        );
        assert_eq!(
            a_identity_seen
                .lock()
                .expect("identity lock")
                .as_ref()
                .expect("identity")
                .to_string(),
            "1.5.0"
        );
        assert_eq!(
            b_provider_seen
                .lock()
                .expect("provider lock")
                .as_ref()
                .expect("provider")
                .as_str(),
            "provider-v2"
        );
        assert_eq!(
            b_identity_seen
                .lock()
                .expect("identity lock")
                .as_ref()
                .expect("identity")
                .to_string(),
            "2.4.0"
        );
    }

    assert_order(["provider-v1", "provider-v2", "consumer-v1", "consumer-v2"]);
    assert_order(["consumer-v2", "provider-v2", "consumer-v1", "provider-v1"]);
}

#[test]
fn provisional_and_versioned_consumers_resolve_independently_per_consumer() {
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let provisional_provider_seen = Arc::new(Mutex::new(None));
    let provisional_identity_seen = Arc::new(Mutex::new(None));
    let versioned_provider_seen = Arc::new(Mutex::new(None));
    let versioned_identity_seen = Arc::new(Mutex::new(None));

    let block =
        BlockBuilder::new(BlockId::new("test.consumer-scoped.mixed".to_owned()).expect("block"))
            .register_module(TestProvider::new(
                "provider-provisional",
                ContractKey::provisional(contract_id.clone()),
                Recorder::new(),
            ))
            .register_module(TestProvider::new(
                "provider-versioned",
                ContractKey::versioned(
                    contract_id.clone(),
                    ContractVersion::parse("1.8.0").expect("version"),
                ),
                Recorder::new(),
            ))
            .register_module(ProvenanceObserver::new(
                "consumer-provisional",
                ContractRequirement::provisional(contract_id.clone()),
                Arc::clone(&provisional_provider_seen),
                Arc::clone(&provisional_identity_seen),
            ))
            .register_module(ProvenanceObserver::new(
                "consumer-versioned",
                ContractRequirement::versioned(
                    contract_id,
                    ContractVersionRequirement::parse("^1").expect("requirement"),
                ),
                Arc::clone(&versioned_provider_seen),
                Arc::clone(&versioned_identity_seen),
            ))
            .build();
    let composition = composition_for(block).expect("composition");
    let mut instance =
        materialize_test_instance(&composition, "test.consumer-scoped.mixed").expect("instance");
    instance.start().expect("start");

    assert_eq!(
        provisional_provider_seen
            .lock()
            .expect("provider lock")
            .as_ref()
            .expect("provider")
            .as_str(),
        "provider-provisional"
    );
    assert_eq!(
        provisional_identity_seen
            .lock()
            .expect("identity lock")
            .as_ref()
            .expect("identity"),
        &ContractIdentity::Provisional
    );
    assert_eq!(
        versioned_provider_seen
            .lock()
            .expect("provider lock")
            .as_ref()
            .expect("provider")
            .as_str(),
        "provider-versioned"
    );
    assert_eq!(
        versioned_identity_seen
            .lock()
            .expect("identity lock")
            .as_ref()
            .expect("identity")
            .to_string(),
        "1.8.0"
    );
}

#[test]
fn optional_resolution_remains_scoped_per_consumer() {
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let optional_seen = Arc::new(Mutex::new(Some("seed".to_owned())));
    let required_provider_seen = Arc::new(Mutex::new(None));
    let required_identity_seen = Arc::new(Mutex::new(None));

    let block =
        BlockBuilder::new(BlockId::new("test.optional.consumer-scoped".to_owned()).expect("block"))
            .register_module(TestProvider::new(
                "provider-v2",
                ContractKey::versioned(
                    contract_id.clone(),
                    ContractVersion::parse("2.0.0").expect("version"),
                ),
                Recorder::new(),
            ))
            .register_module(OptionalObserver::new(
                "optional-consumer",
                ContractRequirement::versioned(
                    contract_id.clone(),
                    ContractVersionRequirement::parse("^1").expect("requirement"),
                ),
                Arc::clone(&optional_seen),
            ))
            .register_module(ProvenanceObserver::new(
                "required-consumer",
                ContractRequirement::versioned(
                    contract_id,
                    ContractVersionRequirement::parse("^2").expect("requirement"),
                ),
                Arc::clone(&required_provider_seen),
                Arc::clone(&required_identity_seen),
            ))
            .build();
    let composition = composition_for(block).expect("composition");
    let mut instance =
        materialize_test_instance(&composition, "test.optional.consumer-scoped").expect("instance");
    instance.start().expect("start");

    assert_eq!(*optional_seen.lock().expect("optional lock"), None);
    assert_eq!(
        required_provider_seen
            .lock()
            .expect("provider lock")
            .as_ref()
            .expect("provider")
            .as_str(),
        "provider-v2"
    );
    assert_eq!(
        required_identity_seen
            .lock()
            .expect("identity lock")
            .as_ref()
            .expect("identity")
            .to_string(),
        "2.0.0"
    );
}

#[test]
fn duplicate_required_contract_ids_are_rejected_per_consumer_module() {
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let block = BlockBuilder::new(
        BlockId::new("test.duplicate.requirement.required".to_owned()).expect("block"),
    )
    .register_module(DuplicateRequirementConsumer::new(
        "consumer",
        vec![
            ContractRequirementDeclaration::versioned(
                contract_id.clone(),
                ContractVersionRequirement::parse("^1").expect("requirement"),
            ),
            ContractRequirementDeclaration::versioned(
                contract_id.clone(),
                ContractVersionRequirement::parse("^2").expect("requirement"),
            ),
        ],
        Vec::new(),
    ))
    .build();

    match composition_for(block) {
        Err(CompositionError::DuplicateContractRequirement {
            module_id,
            contract_id,
        }) => {
            assert_eq!(module_id.as_str(), "consumer");
            assert_eq!(contract_id.as_str(), "fabric.test.notes");
        }
        other => panic!("unexpected duplicate requirement result: {other:?}"),
    }
}

#[test]
fn duplicate_required_and_optional_contract_ids_are_rejected_per_consumer_module() {
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let block = BlockBuilder::new(
        BlockId::new("test.duplicate.requirement.optional".to_owned()).expect("block"),
    )
    .register_module(DuplicateRequirementConsumer::new(
        "consumer",
        vec![ContractRequirementDeclaration::versioned(
            contract_id.clone(),
            ContractVersionRequirement::parse("^1").expect("requirement"),
        )],
        vec![ContractRequirementDeclaration::versioned(
            contract_id.clone(),
            ContractVersionRequirement::parse("^2").expect("requirement"),
        )],
    ))
    .build();

    match composition_for(block) {
        Err(CompositionError::DuplicateContractRequirement {
            module_id,
            contract_id,
        }) => {
            assert_eq!(module_id.as_str(), "consumer");
            assert_eq!(contract_id.as_str(), "fabric.test.notes");
        }
        other => panic!("unexpected duplicate requirement result: {other:?}"),
    }
}

#[test]
fn type_mismatch_remains_rejected_with_versioned_contracts() {
    let contract_id = ContractId::new("fabric.test.notes".to_owned()).expect("contract");
    let block = BlockBuilder::new(BlockId::new("test.type-mismatch".to_owned()).expect("block"))
        .register_module(TestProvider::new(
            "provider",
            ContractKey::versioned(
                contract_id.clone(),
                ContractVersion::parse("1.2.0").expect("version"),
            ),
            Recorder::new(),
        ))
        .register_module(WrongTypeConsumer::new(
            "consumer",
            ContractRequirement::versioned(
                contract_id,
                ContractVersionRequirement::parse("^1").expect("requirement"),
            ),
        ))
        .build();

    let composition = composition_for(block).expect("declarations should validate");
    assert!(matches!(
        materialize_test_instance(&composition, "type-mismatch"),
        Err(CompositionError::ContractTypeMismatch { .. })
            | Err(CompositionError::ModuleFailure { .. })
    ));
}

#[derive(Clone)]
struct PreparedGreeter {
    prepared: Arc<Mutex<Option<String>>>,
}

impl GreetingPort for PreparedGreeter {
    fn greet(&self) -> String {
        self.prepared
            .lock()
            .expect("prepared lock")
            .clone()
            .expect("provider must be bound before use")
    }
}

#[derive(Clone)]
struct ChainedGreeter {
    source: Arc<Mutex<Option<Arc<TestContract>>>>,
    suffix: String,
}

impl GreetingPort for ChainedGreeter {
    fn greet(&self) -> String {
        let source = self
            .source
            .lock()
            .expect("source lock")
            .clone()
            .expect("dependency must be bound before use");
        format!("{}+{}", source.greet(), self.suffix)
    }
}

struct BindChainLink {
    module_id: ModuleId,
    provides: ContractKey<TestContract>,
    requires: ContractRequirement<TestContract>,
    recorder: Arc<Recorder>,
    prepared: Arc<Mutex<Option<String>>>,
    chained: Arc<Mutex<Option<Arc<TestContract>>>>,
    prepare_value: Option<String>,
}

impl BindChainLink {
    fn provider(
        module_id: &str,
        provides: ContractKey<TestContract>,
        prepare_value: &str,
        recorder: Arc<Recorder>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            provides: provides.clone(),
            requires: ContractRequirement::provisional(
                ContractId::new("test.bind-chain.unused".to_owned()).expect("contract"),
            ),
            recorder,
            prepared: Arc::new(Mutex::new(None)),
            chained: Arc::new(Mutex::new(None)),
            prepare_value: Some(prepare_value.to_owned()),
        }
    }

    fn middle(
        module_id: &str,
        provides: ContractKey<TestContract>,
        requires: ContractRequirement<TestContract>,
        recorder: Arc<Recorder>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            provides: provides.clone(),
            requires,
            recorder,
            prepared: Arc::new(Mutex::new(None)),
            chained: Arc::new(Mutex::new(None)),
            prepare_value: None,
        }
    }
}

impl ModuleRuntime for BindChainLink {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<ProvidedContractDeclaration> {
        vec![self.provides.declaration()]
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        if self.prepare_value.is_some() {
            Vec::new()
        } else {
            vec![self.requires.declaration().clone()]
        }
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        let service: Arc<dyn GreetingPort> = if self.prepare_value.is_some() {
            Arc::new(PreparedGreeter {
                prepared: Arc::clone(&self.prepared),
            })
        } else {
            Arc::new(ChainedGreeter {
                source: Arc::clone(&self.chained),
                suffix: self.module_id.as_str().to_owned(),
            })
        };
        Ok(vec![ModuleContract::new(
            &self.provides,
            Arc::new(TestContract { greeter: service }),
        )])
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        // Providers prepare live state; middles resolve and retain their
        // dependency. Neither consults authoring or Block order: bindings are
        // already resolved, but invoking them requires the provider bound first.
        if let Some(value) = &self.prepare_value {
            *self.prepared.lock().expect("prepared lock") = Some(value.clone());
        } else {
            let resolved: Arc<TestContract> = bindings
                .resolve(&self.requires)
                .map_err(|error| ModuleError::new(error.to_string()))?;
            *self.chained.lock().expect("chained lock") = Some(resolved);
        }
        self.recorder
            .push(format!("bind:{}", self.module_id.as_str()));
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("initialize:{}", self.module_id.as_str()));
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("start:{}", self.module_id.as_str()));
        Ok(())
    }

    fn stop(&mut self) {
        self.recorder
            .push(format!("stop:{}", self.module_id.as_str()));
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

impl Module for BindChainLink {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self {
            module_id: self.module_id.clone(),
            provides: self.provides.clone(),
            requires: self.requires.clone(),
            recorder: Arc::clone(&self.recorder),
            prepared: Arc::clone(&self.prepared),
            chained: Arc::clone(&self.chained),
            prepare_value: self.prepare_value.clone(),
        }))
    }
}

struct BindChainConsumer {
    module_id: ModuleId,
    requires: ContractRequirement<TestContract>,
    recorder: Arc<Recorder>,
    observed: Arc<Mutex<Option<String>>>,
}

impl BindChainConsumer {
    fn new(
        module_id: &str,
        requires: ContractRequirement<TestContract>,
        recorder: Arc<Recorder>,
        observed: Arc<Mutex<Option<String>>>,
    ) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
            requires,
            recorder,
            observed,
        }
    }
}

impl ModuleRuntime for BindChainConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
        Vec::new()
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        vec![self.requires.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        // Invoking the provider during our own bind proves the provider was
        // already bound: its behavior depends on its own bound dependency.
        let resolved: Arc<TestContract> = bindings
            .resolve(&self.requires)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.observed.lock().expect("observed lock") = Some(resolved.greet());
        self.recorder
            .push(format!("bind:{}", self.module_id.as_str()));
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("initialize:{}", self.module_id.as_str()));
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.recorder
            .push(format!("start:{}", self.module_id.as_str()));
        Ok(())
    }

    fn stop(&mut self) {
        self.recorder
            .push(format!("stop:{}", self.module_id.as_str()));
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

impl Module for BindChainConsumer {
    fn declaration(&self) -> crate::ModuleDeclaration {
        crate::ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(Self::new(
            self.module_id.as_str(),
            self.requires.clone(),
            Arc::clone(&self.recorder),
            Arc::clone(&self.observed),
        )))
    }
}

#[test]
fn runtime_binding_follows_dependency_order_across_reverse_authored_blocks() {
    let recorder = Recorder::new();
    let observed = Arc::new(Mutex::new(None));
    let key_a: ContractKey<TestContract> = ContractKey::provisional(
        ContractId::new("test.bind-chain.a".to_owned()).expect("contract"),
    );
    let key_b: ContractKey<TestContract> = ContractKey::provisional(
        ContractId::new("test.bind-chain.b".to_owned()).expect("contract"),
    );

    // Deliberately authored consumer-first across three Blocks: the
    // dependency graph still points A -> B -> C.
    let block_c = BlockBuilder::new(BlockId::new("test.bind-chain.c".to_owned()).expect("block"))
        .register_module(BindChainConsumer::new(
            "consumer",
            ContractRequirement::provisional(key_b.id().clone()),
            Arc::clone(&recorder),
            Arc::clone(&observed),
        ))
        .build();
    let block_b = BlockBuilder::new(BlockId::new("test.bind-chain.b".to_owned()).expect("block"))
        .register_module(BindChainLink::middle(
            "middle",
            key_b,
            ContractRequirement::provisional(
                ContractId::new("test.bind-chain.a".to_owned()).expect("contract"),
            ),
            Arc::clone(&recorder),
        ))
        .build();
    let block_a = BlockBuilder::new(BlockId::new("test.bind-chain.a".to_owned()).expect("block"))
        .register_module(BindChainLink::provider(
            "provider",
            key_a,
            "a-value",
            Arc::clone(&recorder),
        ))
        .build();

    let composition = CompositionBuilder::new(
        CompositionId::new("test.bind-chain".to_owned()).expect("composition"),
    )
    .register_block(block_c)
    .register_block(block_b)
    .register_block(block_a)
    .build()
    .expect("acyclic declarations validate regardless of order");

    let mut instance =
        materialize_test_instance(&composition, "test.bind-chain").expect("instance");
    instance.start().expect("start");
    instance.stop();

    assert_eq!(
        observed.lock().expect("observed lock").clone(),
        Some("a-value+middle".to_owned()),
        "consumer bind must observe the fully bound chain"
    );
    assert_eq!(
        recorder.snapshot(),
        vec![
            "bind:provider".to_owned(),
            "bind:middle".to_owned(),
            "bind:consumer".to_owned(),
            "initialize:provider".to_owned(),
            "initialize:middle".to_owned(),
            "initialize:consumer".to_owned(),
            "start:provider".to_owned(),
            "start:middle".to_owned(),
            "start:consumer".to_owned(),
            "stop:consumer".to_owned(),
            "stop:middle".to_owned(),
            "stop:provider".to_owned(),
        ]
    );
}
