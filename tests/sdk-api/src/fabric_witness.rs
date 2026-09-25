use std::sync::{Arc, Mutex};

use fabric::authoring::*;
use fabric::component::declaration::ComponentRelationName;
use fabric::prelude::*;
use fabric::{
    core::{ContractProviderSelection, ContractRequirement},
    ids::module,
};
use fabric_component::{ComponentError, ComponentMaterializer, ComponentParticipationScope};
use fabric_core::{
    BlockBuilder, BlockId, CompositionError, ContractId, ContractKey, ContractVersionRequirement,
    Health, LifecycleState, ModuleBindings, ModuleContract, ModuleDeclaration, ModuleError,
    ModuleId, ModuleRuntime,
};
use fabric_system::{SystemId, SystemSchemaDescriptor};
use fabric_test_adapter_clock_memory::{
    HostBoundClockAdapter, MemoryClock, host_bound_clock_facility,
};
use fabric_test_component_greeter::{
    Greeter, GreeterConfig, GreeterInput, GreeterOutput, PackageComponent, PackageComponentConfig,
    greeter,
};
use fabric_test_resource_clock::{Clock, ClockConfig};
use fabric_test_resource_counter::{
    AdaptedCounter, AdaptedCounterConfig, DerivedCounter, DerivedCounterConfig, DirectCounter,
    DirectCounterConfig, FixedCounterAdapter, FixedCounterAdapterConfig, PackageOnlyConfig,
    PackageOnlyResource,
};
use fabric_test_system_operations::{
    AdaptedOperations, AdaptedOperationsConfig, FixedOperationsAdapter,
    FixedOperationsAdapterConfig, HostBoundOperationsAdapter, HostBoundOperationsAdapterConfig,
    OperationMarker, OperationsContract, OperationsService, TestOperations, TestOperationsConfig,
    host_bound_operations_facility, operations_contract_key,
};

fn test_host() -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("arch"),
    )
}

fn facility_host() -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("arch"),
    )
    .with_facility(host_bound_clock_facility())
}
fn alpha_host() -> HostDescriptor {
    facility_host().with_facility(host_bound_operations_facility())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MacroDependencyInput;

#[derive(Clone, Debug, PartialEq, Eq)]
struct MacroDependencyOutput {
    value: u64,
}

fabric::component! {
    MacroDependencyProbe {
        id: "fabric.test.component.macro-dependency-probe";

        config {
            multiplier: u64;
        }

        relations {
            requires {
                counter: DirectCounter(provisional);
                operations: TestOperations(version = "^1.2");
            }
        }
        api { fn observe(&self, input: MacroDependencyInput) -> MacroDependencyOutput; }
        runtime {
            fn observe(&self, input: MacroDependencyInput) -> MacroDependencyOutput {
                let _ = input;
                MacroDependencyOutput {
                    value: self.relations().counter.current_value().value()
                        + self.relations().operations.current_marker().value()
                        + self.config().multiplier,
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GatewayInput;
#[derive(Clone, Debug, PartialEq, Eq)]
struct GatewayOutput {
    value: String,
}

fabric::component! {
    Gateway {
        id: "fabric.test.gateway";
        config { prefix: String; }
        api { fn handle(&self, input: GatewayInput) -> GatewayOutput; }
        runtime {
            fn handle(&self, input: GatewayInput) -> GatewayOutput {
                let _ = input;
                GatewayOutput { value: "gateway:native".to_owned() }
            }
        }
    }
}

#[derive(Clone)]
struct PingoraAdapter {
    implementation: String,
}

#[derive(Clone)]
struct AlternateGatewayAdapter {
    implementation: String,
}

impl AdapterDefinition for PingoraAdapter {
    fn adapter_definition_id(&self) -> fabric::authoring::AdapterDefinitionId {
        fabric::authoring::AdapterDefinitionId::new("test.manual.pingora-adapter")
            .expect("static test Adapter definition ID is valid")
    }

    type Target = Gateway;
    type Compatibility = ComponentId;
    fn compatibility(&self) -> ComponentId {
        Gateway::component_id()
    }
    fn host_requirement(&self) -> HostRequirement {
        HostRequirement::new().require_facility(host_bound_clock_facility())
    }
    fn declaration(&self, module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(module_id).with_provided_contracts(vec![
            ContractKey::<ComponentRealizationContract<Gateway>>::provisional(
                Gateway::realization_requirement().id().clone(),
            )
            .declaration(),
        ])
    }
    fn materialize_provider(&self, module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(PingoraRuntime {
            module_id,
            implementation: self.implementation.clone(),
        }))
    }
}

impl AdapterDefinition for AlternateGatewayAdapter {
    fn adapter_definition_id(&self) -> fabric::authoring::AdapterDefinitionId {
        fabric::authoring::AdapterDefinitionId::new("test.manual.alternate-gateway-adapter")
            .expect("static test Adapter definition ID is valid")
    }

    type Target = Gateway;
    type Compatibility = ComponentId;
    fn compatibility(&self) -> ComponentId {
        Gateway::component_id()
    }
    fn host_requirement(&self) -> HostRequirement {
        HostRequirement::new().require_facility(host_bound_clock_facility())
    }
    fn declaration(&self, module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(module_id).with_provided_contracts(vec![
            ContractKey::<ComponentRealizationContract<Gateway>>::provisional(
                Gateway::realization_requirement().id().clone(),
            )
            .declaration(),
        ])
    }
    fn materialize_provider(&self, module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(PingoraRuntime {
            module_id,
            implementation: self.implementation.clone(),
        }))
    }
}
struct PingoraRuntime {
    module_id: ModuleId,
    implementation: String,
}
impl ModuleRuntime for PingoraRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }
    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![
            ContractKey::<ComponentRealizationContract<Gateway>>::provisional(
                Gateway::realization_requirement().id().clone(),
            )
            .declaration(),
        ]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        let implementation = self.implementation.clone();
        Ok(vec![ModuleContract::new(
            &ContractKey::provisional(Gateway::realization_requirement().id().clone()),
            Arc::new(ComponentRealizationContract::<Gateway>::new(
                move |config, scope| {
                    let prefix = config.prefix.clone();
                    let implementation = implementation.clone();
                    scope.operation(gateway::api::handle(), move |input: GatewayInput| {
                        let _ = input;
                        let prefix = prefix.clone();
                        let implementation = implementation.clone();
                        async move {
                            Ok(GatewayOutput {
                                value: format!("{prefix}:{implementation}"),
                            })
                        }
                    })?;
                    Ok(Health::Healthy)
                },
            )),
        )])
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

#[derive(Clone)]
struct GatewayAudit;
#[derive(Clone)]
struct GatewayAuditService;

impl ComponentAugmentationDefinition<Gateway> for GatewayAudit {
    type Config = ();
    type Contract = GatewayAuditService;

    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(ContractId::new("fabric.test.gateway.audit").expect("contract"))
    }
}

#[derive(Clone)]
struct GatewayTrace;
#[derive(Clone)]
struct GatewayTraceService;

impl ComponentAugmentationDefinition<Gateway> for GatewayTrace {
    type Config = ();
    type Contract = GatewayTraceService;

    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(ContractId::new("fabric.test.gateway.trace").expect("contract"))
    }
}

#[derive(Clone)]
struct GatewayAuditSupport;

struct GatewayAuditRuntime {
    module_id: ModuleId,
}

struct GatewayTraceRuntime {
    module_id: ModuleId,
}

impl ModuleRuntime for GatewayTraceRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &GatewayTrace::contract_key(),
            Arc::new(GatewayTraceService),
        )])
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

impl ModuleRuntime for GatewayAuditRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &GatewayAudit::contract_key(),
            Arc::new(GatewayAuditService),
        )])
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

impl ComponentAugmentationSupportDefinition<Gateway, GatewayAudit> for GatewayAuditSupport {
    fn declaration(&self, module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(module_id)
    }

    fn materialize(&self, _: &(), module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(GatewayAuditRuntime { module_id }))
    }

    fn prepare(&self, _: &(), _: &ComponentParticipationScope) -> Result<(), ComponentError> {
        Ok(())
    }
}

#[derive(Clone)]
struct GatewayTraceSupport;

impl ComponentAugmentationSupportDefinition<Gateway, GatewayTrace> for GatewayTraceSupport {
    fn declaration(&self, module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(module_id)
    }

    fn materialize(&self, _: &(), module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(GatewayTraceRuntime { module_id }))
    }

    fn prepare(&self, _: &(), _: &ComponentParticipationScope) -> Result<(), ComponentError> {
        Ok(())
    }
}

/// A deliberately failing runtime used to prove that a failed new generation
/// does not affect an already-running generation. It is an ordinary module,
/// not replacement machinery.
#[derive(Clone)]
struct FailingGenerationStart {
    module_id: ModuleId,
}

impl FailingGenerationStart {
    fn new() -> Self {
        Self {
            module_id: module("fabric.test.generational-change.failing-start").expect("module id"),
        }
    }
}

impl ModuleRuntime for FailingGenerationStart {
    fn id(&self) -> &ModuleId {
        &self.module_id
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
        Err(ModuleError::new("forced new-generation startup failure"))
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OpenDocumentInput {
    exists: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DocumentOutput {
    title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DocumentError {
    NotFound,
}

fabric::component! {
    DocumentProbe {
        id: "fabric.test.component.document-probe";
        api { fn open(&self, input: OpenDocumentInput) -> Result<DocumentOutput, DocumentError>; }
        runtime {
            fn open(&self, input: OpenDocumentInput) -> Result<DocumentOutput, DocumentError> {
                if input.exists {
                    Ok(DocumentOutput { title: "Fabric".to_owned() })
                } else {
                    Err(DocumentError::NotFound)
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DualCounterInput;
#[derive(Clone, Debug, PartialEq, Eq)]
struct DualCounterOutput {
    primary: u64,
    secondary: u64,
}

fabric::component! {
    DualCounterProbe {
        id: "fabric.test.component.dual-counter-probe";
        relations {
            requires {
                primary_store: DirectCounter(provisional);
                secondary_store: DirectCounter(provisional);
            }
        }
        api { fn observe(&self, input: DualCounterInput) -> DualCounterOutput; }
        runtime {
            fn observe(&self, input: DualCounterInput) -> DualCounterOutput {
                let _ = input;
                DualCounterOutput {
                    primary: self.relations().primary_store.current_value().value(),
                    secondary: self.relations().secondary_store.current_value().value(),
                }
            }
        }
    }
}

#[derive(Clone)]
struct DirectConsumer {
    module_id: ModuleId,
    requirement: Requires<DirectCounter>,
    capture: Arc<Mutex<Option<u64>>>,
}

impl DirectConsumer {
    fn new(module_id: &str, capture: Arc<Mutex<Option<u64>>>) -> Self {
        Self {
            module_id: module(module_id).expect("module id"),
            requirement: Requires::<DirectCounter>::provisional(),
            capture,
        }
    }
}

impl ModuleRuntime for DirectConsumer {
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
        *self.capture.lock().expect("capture lock") =
            Some(resolved.value().current_value().value());
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
struct OperationsConsumer {
    module_id: ModuleId,
    requirement: SystemRequires<TestOperations>,
    capture: Arc<Mutex<Option<u64>>>,
}

impl OperationsConsumer {
    fn new(module_id: &str, capture: Arc<Mutex<Option<u64>>>) -> Self {
        Self {
            module_id: module(module_id).expect("module id"),
            requirement: SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
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
        *self.capture.lock().expect("capture lock") =
            Some(resolved.value().current_marker().value());
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
struct AdaptedConsumer {
    module_id: ModuleId,
    requirement: Requires<AdaptedCounter>,
    capture: Arc<Mutex<Option<u64>>>,
}

impl AdaptedConsumer {
    fn new(module_id: &str, capture: Arc<Mutex<Option<u64>>>) -> Self {
        Self {
            module_id: module(module_id).expect("module id"),
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
        *self.capture.lock().expect("capture lock") =
            Some(resolved.value().current_value().value());
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
struct ClockConsumer {
    module_id: ModuleId,
    requirement: Requires<Clock>,
    capture: Arc<Mutex<Option<u64>>>,
}

impl ClockConsumer {
    fn new(capture: Arc<Mutex<Option<u64>>>) -> Self {
        Self {
            module_id: module("fabric.test.fabric.clock.consumer").expect("module id"),
            requirement: Requires::<Clock>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
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
        *self.capture.lock().expect("capture lock") = Some(tick.value());
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
struct RailsCapture {
    materializer: Arc<ComponentMaterializer>,
}

#[derive(Clone)]
struct RailsCaptureModule {
    module_id: ModuleId,
    capture: Arc<Mutex<Option<RailsCapture>>>,
}

#[derive(Clone)]
struct CompetingOperationsProvider {
    module_id: ModuleId,
    marker: u64,
}
struct CompetingOperationsContract(u64);
impl OperationsService for CompetingOperationsContract {
    fn current_marker(&self) -> OperationMarker {
        OperationMarker::new(self.0)
    }
}
impl CompetingOperationsProvider {
    fn new(marker: u64) -> Self {
        Self {
            module_id: module("fabric.test.competing-system-provider").expect("module"),
            marker,
        }
    }
}
impl ModuleRuntime for CompetingOperationsProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }
    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![operations_contract_key().declaration()]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &operations_contract_key(),
            Arc::new(OperationsContract::new(Arc::new(
                CompetingOperationsContract(self.marker),
            ))),
        )])
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

impl RailsCaptureModule {
    fn new(capture: Arc<Mutex<Option<RailsCapture>>>) -> Self {
        Self {
            module_id: module("fabric.test.fabric.rails.capture").expect("module id"),
            capture,
        }
    }
}

impl ModuleRuntime for RailsCaptureModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![
            ContractRequirement::<ComponentMaterializer>::provisional(
                fabric_component::component_materializer_contract_id(),
            )
            .declaration()
            .clone(),
        ]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let materializer_requirement = ContractRequirement::<ComponentMaterializer>::provisional(
            fabric_component::component_materializer_contract_id(),
        );
        *self.capture.lock().expect("capture lock") = Some(RailsCapture {
            materializer: bindings
                .resolve(&materializer_requirement)
                .map_err(|error| ModuleError::new(error.to_string()))?,
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
struct PackageNotesContract;

struct PackageNotesSystem;

#[derive(Clone)]
struct PackageNotesConfig {
    #[allow(dead_code)]
    note: String,
}

impl PrimarySystemContract for PackageNotesSystem {
    type Contract = PackageNotesContract;

    fn primary_contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.package-notes.capability").expect("contract id"),
        )
    }
}

impl SystemDefinition for PackageNotesSystem {
    type Config = PackageNotesConfig;

    fn system_id() -> SystemId {
        SystemId::new("fabric.test.package-notes").expect("system id")
    }

    fn schema() -> SystemSchemaDescriptor {
        SystemSchemaDescriptor::provisional(Self::system_id())
    }

    fn declaration(selection: &SystemSelection<Self>) -> ModuleDeclaration {
        ModuleDeclaration::new(selection.module_id().clone())
            .with_provided_contracts(vec![Self::primary_contract_key().declaration()])
    }
}

#[test]
fn fabric_builds_two_adapted_contributions_without_raw_parts_ceremony() {
    let counter_capture = Arc::new(Mutex::new(None));
    let adapted = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(FixedCounterAdapter::new(FixedCounterAdapterConfig {
            value: 10,
        }))
        .expect("adapter");
    let adapted_operations = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("selection")
        .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
            value: 7,
        }))
        .expect("adapter");

    let built = Fabric::new("fabric.test.fabric.two-adapted")
        .expect("fabric")
        .resource(adapted)
        .system(adapted_operations)
        .block("consumer", |block| {
            block.module(AdaptedConsumer::new(
                "fabric.test.fabric.adapted.consumer",
                Arc::clone(&counter_capture),
            ))
        })
        .expect("consumer block")
        .build()
        .expect("typed contributions build without raw-parts ceremony");

    assert_eq!(built.manifest().resources().len(), 1);
    assert_eq!(built.manifest().systems().len(), 1);
    assert!(
        built
            .manifest()
            .diagnostics()
            .provider_selections()
            .is_empty()
    );

    let mut instance = built
        .materialize_on("fabric.test.fabric.two-adapted.instance", &test_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(counter_capture.lock().expect("capture").clone(), Some(10));
}

#[test]
fn fabric_manifest_exposes_typed_semantic_knowledge() {
    let built = Fabric::new("fabric.test.fabric.manifest")
        .expect("fabric")
        .resource(
            DirectCounter::select("primary", DirectCounterConfig { value: 3 }).expect("selection"),
        )
        .system(TestOperations::select(TestOperationsConfig::new(5, 1)).expect("selection"))
        .component(Greeter::define(GreeterConfig {}))
        .component(PackageComponent::define(PackageComponentConfig {
            prefix: "hello".to_owned(),
        }))
        .build()
        .expect("manifest build");

    let manifest = built.manifest();
    assert_eq!(manifest.resources().len(), 1);
    assert_eq!(
        manifest.resources()[0].resource_id().as_str(),
        "fabric.test.counter"
    );
    assert_eq!(manifest.resources()[0].name().as_str(), "primary");
    assert_eq!(
        manifest.resources()[0].schema().resource(),
        manifest.resources()[0].resource_id()
    );
    assert_eq!(manifest.systems().len(), 1);
    assert_eq!(
        manifest.systems()[0].system_id().as_str(),
        "fabric.test.operations"
    );
    assert_eq!(manifest.components().len(), 2);
    assert_eq!(
        manifest.components()[0].component_id().as_str(),
        "fabric.test.greeter"
    );
    assert_eq!(manifest.components()[0].operations().len(), 1);
    assert_eq!(
        manifest.components()[1].component_id().as_str(),
        "fabric.test.package-component"
    );
    assert_eq!(manifest.components()[1].operations().len(), 1);
    assert!(
        !manifest.diagnostics().module_declarations().is_empty(),
        "typed contributions must retain module declarations"
    );
    assert!(
        manifest
            .diagnostics()
            .module_declarations()
            .iter()
            .any(|declaration| !declaration.provided_contracts().is_empty()),
        "provided contracts must stay visible through declarations"
    );
    assert!(manifest.diagnostics().provider_selections().is_empty());
    assert!(manifest.diagnostics().raw_blocks().is_empty());
}

#[test]
fn fabric_realization_manifest_covers_target_provider_and_selection() {
    let adapted = AdaptedCounter::select("primary", AdaptedCounterConfig {})
        .expect("selection")
        .using(FixedCounterAdapter::new(FixedCounterAdapterConfig {
            value: 4,
        }))
        .expect("adapter");
    let target_module = adapted.resource().module_id().clone();
    let provider_module = adapted.adapter().provider_module_id().clone();

    let built = Fabric::new("fabric.test.fabric.realization-manifest")
        .expect("fabric")
        .resource(adapted)
        .build()
        .expect("realization build");

    let manifest = built.manifest();
    assert_eq!(manifest.resources().len(), 1);
    assert_eq!(target_module, target_module);
    let provider_declaration = manifest
        .diagnostics()
        .module_declarations()
        .iter()
        .find(|declaration| declaration.module_id() == &provider_module)
        .expect("provider declaration");
    assert!(provider_declaration.host_requirement().is_some());
    assert_eq!(provider_module, target_module);
    assert!(manifest.diagnostics().provider_selections().is_empty());
}

#[test]
fn fabric_raw_blocks_coexist_as_opaque_presence() {
    let capture = Arc::new(Mutex::new(None));
    let raw_block = BlockBuilder::new(BlockId::new("fabric.test.fabric.raw").expect("block id"))
        .register_module(DirectConsumer::new(
            "fabric.test.fabric.raw.consumer",
            Arc::clone(&capture),
        ))
        .build();

    // The raw consumer contribution is authored before the typed provider
    // contribution on purpose: contribution and Block order are structural
    // only, while binding follows the resolved dependency graph.
    let built = Fabric::new("fabric.test.fabric.raw-coexist")
        .expect("fabric")
        .with_block(raw_block)
        .resource(
            DirectCounter::select("primary", DirectCounterConfig { value: 11 }).expect("selection"),
        )
        .build()
        .expect("raw coexistence build");

    assert_eq!(
        built.manifest().diagnostics().raw_blocks(),
        &[BlockId::new("fabric.test.fabric.raw").expect("block id")]
    );
    assert_eq!(built.manifest().resources().len(), 1);

    let mut instance = built
        .materialize("fabric.test.fabric.raw-coexist.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(capture.lock().expect("capture").clone(), Some(11));
}

#[test]
fn fabric_declaration_only_resource_builds_manifests_and_fails_materialization() {
    let built = Fabric::new("fabric.test.fabric.package-resource")
        .expect("fabric")
        .resource(
            PackageOnlyResource::select(
                "primary",
                PackageOnlyConfig {
                    endpoint: "https://packages.example/capability".to_owned(),
                },
            )
            .expect("selection"),
        )
        .build()
        .expect("declaration-only resource builds");

    assert_eq!(built.manifest().resources().len(), 1);
    assert_eq!(
        built.manifest().resources()[0].resource_id().as_str(),
        "fabric.test.package-only"
    );
    let module_id = built.manifest().diagnostics().module_declarations()[0]
        .module_id()
        .clone();

    let error = built
        .materialize("fabric.test.fabric.package-resource.instance")
        .expect_err("declaration-only resource must not materialize");
    assert!(matches!(
        error,
        CompositionError::MissingRuntimeMaterializer { module_id: missing }
            if missing == module_id
    ));
}

#[test]
fn fabric_declaration_only_system_builds_manifests_and_fails_materialization() {
    let built = Fabric::new("fabric.test.fabric.package-system")
        .expect("fabric")
        .system(
            PackageNotesSystem::select(PackageNotesConfig {
                note: "package facility".to_owned(),
            })
            .expect("selection"),
        )
        .build()
        .expect("declaration-only system builds");

    assert_eq!(built.manifest().systems().len(), 1);
    assert_eq!(
        built.manifest().systems()[0].system_id().as_str(),
        "fabric.test.package-notes"
    );
    let module_id = built.manifest().diagnostics().module_declarations()[0]
        .module_id()
        .clone();

    let error = built
        .materialize("fabric.test.fabric.package-system.instance")
        .expect_err("declaration-only system must not materialize");
    assert!(matches!(
        error,
        CompositionError::MissingRuntimeMaterializer { module_id: missing }
            if missing == module_id
    ));
}

#[test]
fn fabric_declaration_only_component_is_host_known_without_runtime() {
    let capture = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.fabric.package-component")
        .expect("fabric")
        .component(PackageComponent::define(PackageComponentConfig {
            prefix: "hello".to_owned(),
        }))
        .block("capture", |block| {
            block.module(RailsCaptureModule::new(Arc::clone(&capture)))
        })
        .expect("capture block")
        .build()
        .expect("declaration-only component builds");

    assert_eq!(built.manifest().components().len(), 1);
    assert_eq!(
        built.manifest().diagnostics().composition_exports().len(),
        1
    );
    assert_eq!(
        built.manifest().components()[0].component_id().as_str(),
        "fabric.test.package-component"
    );
    assert_eq!(built.manifest().components()[0].operations().len(), 1);
    assert_eq!(
        built.manifest().diagnostics().module_declarations().len(),
        1,
        "the native host module carries the declaration without fake runtime"
    );

    let mut instance = built
        .materialize("fabric.test.fabric.package-component.instance")
        .expect("instance with host-known declaration materializes");
    instance.start().expect("start");

    let rails = capture
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured rails");
    let component_id = ComponentId::new("fabric.test.package-component").expect("component id");
    assert_eq!(
        rails.materializer.known_component_ids(),
        vec![component_id.clone()]
    );
    assert_eq!(
        rails
            .materializer
            .materialize(&component_id)
            .expect_err("known component without attachment must fail explicitly"),
        ComponentError::MissingComponentParticipationRealization(component_id)
    );

    instance.stop().expect("stop instance");
}

#[test]
fn fabric_self_contained_resource_flows_through_normal_authoring() {
    let capture = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.fabric.direct")
        .expect("fabric")
        .resource(
            DirectCounter::select("primary", DirectCounterConfig { value: 29 }).expect("selection"),
        )
        .block("consumer", |block| {
            block.module(DirectConsumer::new(
                "fabric.test.fabric.direct.consumer",
                Arc::clone(&capture),
            ))
        })
        .expect("consumer block")
        .build()
        .expect("direct resource build");

    let mut instance = built
        .materialize("fabric.test.fabric.direct.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(capture.lock().expect("capture").clone(), Some(29));
}

#[test]
fn fabric_self_contained_system_flows_through_normal_authoring() {
    let capture = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.fabric.system-direct")
        .expect("fabric")
        .system(TestOperations::select(TestOperationsConfig::new(7, 3)).expect("selection"))
        .block("consumer", |block| {
            block.module(OperationsConsumer::new(
                "fabric.test.fabric.system.consumer",
                Arc::clone(&capture),
            ))
        })
        .expect("consumer block")
        .build()
        .expect("direct system build");

    let mut instance = built
        .materialize("fabric.test.fabric.system-direct.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(capture.lock().expect("capture").clone(), Some(7));
}

#[test]
fn fabric_handwritten_adapter_flows_through_normal_authoring() {
    let capture = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.fabric.clock")
        .expect("fabric")
        .resource(
            Clock::select("primary", ClockConfig::default())
                .expect("selection")
                .using(MemoryClock::new(13))
                .expect("adapter"),
        )
        .block("consumer", |block| {
            block.module(ClockConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer block")
        .build()
        .expect("handwritten adapter build");

    let mut instance = built
        .materialize_on("fabric.test.fabric.clock.instance", &test_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(capture.lock().expect("capture").clone(), Some(13));
}

#[test]
fn fabric_host_bound_adapter_flows_through_normal_authoring() {
    let capture = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.fabric.host-bound")
        .expect("fabric")
        .resource(
            Clock::select("primary", ClockConfig::default())
                .expect("selection")
                .using(HostBoundClockAdapter::new(41))
                .expect("adapter"),
        )
        .block("consumer", |block| {
            block.module(ClockConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer block")
        .build()
        .expect("host-bound build");

    let mut instance = built
        .materialize_on("fabric.test.fabric.host-bound.instance", &facility_host())
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(capture.lock().expect("capture").clone(), Some(41));
}

#[test]
fn fabric_runtime_component_flows_through_normal_authoring() {
    let built = Fabric::new("fabric.test.fabric.greeter")
        .expect("fabric")
        .component(Greeter::define(GreeterConfig {}))
        .build()
        .expect("greeter build");

    assert_eq!(built.manifest().components().len(), 1);

    let mut instance = built
        .materialize("fabric.test.fabric.greeter.instance")
        .expect("instance");
    instance.start().expect("start");

    let components = instance.components().expect("component operator export");
    components
        .materialize::<Greeter>()
        .expect("materialize greeter");
    let output: GreeterOutput = futures::executor::block_on(components.invoke_external(
        &greeter::api::greet(),
        GreeterInput {
            name: "fabric".to_owned(),
        },
    ))
    .expect("invoke");
    assert_eq!(output.message, "hello, fabric");

    instance.stop().expect("stop instance");
}

#[test]
fn gateway_component_is_realized_by_pingora_adapter_through_core_contracts() {
    let gateway = ComponentSpec::<Gateway>::declaration_only(GatewayConfig {
        prefix: "gateway".to_owned(),
    })
    .using(PingoraAdapter {
        implementation: "pingora".to_owned(),
    })
    .expect("typed Gateway adapter realization");
    let built = Fabric::new("fabric.test.gateway")
        .expect("fabric")
        .component(gateway)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_on("fabric.test.gateway.local", &facility_host())
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("components");
    components
        .materialize::<Gateway>()
        .expect("adapter realized gateway");
    let output: GatewayOutput = futures::executor::block_on(
        components.invoke_external(&gateway::api::handle(), GatewayInput),
    )
    .expect("invoke");
    assert_eq!(output.value, "gateway:pingora");
    components
        .dematerialize::<Gateway>()
        .expect("dematerialize");
    instance.stop().expect("stop instance");
}

#[test]
fn adapter_realized_component_prepares_external_augmentation_without_adapter_knowledge() {
    let audit = ComponentSpec::<Gateway>::declaration_only(GatewayConfig {
        prefix: "gateway".to_owned(),
    })
    .augment::<GatewayAudit>(())
    .expect("attach external audit")
    .using(GatewayAuditSupport);
    let audit_requirement = audit.requirement();
    let trace = audit
        .into_set()
        .augment::<GatewayTrace>(())
        .expect("attach external trace")
        .using(GatewayTraceSupport);
    let trace_requirement = trace.requirement();
    assert_eq!(
        audit_requirement.target_component_id(),
        &Gateway::component_id()
    );
    assert_eq!(
        trace_requirement.target_component_id(),
        &Gateway::component_id()
    );
    let gateway = trace
        .into_set()
        .using_adapter(PingoraAdapter {
            implementation: "pingora".to_owned(),
        })
        .expect("adapter realization");
    let built = Fabric::new("fabric.test.gateway.augmentation")
        .expect("fabric")
        .component(gateway)
        .build()
        .expect("build");
    assert_eq!(built.manifest().component_augmentations().len(), 2);
    let mut instance = built
        .materialize_on("fabric.test.gateway.augmentation.local", &facility_host())
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("components");
    components
        .materialize::<Gateway>()
        .expect("adapter realized gateway participation");
    let output: GatewayOutput = futures::executor::block_on(
        components.invoke_external(&gateway::api::handle(), GatewayInput),
    )
    .expect("invoke");
    assert_eq!(output.value, "gateway:pingora");
    instance.stop().expect("stop instance");
}

#[test]
fn gateway_adapter_realization_keeps_component_config_per_composition() {
    let build = |composition_id: &str, prefix: &str| {
        Fabric::new(composition_id)
            .expect("fabric")
            .component(
                ComponentSpec::<Gateway>::declaration_only(GatewayConfig {
                    prefix: prefix.to_owned(),
                })
                .using(PingoraAdapter {
                    implementation: "pingora".to_owned(),
                })
                .expect("typed gateway realization"),
            )
            .build()
            .expect("build")
    };

    for (composition_id, prefix) in [
        ("fabric.test.gateway.a", "a"),
        ("fabric.test.gateway.b", "b"),
    ] {
        let built = build(composition_id, prefix);
        let mut instance = built
            .materialize_on(format!("{composition_id}.local"), &facility_host())
            .expect("materialize");
        instance.start().expect("start");
        let components = instance.components().expect("components");
        components
            .materialize::<Gateway>()
            .expect("materialize gateway");
        let output: GatewayOutput = futures::executor::block_on(
            components.invoke_external(&gateway::api::handle(), GatewayInput),
        )
        .expect("invoke");
        assert_eq!(output.value, format!("{prefix}:pingora"));
        components
            .dematerialize::<Gateway>()
            .expect("dematerialize");
        instance.stop().expect("stop instance");
    }
}

#[test]
fn gateway_adapter_realization_rejects_an_incompatible_host() {
    let gateway = ComponentSpec::<Gateway>::declaration_only(GatewayConfig {
        prefix: "gateway".to_owned(),
    })
    .using(PingoraAdapter {
        implementation: "pingora".to_owned(),
    })
    .expect("typed gateway realization");
    let built = Fabric::new("fabric.test.gateway.host")
        .expect("fabric")
        .component(gateway)
        .build()
        .expect("build");
    assert!(
        built
            .materialize_on("fabric.test.gateway.host.local", &test_host())
            .is_err()
    );
}

#[test]
fn gateway_semantics_support_an_alternate_adapter_realization() {
    let gateway = ComponentSpec::<Gateway>::declaration_only(GatewayConfig {
        prefix: "edge".to_owned(),
    })
    .using(AlternateGatewayAdapter {
        implementation: "alternate".to_owned(),
    })
    .expect("typed alternate gateway realization");
    let built = Fabric::new("fabric.test.gateway.alternate")
        .expect("fabric")
        .component(gateway)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_on("fabric.test.gateway.alternate.local", &facility_host())
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("components");
    components
        .materialize::<Gateway>()
        .expect("materialize gateway");
    let output: GatewayOutput = futures::executor::block_on(
        components.invoke_external(&gateway::api::handle(), GatewayInput),
    )
    .expect("invoke");
    assert_eq!(output.value, "edge:alternate");
    components
        .dematerialize::<Gateway>()
        .expect("dematerialize");
    instance.stop().expect("stop instance");
}

#[test]
fn generational_component_realization_change_keeps_live_instances_independent() {
    // Equal CompositionIds are author-selected logical labels, not declaration
    // revisions. These are independently authored declarations that retain the
    // same Gateway semantics and config while selecting different realizations.
    let first_built = Fabric::new("fabric.test.generational.gateway")
        .expect("fabric")
        .component(
            ComponentSpec::<Gateway>::declaration_only(GatewayConfig {
                prefix: "edge".to_owned(),
            })
            .using(PingoraAdapter {
                implementation: "pingora".to_owned(),
            })
            .expect("Pingora realization"),
        )
        .build()
        .expect("first declaration");
    let second_built = Fabric::new("fabric.test.generational.gateway")
        .expect("fabric")
        .component(
            ComponentSpec::<Gateway>::declaration_only(GatewayConfig {
                prefix: "edge".to_owned(),
            })
            .using(AlternateGatewayAdapter {
                implementation: "alternate".to_owned(),
            })
            .expect("alternate realization"),
        )
        .build()
        .expect("second declaration");

    assert_eq!(
        first_built.core().id(),
        second_built.core().id(),
        "CompositionId does not establish declaration revision equality"
    );
    assert_eq!(
        first_built.manifest().components()[0].component_id(),
        second_built.manifest().components()[0].component_id()
    );

    let instance_id = "fabric.test.generational.gateway.local";
    let mut first = first_built
        .materialize_on(instance_id, &facility_host())
        .expect("first instance");
    first.start().expect("first starts");
    let first_generation = first.generation();
    let first_components = first.components().expect("first components");
    let first_participation = first_components
        .materialize::<Gateway>()
        .expect("first Gateway participation")
        .participation()
        .clone();

    let mut second = second_built
        .materialize_on(instance_id, &facility_host())
        .expect("second instance");
    second.start().expect("second starts");
    let second_generation = second.generation();
    let second_components = second.components().expect("second components");
    let second_participation = second_components
        .materialize::<Gateway>()
        .expect("second Gateway participation")
        .participation()
        .clone();

    assert_eq!(first.instance_id(), second.instance_id());
    assert_ne!(first_generation, second_generation);
    assert_eq!(first.lifecycle(), LifecycleState::Running);
    assert_eq!(second.lifecycle(), LifecycleState::Running);
    assert_eq!(first_participation.generation(), first_generation);
    assert_eq!(second_participation.generation(), second_generation);
    assert_ne!(first_participation, second_participation);
    assert_eq!(
        first_participation.component().component_id(),
        &Gateway::component_id()
    );
    assert_eq!(
        second_participation.component().component_id(),
        &Gateway::component_id()
    );

    let first_output: GatewayOutput = futures::executor::block_on(
        first_components.invoke_external(&gateway::api::handle(), GatewayInput),
    )
    .expect("first invocation");
    let second_output: GatewayOutput = futures::executor::block_on(
        second_components.invoke_external(&gateway::api::handle(), GatewayInput),
    )
    .expect("second invocation");
    assert_eq!(first_output.value, "edge:pingora");
    assert_eq!(second_output.value, "edge:alternate");

    // Fabric has no primary-generation or cutover concept: the caller stops
    // the old generation explicitly, and that leaves the new one untouched.
    first.stop().expect("stop runtime");
    assert_eq!(first.lifecycle(), LifecycleState::Stopped);
    assert_eq!(second.lifecycle(), LifecycleState::Running);
    let second_output: GatewayOutput = futures::executor::block_on(
        second_components.invoke_external(&gateway::api::handle(), GatewayInput),
    )
    .expect("second remains usable after first stops");
    assert_eq!(second_output.value, "edge:alternate");
    second.stop().expect("stop runtime");
}

#[test]
fn failed_new_generation_is_isolated_from_a_running_generation() {
    let running_built = Fabric::new("fabric.test.generational.failure.running")
        .expect("fabric")
        .component(
            ComponentSpec::<Gateway>::declaration_only(GatewayConfig {
                prefix: "stable".to_owned(),
            })
            .using(PingoraAdapter {
                implementation: "pingora".to_owned(),
            })
            .expect("realization"),
        )
        .build()
        .expect("running declaration");
    let failing_built = Fabric::new("fabric.test.generational.failure.new")
        .expect("fabric")
        .component(
            ComponentSpec::<Gateway>::declaration_only(GatewayConfig {
                prefix: "new".to_owned(),
            })
            .using(AlternateGatewayAdapter {
                implementation: "alternate".to_owned(),
            })
            .expect("realization"),
        )
        .block("failing-start", |block| {
            block.module(FailingGenerationStart::new())
        })
        .expect("failure block")
        .build()
        .expect("failing declaration");

    let instance_id = "fabric.test.generational.failure.local";
    let mut running = running_built
        .materialize_on(instance_id, &facility_host())
        .expect("running instance");
    running.start().expect("running start");
    let running_generation = running.generation();
    let running_components = running.components().expect("components");
    running_components
        .materialize::<Gateway>()
        .expect("running Gateway participation");

    let mut failed = failing_built
        .materialize_on(instance_id, &facility_host())
        .expect("failed generation is still returned Ready");
    assert_ne!(running_generation, failed.generation());
    assert!(failed.start().is_err());
    assert_eq!(failed.lifecycle(), LifecycleState::Stopped);
    assert_eq!(running.lifecycle(), LifecycleState::Running);
    assert_eq!(running.generation(), running_generation);
    let output: GatewayOutput = futures::executor::block_on(
        running_components.invoke_external(&gateway::api::handle(), GatewayInput),
    )
    .expect("running generation remains usable");
    assert_eq!(output.value, "stable:pingora");

    // A pre-Instance failure is isolated in the same way: the incompatible
    // Host prevents a usable new Instance from being returned at all.
    assert!(
        failing_built
            .materialize_on(instance_id, &test_host())
            .is_err()
    );
    assert_eq!(running.lifecycle(), LifecycleState::Running);
    running.stop().expect("stop instance");
}

#[test]
fn generational_resource_realization_change_is_fresh_and_has_no_state_transfer() {
    let first_capture = Arc::new(Mutex::new(None));
    let second_capture = Arc::new(Mutex::new(None));
    let first_built = Fabric::new("fabric.test.generational.clock")
        .expect("fabric")
        .resource(
            Clock::select("primary", ClockConfig::default())
                .expect("selection")
                .using(MemoryClock::new(11))
                .expect("memory realization"),
        )
        .block("consumer", |block| {
            block.module(ClockConsumer::new(Arc::clone(&first_capture)))
        })
        .expect("consumer block")
        .build()
        .expect("first declaration");
    let second_built = Fabric::new("fabric.test.generational.clock")
        .expect("fabric")
        .resource(
            Clock::select("primary", ClockConfig::default())
                .expect("selection")
                .using(HostBoundClockAdapter::new(29))
                .expect("host-bound realization"),
        )
        .block("consumer", |block| {
            block.module(ClockConsumer::new(Arc::clone(&second_capture)))
        })
        .expect("consumer block")
        .build()
        .expect("second declaration");

    assert_eq!(first_built.core().id(), second_built.core().id());
    assert_eq!(
        first_built.manifest().resources()[0].resource_id(),
        second_built.manifest().resources()[0].resource_id()
    );
    assert_eq!(
        first_built.manifest().resources()[0].name(),
        second_built.manifest().resources()[0].name()
    );

    let instance_id = "fabric.test.generational.clock.local";
    let mut first = first_built
        .materialize_on(instance_id, &facility_host())
        .expect("first instance");
    let mut second = second_built
        .materialize_on(instance_id, &facility_host())
        .expect("second instance");
    assert_eq!(first.instance_id(), second.instance_id());
    assert_ne!(first.generation(), second.generation());
    first.start().expect("first starts");
    second.start().expect("second starts");
    assert_eq!(*first_capture.lock().expect("capture"), Some(11));
    assert_eq!(*second_capture.lock().expect("capture"), Some(29));
    first.stop().expect("stop runtime");
    second.stop().expect("stop runtime");
}

#[test]
fn fabric_component_operator_is_instance_local_and_preserves_lifecycle_errors() {
    let built = Fabric::new("fabric.test.fabric.component-operator-instances")
        .expect("fabric")
        .component(Greeter::define(GreeterConfig {}))
        .build()
        .expect("build");

    let mut first = built
        .materialize("fabric.test.operator.first")
        .expect("first");
    let mut second = built
        .materialize("fabric.test.operator.second")
        .expect("second");
    assert_ne!(first.generation(), second.generation());

    assert!(matches!(
        first
            .components()
            .expect("first component operator")
            .materialize::<Greeter>(),
        Err(ComponentError::Unavailable)
    ));
    first.start().expect("first start");
    second.start().expect("second start");
    first
        .components()
        .expect("first component operator")
        .materialize::<Greeter>()
        .expect("first greeter");
    assert!(matches!(
        futures::executor::block_on(
            second
                .components()
                .expect("second component operator")
                .invoke_external(
                    &greeter::api::greet(),
                    GreeterInput {
                        name: "Ada".to_owned()
                    },
                ),
        ),
        Err(ComponentError::UnknownOperation(_))
    ));
    second
        .components()
        .expect("second component operator")
        .materialize::<Greeter>()
        .expect("second greeter");
    let output = futures::executor::block_on(
        second
            .components()
            .expect("second component operator")
            .invoke_external(
                &greeter::api::greet(),
                GreeterInput {
                    name: "Ada".to_owned(),
                },
            ),
    )
    .expect("second invocation");
    assert_eq!(output.message, "hello, Ada");
    first.stop().expect("stop runtime");
    assert!(matches!(
        futures::executor::block_on(
            first
                .components()
                .expect("first component operator")
                .invoke_external(
                    &greeter::api::greet(),
                    GreeterInput {
                        name: "Ada".to_owned()
                    },
                ),
        ),
        Err(ComponentError::Unavailable)
    ));
    second.stop().expect("stop runtime");
}

#[test]
fn component_macro_resolves_typed_resource_and_system_dependencies() {
    let counter = DirectCounter::select("primary", DirectCounterConfig { value: 5 })
        .expect("counter selection");
    let operations =
        TestOperations::select(TestOperationsConfig::new(7, 1)).expect("operations selection");
    let built = Fabric::new("fabric.test.component.macro-dependencies")
        .expect("fabric")
        .component(
            MacroDependencyProbe::define(MacroDependencyProbeConfig { multiplier: 2 })
                .select_resource_provider(&counter)
                .select_system_provider(&operations),
        )
        .system(operations)
        .resource(counter)
        .build()
        .expect("build");

    let declaration = built
        .manifest()
        .components()
        .iter()
        .find(|value| value.component_id() == &MacroDependencyProbe::component_id())
        .expect("macro component declaration");
    assert_eq!(declaration.resource_requirements().len(), 1);
    assert_eq!(declaration.system_requirements().len(), 1);
    assert_eq!(built.manifest().component_resource_bindings().len(), 1);
    assert_eq!(built.manifest().component_system_bindings().len(), 1);

    let mut instance = built
        .materialize("fabric.test.component.macro-dependencies.instance")
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("component operator export");
    components
        .materialize::<MacroDependencyProbe>()
        .expect("materialize component");
    let output: MacroDependencyOutput = futures::executor::block_on(components.invoke_external(
        &macro_dependency_probe::api::observe(),
        MacroDependencyInput,
    ))
    .expect("invoke macro component operation");
    assert_eq!(output.value, 14);
    instance.stop().expect("stop instance");
}

#[test]
fn component_macro_keeps_domain_results_inside_typed_output() {
    let built = Fabric::new("fabric.test.component.document-probe")
        .expect("fabric")
        .component(DocumentProbe::define())
        .build()
        .expect("build");
    let mut instance = built
        .materialize("fabric.test.component.document-probe.instance")
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("component host");

    assert!(matches!(
        futures::executor::block_on(components.invoke_external(
            &document_probe::api::open(),
            OpenDocumentInput { exists: false },
        ),),
        Err(ComponentError::UnknownOperation(_))
    ));

    components
        .materialize::<DocumentProbe>()
        .expect("component materialization");
    let missing = futures::executor::block_on(components.invoke_external(
        &document_probe::api::open(),
        OpenDocumentInput { exists: false },
    ))
    .expect("runtime invocation succeeds");
    assert_eq!(missing, Err(DocumentError::NotFound));

    let found = futures::executor::block_on(components.invoke_external(
        &document_probe::api::open(),
        OpenDocumentInput { exists: true },
    ))
    .expect("runtime invocation succeeds");
    assert_eq!(
        found,
        Ok(DocumentOutput {
            title: "Fabric".to_owned(),
        })
    );
    instance.stop().expect("stop instance");
}

fn run_dual_counter_probe(swap: bool) -> DualCounterOutput {
    let primary = DirectCounter::select("primary", DirectCounterConfig { value: 11 })
        .expect("primary selection");
    let secondary = DirectCounter::select("secondary", DirectCounterConfig { value: 22 })
        .expect("secondary selection");
    let (primary_provider, secondary_provider) = if swap {
        (&secondary, &primary)
    } else {
        (&primary, &secondary)
    };
    let built = Fabric::new("fabric.test.component.dual-counter-probe")
        .expect("fabric")
        .component(
            DualCounterProbe::define()
                .select_named_resource_provider(
                    &ComponentResourceRequirement::new(
                        ComponentRelationName::new("primary_store").expect("relation name"),
                        Requires::<DirectCounter>::provisional(),
                    ),
                    primary_provider,
                )
                .select_named_resource_provider(
                    &ComponentResourceRequirement::new(
                        ComponentRelationName::new("secondary_store").expect("relation name"),
                        Requires::<DirectCounter>::provisional(),
                    ),
                    secondary_provider,
                ),
        )
        .resource(primary)
        .resource(secondary)
        .build()
        .expect("build");
    let declaration = built.manifest().components().first().expect("declaration");
    assert_eq!(declaration.resource_requirements().len(), 2);
    assert_eq!(
        declaration.resource_requirements()[0].name().as_str(),
        "primary_store"
    );
    assert_eq!(
        declaration.resource_requirements()[1].name().as_str(),
        "secondary_store"
    );
    assert_eq!(built.manifest().component_resource_bindings().len(), 2);
    let mut instance = built
        .materialize("fabric.test.dual-counter.instance")
        .expect("instance");
    instance.start().expect("start");
    let components = instance.components().expect("component operator export");
    components
        .materialize::<DualCounterProbe>()
        .expect("component");
    let output = futures::executor::block_on(
        components.invoke_external(&dual_counter_probe::api::observe(), DualCounterInput),
    )
    .expect("invoke");
    instance.stop().expect("stop instance");
    output
}

#[test]
fn component_resource_requirement_occurrences_bind_independently_and_can_swap_providers() {
    assert_eq!(
        run_dual_counter_probe(false),
        DualCounterOutput {
            primary: 11,
            secondary: 22
        }
    );
    assert_eq!(
        run_dual_counter_probe(true),
        DualCounterOutput {
            primary: 22,
            secondary: 11
        }
    );
}

#[test]
fn handwritten_component_authoring_can_name_same_target_resource_requirements() {
    let primary =
        DirectCounter::select("primary", DirectCounterConfig { value: 1 }).expect("primary");
    let cache = DirectCounter::select("cache", DirectCounterConfig { value: 2 }).expect("cache");
    let storage_requirement = ComponentResourceRequirement::new(
        ComponentRelationName::new("storage").expect("name"),
        Requires::<DirectCounter>::provisional(),
    );
    let cache_requirement = ComponentResourceRequirement::new(
        ComponentRelationName::new("cache").expect("name"),
        Requires::<DirectCounter>::provisional(),
    );
    let built = Fabric::new("fabric.test.handwritten.named-resource-requirements")
        .expect("fabric")
        .component(
            PackageComponent::define(PackageComponentConfig {
                prefix: "named".to_owned(),
            })
            .requires_named_resource(storage_requirement.clone())
            .requires_named_resource(cache_requirement.clone())
            .select_named_resource_provider(&storage_requirement, &primary)
            .select_named_resource_provider(&cache_requirement, &cache),
        )
        .resource(primary)
        .resource(cache)
        .build()
        .expect("build");
    assert_eq!(
        built.manifest().components()[0]
            .resource_requirements()
            .len(),
        2
    );
    assert_eq!(
        built.manifest().component_resource_bindings()[0]
            .requirement_name()
            .as_str(),
        "storage"
    );
    assert_eq!(
        built.manifest().component_resource_bindings()[1]
            .requirement_name()
            .as_str(),
        "cache"
    );
}

#[test]
fn fabric_explicit_selection_resolves_ambiguity_and_core_owns_ambiguity() {
    let direct_a =
        DirectCounter::select("primary", DirectCounterConfig { value: 9 }).expect("selection");
    let direct_b =
        DirectCounter::select("secondary", DirectCounterConfig { value: 50 }).expect("selection");
    let derived = DerivedCounter::select("derived", DerivedCounterConfig {}).expect("selection");
    let derived_module = derived.module_id().clone();
    let chosen_provider = direct_b.module_id().clone();

    let error = Fabric::new("fabric.test.fabric.ambiguous")
        .expect("fabric")
        .resource(direct_a)
        .resource(direct_b)
        .resource(derived)
        .build()
        .expect_err("ambiguous dependency must fail in Core");
    assert!(
        matches!(
            error,
            FabricBuildError::Composition(CompositionError::AmbiguousProvider {
                module_id,
                ..
            }) if module_id == derived_module
        ),
        "ambiguity must stay Core-owned"
    );

    let capture = Arc::new(Mutex::new(None));
    let direct_a =
        DirectCounter::select("primary", DirectCounterConfig { value: 9 }).expect("selection");
    let direct_b =
        DirectCounter::select("secondary", DirectCounterConfig { value: 50 }).expect("selection");
    let derived = DerivedCounter::select("derived", DerivedCounterConfig {}).expect("selection");
    let built = Fabric::new("fabric.test.fabric.selected")
        .expect("fabric")
        .resource(direct_a)
        .resource(direct_b)
        .resource(derived)
        .select_provider(ContractProviderSelection::new(
            derived_module.clone(),
            DirectCounter::primary_contract_key().id().clone(),
            chosen_provider.clone(),
        ))
        .block("consumer", |block| {
            block.module(DerivedValueConsumer::new(Arc::clone(&capture)))
        })
        .expect("consumer block")
        .build()
        .expect("explicit selection builds");

    assert_eq!(
        built.manifest().diagnostics().provider_selections().len(),
        1
    );

    let mut instance = built
        .materialize("fabric.test.fabric.selected.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");

    assert_eq!(capture.lock().expect("capture").clone(), Some(100));
}

#[derive(Clone)]
struct DerivedValueConsumer {
    module_id: ModuleId,
    requirement: Requires<DerivedCounter>,
    capture: Arc<Mutex<Option<u64>>>,
}

impl DerivedValueConsumer {
    fn new(capture: Arc<Mutex<Option<u64>>>) -> Self {
        Self {
            module_id: module("fabric.test.fabric.derived.consumer").expect("module id"),
            requirement: Requires::<DerivedCounter>::provisional(),
            capture,
        }
    }
}

impl ModuleRuntime for DerivedValueConsumer {
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
        *self.capture.lock().expect("capture lock") =
            Some(resolved.value().current_value().value());
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
fn fabric_duplicate_system_identity_fails_through_core() {
    let error = Fabric::new("fabric.test.fabric.duplicate-system")
        .expect("fabric")
        .system(TestOperations::select(TestOperationsConfig::new(1, 1)).expect("selection"))
        .system(TestOperations::select(TestOperationsConfig::new(99, 5)).expect("selection"))
        .build()
        .expect_err("duplicate system occurrence must fail");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::DuplicateModuleId { .. })
    ));
}

#[test]
fn sdk_composition_preserves_manifest_and_explicit_core_escape_hatches() {
    let composition: fabric::Composition = Fabric::new("fabric.test.fabric.parts")
        .expect("fabric")
        .resource(
            DirectCounter::select("primary", DirectCounterConfig { value: 1 }).expect("selection"),
        )
        .build()
        .expect("composition build");

    assert_eq!(composition.id().as_str(), "fabric.test.fabric.parts");
    assert_eq!(composition.id(), composition.core().id());
    let debug = format!("{composition:?}");
    assert!(debug.contains("Composition") && debug.contains("fabric.test.fabric.parts"));
    assert!(!debug.contains("manifest") && !debug.contains("component_host_export"));
    assert_eq!(composition.manifest().resources().len(), 1);
    let core: &fabric::core::Composition = composition.core();
    takes_raw_composition(core);

    let composition = Fabric::new("fabric.test.fabric.core-consume")
        .expect("fabric")
        .resource(
            DirectCounter::select("primary", DirectCounterConfig { value: 2 }).expect("selection"),
        )
        .build()
        .expect("composition build");
    let core: fabric::core::Composition = composition.into_core();
    takes_raw_composition(&core);
}

fn takes_raw_composition(_composition: &fabric::core::Composition) {}

struct ResourceOwnedComponent;
struct ResourceOwnedComponentB;
struct SystemOwnedComponent;
struct SystemOwnedComponentB;
struct CombinedOwnedComponent;
struct AlphaComponent;
struct MismatchedSelfRealizingComponent;

#[derive(Clone)]
struct MismatchedSelfRealizingComponentConfig;

impl ComponentDefinition for MismatchedSelfRealizingComponent {
    type Config = MismatchedSelfRealizingComponentConfig;

    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.component.mismatched-self-realization").expect("component id")
    }

    fn declaration() -> fabric_component::ComponentDeclaration {
        fabric_component::ComponentDeclaration::new(Self::component_id(), Vec::new())
    }
}

impl SelfRealizingComponentDefinition for MismatchedSelfRealizingComponent {
    fn self_realization(
        _config: &Self::Config,
    ) -> fabric_component::ComponentParticipationRealization {
        fabric_component::ComponentParticipationRealization::new(
            ComponentId::new("fabric.test.component.other-self-realization").expect("component id"),
            |_scope| Ok(Health::Healthy),
        )
    }
}

#[test]
fn self_realization_must_match_its_component_declaration() {
    assert!(
        ComponentSpec::<MismatchedSelfRealizingComponent>::self_realizing(
            MismatchedSelfRealizingComponentConfig,
        )
        .is_err(),
        "a self realization for another ComponentId must be rejected"
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AlphaObservation {
    tick: u64,
    marker: u64,
}
#[derive(Clone)]
struct AlphaConfig {
    capture: Arc<Mutex<Vec<AlphaObservation>>>,
}
impl ComponentDefinition for AlphaComponent {
    type Config = AlphaConfig;
    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.alpha.component").expect("id")
    }
    fn declaration() -> fabric_component::ComponentDeclaration {
        fabric_component::ComponentDeclaration::new(Self::component_id(), Vec::new())
    }
}
impl SelfRealizingComponentDefinition for AlphaComponent {
    fn self_realization(
        config: &Self::Config,
    ) -> fabric_component::ComponentParticipationRealization {
        let capture = Arc::clone(&config.capture);
        fabric_component::ComponentParticipationRealization::new(
            Self::component_id(),
            move |scope| {
                let clock = scope.resource(&Requires::<Clock>::versioned(
                    ContractVersionRequirement::parse("^1").expect("clock requirement"),
                ))?;
                let system = scope.system(&SystemRequires::<AdaptedOperations>::versioned(
                    ContractVersionRequirement::parse("^2").expect("system requirement"),
                ))?;
                capture.lock().expect("capture").push(AlphaObservation {
                    tick: clock
                        .current_tick()
                        .map_err(|_| fabric_component::ComponentError::Unavailable)?
                        .value(),
                    marker: system.current_marker().value(),
                });
                Ok(Health::Healthy)
            },
        )
    }
}

#[derive(Clone)]
struct ResourceOwnedComponentConfig {
    capture: Arc<Mutex<Vec<u64>>>,
}

impl ComponentDefinition for ResourceOwnedComponent {
    type Config = ResourceOwnedComponentConfig;

    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.component.resource-owned").expect("component id")
    }

    fn declaration() -> fabric_component::ComponentDeclaration {
        fabric_component::ComponentDeclaration::new(Self::component_id(), Vec::new())
    }
}
impl SelfRealizingComponentDefinition for ResourceOwnedComponent {
    fn self_realization(
        config: &Self::Config,
    ) -> fabric_component::ComponentParticipationRealization {
        let capture = Arc::clone(&config.capture);
        fabric_component::ComponentParticipationRealization::new(
            Self::component_id(),
            move |scope| {
                let counter = scope.resource(&Requires::<DirectCounter>::provisional())?;
                capture
                    .lock()
                    .expect("capture lock")
                    .push(counter.current_value().value());
                Ok(Health::Healthy)
            },
        )
    }
}

impl ComponentDefinition for ResourceOwnedComponentB {
    type Config = ResourceOwnedComponentConfig;
    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.component.resource-owned-b").expect("component id")
    }
    fn declaration() -> fabric_component::ComponentDeclaration {
        fabric_component::ComponentDeclaration::new(Self::component_id(), Vec::new())
    }
}
impl SelfRealizingComponentDefinition for ResourceOwnedComponentB {
    fn self_realization(
        config: &Self::Config,
    ) -> fabric_component::ComponentParticipationRealization {
        let capture = Arc::clone(&config.capture);
        fabric_component::ComponentParticipationRealization::new(
            Self::component_id(),
            move |scope| {
                let counter = scope.resource(&Requires::<DirectCounter>::provisional())?;
                capture
                    .lock()
                    .expect("capture lock")
                    .push(counter.current_value().value());
                Ok(Health::Healthy)
            },
        )
    }
}

impl ComponentDefinition for SystemOwnedComponent {
    type Config = ResourceOwnedComponentConfig;
    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.component.system-owned").expect("component id")
    }
    fn declaration() -> fabric_component::ComponentDeclaration {
        fabric_component::ComponentDeclaration::new(Self::component_id(), Vec::new())
    }
}
impl SelfRealizingComponentDefinition for SystemOwnedComponent {
    fn self_realization(
        config: &Self::Config,
    ) -> fabric_component::ComponentParticipationRealization {
        let capture = Arc::clone(&config.capture);
        fabric_component::ComponentParticipationRealization::new(
            Self::component_id(),
            move |scope| {
                let system = scope.system(&SystemRequires::<TestOperations>::versioned(
                    ContractVersionRequirement::parse("^1.2").expect("requirement"),
                ))?;
                capture
                    .lock()
                    .expect("capture lock")
                    .push(system.current_marker().value());
                Ok(Health::Healthy)
            },
        )
    }
}

impl ComponentDefinition for SystemOwnedComponentB {
    type Config = ResourceOwnedComponentConfig;
    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.component.system-owned-b").expect("component id")
    }
    fn declaration() -> fabric_component::ComponentDeclaration {
        fabric_component::ComponentDeclaration::new(Self::component_id(), Vec::new())
    }
}
impl SelfRealizingComponentDefinition for SystemOwnedComponentB {
    fn self_realization(
        config: &Self::Config,
    ) -> fabric_component::ComponentParticipationRealization {
        let capture = Arc::clone(&config.capture);
        fabric_component::ComponentParticipationRealization::new(
            Self::component_id(),
            move |scope| {
                let system = scope.system(&SystemRequires::<TestOperations>::versioned(
                    ContractVersionRequirement::parse("^1.2").expect("requirement"),
                ))?;
                capture
                    .lock()
                    .expect("capture lock")
                    .push(system.current_marker().value());
                Ok(Health::Healthy)
            },
        )
    }
}

impl ComponentDefinition for CombinedOwnedComponent {
    type Config = ResourceOwnedComponentConfig;
    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.component.resource-system-owned").expect("component id")
    }
    fn declaration() -> fabric_component::ComponentDeclaration {
        fabric_component::ComponentDeclaration::new(Self::component_id(), Vec::new())
    }
}
impl SelfRealizingComponentDefinition for CombinedOwnedComponent {
    fn self_realization(
        config: &Self::Config,
    ) -> fabric_component::ComponentParticipationRealization {
        let capture = Arc::clone(&config.capture);
        fabric_component::ComponentParticipationRealization::new(
            Self::component_id(),
            move |scope| {
                let resource = scope.resource(&Requires::<DirectCounter>::provisional())?;
                let system = scope.system(&SystemRequires::<TestOperations>::versioned(
                    ContractVersionRequirement::parse("^1.2").expect("requirement"),
                ))?;
                capture
                    .lock()
                    .expect("capture lock")
                    .push(resource.current_value().value() + system.current_marker().value());
                Ok(Health::Healthy)
            },
        )
    }
}

macro_rules! self_realizing_define {
    ($component:ty, $config:ty) => {
        impl $component {
            fn define(config: $config) -> ComponentSpec<Self> {
                ComponentSpec::<Self>::self_realizing(config)
                    .expect("handwritten self realization matches ComponentId")
            }
        }
    };
}

self_realizing_define!(AlphaComponent, AlphaConfig);
self_realizing_define!(ResourceOwnedComponent, ResourceOwnedComponentConfig);
self_realizing_define!(ResourceOwnedComponentB, ResourceOwnedComponentConfig);
self_realizing_define!(SystemOwnedComponent, ResourceOwnedComponentConfig);
self_realizing_define!(SystemOwnedComponentB, ResourceOwnedComponentConfig);
self_realizing_define!(CombinedOwnedComponent, ResourceOwnedComponentConfig);

#[test]
fn component_owned_system_requirement_lowers_through_core_and_reaches_its_runtime() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let rails = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.component.system-owned")
        .expect("fabric")
        .system(TestOperations::select(TestOperationsConfig::new(17, 1)).expect("system"))
        .component(
            SystemOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&capture),
            })
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            )),
        )
        .block("capture", |block| {
            block.module(RailsCaptureModule::new(Arc::clone(&rails)))
        })
        .expect("capture")
        .build()
        .expect("Core resolves System carrier");
    assert_eq!(
        built.manifest().components()[0].system_requirements()[0].system_id(),
        &TestOperations::system_id()
    );
    let mut instance = built
        .materialize("fabric.test.component.system-owned.instance")
        .expect("instance");
    instance.start().expect("start");
    rails
        .lock()
        .expect("rails")
        .clone()
        .expect("captured")
        .materializer
        .materialize(&SystemOwnedComponent::component_id())
        .expect("component");
    assert_eq!(*capture.lock().expect("capture"), vec![17]);
    instance.stop().expect("stop instance");
}

#[test]
fn component_owned_system_missing_provider_is_a_core_error() {
    let error = Fabric::new("fabric.test.component.system-missing")
        .expect("fabric")
        .component(
            SystemOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::new(Mutex::new(Vec::new())),
            })
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            )),
        )
        .build()
        .expect_err("Core owns missing provider");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::MissingProvider { .. })
    ));
}

#[test]
fn component_system_provider_ambiguity_remains_a_core_error() {
    let error = Fabric::new("fabric.test.component.system-ambiguous")
        .expect("fabric")
        .system(TestOperations::select(TestOperationsConfig::new(1, 1)).expect("system"))
        .block("competing", |block| {
            block.module(CompetingOperationsProvider::new(99))
        })
        .expect("block")
        .component(
            SystemOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::new(Mutex::new(Vec::new())),
            })
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            )),
        )
        .build()
        .expect_err("Core owns provider ambiguity");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::AmbiguousProvider { .. })
    ));
}

#[test]
fn component_system_explicit_selection_chooses_system_and_is_manifest_truth() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let rails = Arc::new(Mutex::new(None));
    let system = TestOperations::select(TestOperationsConfig::new(41, 1)).expect("system");
    let built = Fabric::new("fabric.test.component.system-selected")
        .expect("fabric")
        .system(system.clone())
        .block("competing", |block| {
            block.module(CompetingOperationsProvider::new(99))
        })
        .expect("block")
        .component(
            SystemOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&capture),
            })
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            ))
            .select_system_provider(&system),
        )
        .block("capture", |block| {
            block.module(RailsCaptureModule::new(Arc::clone(&rails)))
        })
        .expect("capture")
        .build()
        .expect("selection resolves Core ambiguity");
    let selection = &built.manifest().component_system_bindings()[0];
    assert_eq!(
        selection.component_id(),
        &SystemOwnedComponent::component_id()
    );
    assert_eq!(selection.system_id(), &TestOperations::system_id());
    assert_eq!(selection.system_id(), &TestOperations::system_id());
    let mut instance = built
        .materialize("fabric.test.component.system-selected.instance")
        .expect("instance");
    instance.start().expect("start");
    rails
        .lock()
        .expect("rails")
        .clone()
        .expect("captured")
        .materializer
        .materialize(&SystemOwnedComponent::component_id())
        .expect("component");
    assert_eq!(*capture.lock().expect("capture"), vec![41]);
    instance.stop().expect("stop instance");
}

#[test]
fn two_components_independently_consume_one_system_and_combined_dependencies_remain_distinct() {
    let a = Arc::new(Mutex::new(Vec::new()));
    let b = Arc::new(Mutex::new(Vec::new()));
    let combined = Arc::new(Mutex::new(Vec::new()));
    let rails = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.component.system-shared")
        .expect("fabric")
        .system(TestOperations::select(TestOperationsConfig::new(7, 1)).expect("system"))
        .resource(
            DirectCounter::select("primary", DirectCounterConfig { value: 5 }).expect("resource"),
        )
        .component(
            SystemOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&a),
            })
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            )),
        )
        .component(
            SystemOwnedComponentB::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&b),
            })
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            )),
        )
        .component(
            CombinedOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&combined),
            })
            .requires_resource(Requires::<DirectCounter>::provisional())
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            )),
        )
        .block("capture", |block| {
            block.module(RailsCaptureModule::new(Arc::clone(&rails)))
        })
        .expect("capture")
        .build()
        .expect("shared System build");
    assert_eq!(
        built.manifest().components()[2]
            .resource_requirements()
            .len(),
        1
    );
    assert_eq!(
        built.manifest().components()[2].system_requirements().len(),
        1
    );
    let mut instance = built
        .materialize("fabric.test.component.system-shared.instance")
        .expect("instance");
    instance.start().expect("start");
    let materializer = rails
        .lock()
        .expect("rails")
        .clone()
        .expect("captured")
        .materializer;
    materializer
        .materialize(&SystemOwnedComponent::component_id())
        .expect("A");
    materializer
        .materialize(&SystemOwnedComponentB::component_id())
        .expect("B");
    materializer
        .materialize(&CombinedOwnedComponent::component_id())
        .expect("combined");
    assert_eq!(*a.lock().expect("A"), vec![7]);
    assert_eq!(*b.lock().expect("B"), vec![8]);
    assert_eq!(*combined.lock().expect("combined"), vec![14]);
    instance.stop().expect("stop instance");
}

#[test]
fn declaration_only_component_keeps_system_requirement_without_participation() {
    let built = Fabric::new("fabric.test.component.declaration-system")
        .expect("fabric")
        .system(TestOperations::select(TestOperationsConfig::new(1, 1)).expect("system"))
        .component(
            PackageComponent::define(PackageComponentConfig {
                prefix: "hello".to_owned(),
            })
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            )),
        )
        .build()
        .expect("declaration-only System build");
    assert_eq!(
        built.manifest().components()[0].system_requirements().len(),
        1
    );
    let mut instance = built
        .materialize("fabric.test.component.declaration-system.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");
}

#[test]
fn component_system_handoffs_are_fresh_for_each_materialized_instance() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let rails = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.component.system-fresh-instances")
        .expect("fabric")
        .system(TestOperations::select(TestOperationsConfig::new(31, 1)).expect("system"))
        .component(
            SystemOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&capture),
            })
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            )),
        )
        .block("capture", |block| {
            block.module(RailsCaptureModule::new(Arc::clone(&rails)))
        })
        .expect("capture")
        .build()
        .expect("build");
    for name in ["one", "two"] {
        let mut instance = built
            .materialize(format!("fabric.test.component.system-fresh.{name}"))
            .expect("instance");
        instance.start().expect("start");
        let materializer = rails
            .lock()
            .expect("rails")
            .clone()
            .expect("captured")
            .materializer;
        materializer
            .materialize(&SystemOwnedComponent::component_id())
            .expect("component");
        instance.stop().expect("stop instance");
    }
    assert_eq!(*capture.lock().expect("capture"), vec![31, 31]);
}

#[test]
fn duplicate_component_system_requirement_is_rejected_by_core_duplicate_module_identity() {
    let error = Fabric::new("fabric.test.component.duplicate-system-requirement")
        .expect("fabric")
        .system(TestOperations::select(TestOperationsConfig::new(1, 1)).expect("system"))
        .component(
            SystemOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::new(Mutex::new(Vec::new())),
            })
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            ))
            .requires_system(SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            )),
        )
        .build()
        .expect_err("identical requirement carriers have one deterministic identity");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::DuplicateModuleId { .. })
    ));
}

#[test]
fn integrated_alpha_system_runs_end_to_end() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let rails = Arc::new(Mutex::new(None));
    let clock_selection = Clock::select("alpha", ClockConfig::default()).expect("clock");
    let clock = clock_selection
        .clone()
        .using(HostBoundClockAdapter::new(100))
        .expect("adapter");
    let system_selection =
        AdaptedOperations::select(AdaptedOperationsConfig::default()).expect("system");
    let system = system_selection
        .clone()
        .using(HostBoundOperationsAdapter::new(
            HostBoundOperationsAdapterConfig { value: 7 },
        ))
        .expect("adapter");
    let built = Fabric::new("fabric.test.alpha.integrated")
        .expect("fabric")
        .component(
            AlphaComponent::define(AlphaConfig {
                capture: Arc::clone(&capture),
            })
            .requires_resource(Requires::<Clock>::versioned(
                ContractVersionRequirement::parse("^1").expect("clock requirement"),
            ))
            .select_resource_provider(&clock_selection)
            .requires_system(SystemRequires::<AdaptedOperations>::versioned(
                ContractVersionRequirement::parse("^2").expect("system requirement"),
            ))
            .select_system_provider(&system_selection),
        )
        .system(system)
        .resource(clock)
        .block("rails", |b| {
            b.module(RailsCaptureModule::new(Arc::clone(&rails)))
        })
        .expect("rails")
        .build()
        .expect("build");
    assert_eq!(built.manifest().resources().len(), 1);
    assert_eq!(built.manifest().systems().len(), 1);
    assert_eq!(
        built.manifest().components()[0]
            .resource_requirements()
            .len(),
        1
    );
    assert_eq!(
        built.manifest().components()[0].system_requirements().len(),
        1
    );
    assert_eq!(built.manifest().component_resource_bindings().len(), 1);
    assert_eq!(built.manifest().component_system_bindings().len(), 1);
    let mut instance = built
        .materialize_on("fabric.test.alpha.instance", &alpha_host())
        .expect("host");
    assert_eq!(instance.lifecycle(), LifecycleState::Ready);
    let generation = instance.generation();
    instance.start().expect("start");
    let report = instance.core().report();
    assert_eq!(report.lifecycle, LifecycleState::Running);
    assert_eq!(report.generation, generation);
    rails
        .lock()
        .expect("rails")
        .clone()
        .expect("captured")
        .materializer
        .materialize(&AlphaComponent::component_id())
        .expect("component");
    assert_eq!(
        *capture.lock().expect("capture"),
        vec![AlphaObservation {
            tick: 100,
            marker: 7
        }]
    );
    assert_eq!(instance.lifecycle(), LifecycleState::Running);
    instance.stop().expect("stop instance");
    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
}

#[test]
fn integrated_alpha_system_rejects_incompatible_host() {
    let observations = Arc::new(Mutex::new(Vec::new()));
    let clock_selection = Clock::select("alpha", ClockConfig::default()).expect("clock");
    let clock = clock_selection
        .clone()
        .using(HostBoundClockAdapter::new(100))
        .expect("adapter");
    let system_selection =
        AdaptedOperations::select(AdaptedOperationsConfig::default()).expect("system");
    let system = system_selection
        .clone()
        .using(HostBoundOperationsAdapter::new(
            HostBoundOperationsAdapterConfig { value: 7 },
        ))
        .expect("adapter");
    let built = Fabric::new("fabric.test.alpha.incompatible-host")
        .expect("fabric")
        .component(
            AlphaComponent::define(AlphaConfig {
                capture: Arc::clone(&observations),
            })
            .requires_resource(Requires::<Clock>::versioned(
                ContractVersionRequirement::parse("^1").expect("clock requirement"),
            ))
            .select_resource_provider(&clock_selection)
            .requires_system(SystemRequires::<AdaptedOperations>::versioned(
                ContractVersionRequirement::parse("^2").expect("system requirement"),
            ))
            .select_system_provider(&system_selection),
        )
        .system(system)
        .resource(clock)
        .build()
        .expect("semantic build");
    let error = built
        .materialize_on("fabric.test.alpha.incompatible.instance", &facility_host())
        .expect_err("operations facility is absent");
    assert!(matches!(error, CompositionError::HostIncompatible { .. }));
    assert!(observations.lock().expect("observations").is_empty());
}

#[test]
fn component_owned_resource_requirement_lowers_through_core_and_reaches_its_runtime() {
    let component_capture = Arc::new(Mutex::new(Vec::new()));
    let rails_capture = Arc::new(Mutex::new(None));
    let resource =
        DirectCounter::select("primary", DirectCounterConfig { value: 73 }).expect("resource");
    let built = Fabric::new("fabric.test.component.resource-owned")
        .expect("fabric")
        .resource(resource)
        .component(
            ResourceOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&component_capture),
            })
            .requires_resource(Requires::<DirectCounter>::provisional()),
        )
        .block("capture", |block| {
            block.module(RailsCaptureModule::new(Arc::clone(&rails_capture)))
        })
        .expect("capture")
        .build()
        .expect("component requirement resolves through Core");

    assert_eq!(
        built.manifest().components()[0]
            .resource_requirements()
            .len(),
        1
    );
    assert_eq!(
        built.manifest().components()[0].resource_requirements()[0].resource_id(),
        &DirectCounter::resource_id()
    );

    let mut instance = built
        .materialize("fabric.test.component.resource-owned.instance")
        .expect("instance");
    instance.start().expect("start");
    rails_capture
        .lock()
        .expect("rails")
        .clone()
        .expect("captured")
        .materializer
        .materialize(&ResourceOwnedComponent::component_id())
        .expect("component materializes");
    assert_eq!(*component_capture.lock().expect("capture"), vec![73]);
    instance.stop().expect("stop instance");
}

#[test]
fn component_owned_resource_requirement_missing_provider_is_a_core_error() {
    let error = Fabric::new("fabric.test.component.resource-missing")
        .expect("fabric")
        .component(
            ResourceOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::new(Mutex::new(Vec::new())),
            })
            .requires_resource(Requires::<DirectCounter>::provisional()),
        )
        .build()
        .expect_err("missing provider must remain Core-owned");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::MissingProvider { .. })
    ));
}

#[test]
fn component_owned_resource_requirement_uses_explicit_resource_selection() {
    let primary =
        DirectCounter::select("primary", DirectCounterConfig { value: 3 }).expect("primary");
    let secondary =
        DirectCounter::select("secondary", DirectCounterConfig { value: 89 }).expect("secondary");
    let capture = Arc::new(Mutex::new(Vec::new()));
    let rails_capture = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.component.resource-selected")
        .expect("fabric")
        .resource(primary)
        .resource(secondary.clone())
        .component(
            ResourceOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&capture),
            })
            .requires_resource(Requires::<DirectCounter>::provisional())
            .select_resource_provider(&secondary),
        )
        .block("capture", |block| {
            block.module(RailsCaptureModule::new(Arc::clone(&rails_capture)))
        })
        .expect("capture")
        .build()
        .expect("explicit ComponentInstanceBinding selection resolves ambiguity");
    assert_eq!(
        built.manifest().diagnostics().provider_selections().len(),
        1
    );
    let mut instance = built
        .materialize("fabric.test.component.resource-selected.instance")
        .expect("instance");
    instance.start().expect("start");
    rails_capture
        .lock()
        .expect("rails")
        .clone()
        .expect("captured")
        .materializer
        .materialize(&ResourceOwnedComponent::component_id())
        .expect("component materializes");
    assert_eq!(*capture.lock().expect("capture"), vec![89]);
    instance.stop().expect("stop instance");
}

#[test]
fn components_with_identical_resource_requirements_keep_independent_selected_providers() {
    let primary =
        DirectCounter::select("primary", DirectCounterConfig { value: 11 }).expect("primary");
    let secondary =
        DirectCounter::select("secondary", DirectCounterConfig { value: 22 }).expect("secondary");
    let a = Arc::new(Mutex::new(Vec::new()));
    let b = Arc::new(Mutex::new(Vec::new()));
    let rails = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.component.independent-resource-providers")
        .expect("fabric")
        .resource(primary.clone())
        .resource(secondary.clone())
        .component(
            ResourceOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&a),
            })
            .requires_resource(Requires::<DirectCounter>::provisional())
            .select_resource_provider(&primary),
        )
        .component(
            ResourceOwnedComponentB::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&b),
            })
            .requires_resource(Requires::<DirectCounter>::provisional())
            .select_resource_provider(&secondary),
        )
        .block("capture", |block| {
            block.module(RailsCaptureModule::new(Arc::clone(&rails)))
        })
        .expect("capture")
        .build()
        .expect("independent selections build");
    assert_eq!(built.manifest().component_resource_bindings().len(), 2);
    let mut instance = built
        .materialize("fabric.test.component.independent-resource-providers.instance")
        .expect("instance");
    instance.start().expect("start");
    let materializer = rails
        .lock()
        .expect("rails")
        .clone()
        .expect("captured")
        .materializer;
    materializer
        .materialize(&ResourceOwnedComponent::component_id())
        .expect("A");
    materializer
        .materialize(&ResourceOwnedComponentB::component_id())
        .expect("B");
    assert_eq!(*a.lock().expect("A capture"), vec![11]);
    assert_eq!(*b.lock().expect("B capture"), vec![22]);
    instance.stop().expect("stop instance");
}

#[test]
fn component_resource_ambiguity_remains_a_core_error() {
    let error = Fabric::new("fabric.test.component.resource-ambiguous")
        .expect("fabric")
        .resource(
            DirectCounter::select("primary", DirectCounterConfig { value: 1 }).expect("primary"),
        )
        .resource(
            DirectCounter::select("secondary", DirectCounterConfig { value: 2 })
                .expect("secondary"),
        )
        .component(
            ResourceOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::new(Mutex::new(Vec::new())),
            })
            .requires_resource(Requires::<DirectCounter>::provisional()),
        )
        .build()
        .expect_err("Core must reject ambiguity");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::AmbiguousProvider { .. })
    ));
}

#[test]
fn components_with_identical_resource_requirements_may_share_one_provider() {
    let provider =
        DirectCounter::select("primary", DirectCounterConfig { value: 44 }).expect("provider");
    let a = Arc::new(Mutex::new(Vec::new()));
    let b = Arc::new(Mutex::new(Vec::new()));
    let rails = Arc::new(Mutex::new(None));
    let built = Fabric::new("fabric.test.component.shared-resource-provider")
        .expect("fabric")
        .resource(provider.clone())
        .component(
            ResourceOwnedComponent::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&a),
            })
            .requires_resource(Requires::<DirectCounter>::provisional())
            .select_resource_provider(&provider),
        )
        .component(
            ResourceOwnedComponentB::define(ResourceOwnedComponentConfig {
                capture: Arc::clone(&b),
            })
            .requires_resource(Requires::<DirectCounter>::provisional())
            .select_resource_provider(&provider),
        )
        .block("capture", |block| {
            block.module(RailsCaptureModule::new(Arc::clone(&rails)))
        })
        .expect("capture")
        .build()
        .expect("shared provider build");
    let mut instance = built
        .materialize("fabric.test.component.shared-resource-provider.instance")
        .expect("instance");
    instance.start().expect("start");
    let materializer = rails
        .lock()
        .expect("rails")
        .clone()
        .expect("captured")
        .materializer;
    materializer
        .materialize(&ResourceOwnedComponent::component_id())
        .expect("A");
    materializer
        .materialize(&ResourceOwnedComponentB::component_id())
        .expect("B");
    assert_eq!(*a.lock().expect("A"), vec![44]);
    assert_eq!(*b.lock().expect("B"), vec![44]);
    instance.stop().expect("stop instance");
}

#[test]
fn declaration_only_component_keeps_resource_requirement_without_participation() {
    let built = Fabric::new("fabric.test.component.declaration-resource")
        .expect("fabric")
        .resource(
            DirectCounter::select("primary", DirectCounterConfig { value: 1 }).expect("provider"),
        )
        .component(
            PackageComponent::define(PackageComponentConfig {
                prefix: "hello".to_owned(),
            })
            .requires_resource(Requires::<DirectCounter>::provisional()),
        )
        .build()
        .expect("declaration-only requirement build");
    assert_eq!(
        built.manifest().components()[0]
            .resource_requirements()
            .len(),
        1
    );
    let mut instance = built
        .materialize("fabric.test.component.declaration-resource.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop().expect("stop instance");
}
