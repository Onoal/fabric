use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use fabric::authoring::*;
use fabric::authoring::{CompositionExt, FabricBuilder};
use fabric::core::{
    CompositionError, CompositionExport, ContractId, Module, ModuleBindings, ModuleContract,
    ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
};
use fabric::prelude::*;

#[derive(Clone)]
struct FixtureConfig {
    events: Arc<Mutex<Vec<String>>>,
    fail_start: bool,
    fail_stop: bool,
}

struct FixtureState {
    events: Arc<Mutex<Vec<String>>>,
    value: AtomicUsize,
    health: AtomicUsize,
    fail_start: bool,
    fail_stop: bool,
}

// These declarations deliberately own semantic truth only. They prove that an
// API, Config, and Relations do not silently create a self runtime.
fabric::resource! {
    ApiOnlyResource {
        id: "fabric.test.lifecycle.api-only-resource";

        api {
            fn read(&self, key: u64) -> u64;
        }
    }
}

fabric::system! {
    ApiOnlySystem {
        id: "fabric.test.lifecycle.api-only-system";

        api {
            fn now(&self) -> u64;
        }
    }
}

// Semantic occurrence configuration and concrete Adapter configuration are
// supplied independently. The Resource Relation is retained on the selected
// Adapter-owned participant without reviving a Resource runtime proxy.
fabric::resource! {
    CanonicalConfiguredResource {
        id: "fabric.test.lifecycle.canonical-configured-resource";

        config {
            namespace: String;
        }

        relations {
            requires {
                store: CanonicalAdapterResource;
            }
        }

        api {
            fn label(&self) -> String;
        }
    }
}

struct CanonicalAdapterState {
    value: AtomicUsize,
    events: Arc<Mutex<Vec<String>>>,
}

fabric::resource! {
    CanonicalAdapterResource {
        id: "fabric.test.lifecycle.canonical-adapter-resource";

        api {
            fn read(&self) -> usize;
        }
    }
}

fabric::system! {
    CanonicalAdapterSystem {
        id: "fabric.test.lifecycle.canonical-adapter-system";

        api {
            fn now(&self) -> usize;
        }
    }
}

// The target alone selects the normal realization contract.  In particular,
// this authoring names neither `resource`/`system` nor a generated interface.
fabric::adapter! {
    CanonicalResourceAdapter for CanonicalAdapterResource {
        id: "test.canonical-resource-adapter";
        config {
            events: Arc<Mutex<Vec<String>>>;
        }

        state {
            CanonicalAdapterState = CanonicalAdapterState {
                value: AtomicUsize::new(0),
                events: Arc::clone(&config.events),
            };
        }

        runtime {
            fn read(&self) -> usize {
                self.state.get().value.fetch_add(1, Ordering::SeqCst) + 1
            }
        }

        lifecycle {
            initialize {
                self.state.get().events.lock().expect("events").push("initialize".to_owned());
                Ok(())
            }

            start {
                self.state.get().events.lock().expect("events").push("start".to_owned());
                Ok(())
            }

            stop {
                self.state.get().events.lock().expect("events").push("stop".to_owned());
                Ok(())
            }

            health: Health::Degraded;
        }
    }
}

fabric::adapter! {
    CanonicalConfiguredAdapter for CanonicalConfiguredResource {
        id: "test.canonical-configured-adapter";
        config {
            prefix: String;
        }

        runtime {
            fn label(&self) -> String {
                self.config.prefix.clone()
            }
        }
    }
}

// Differential realization keeps `read` semantic while the Adapter owns the
// lower-level `read_bytes` operation and the direct `write` operation.
fabric::resource! {
    DifferentialDocumentStore {
        id: "fabric.test.lifecycle.differential-document-store";

        api {
            fn read(&self, key: String) -> String;
            fn write(&self, key: String, value: String) -> usize;
        }

        realization {
            mediate read;
            fn read_bytes(&self, key: String) -> Vec<u8>;
        }

        runtime {
            fn read(&self, key: String) -> String {
                String::from_utf8(self.realization.read_bytes(key)).expect("utf8")
            }
        }
    }
}

fabric::adapter! {
    DifferentialDocumentAdapter for DifferentialDocumentStore {
        id: "test.differential-document-adapter";
        runtime {
            fn write(&self, _key: String, value: String) -> usize { value.len() }
            fn read_bytes(&self, key: String) -> Vec<u8> { key.into_bytes() }
        }
    }
}

fabric::system! {
    DifferentialClock {
        id: "fabric.test.lifecycle.differential-clock";
        api { fn now(&self) -> u64; fn label(&self) -> String; }
        realization {
            mediate now;
            fn raw_now(&self) -> u64;
        }
        runtime {
            fn now(&self) -> u64 { self.realization.raw_now() + 1 }
        }
    }
}

fabric::adapter! {
    DifferentialClockAdapter for DifferentialClock {
        id: "test.differential-clock-adapter";
        runtime {
            fn label(&self) -> String { "direct".to_owned() }
            fn raw_now(&self) -> u64 { 41 }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CanonicalAdapterObservation {
    first_resource: usize,
    second_resource: usize,
    system: usize,
    label: String,
}

type DifferentialObservation = (String, usize, u64, String);

#[derive(Clone)]
struct DifferentialConsumer {
    module_id: ModuleId,
    document: ContractDependency<differential_document_store::raw::ApiContract>,
    clock: ContractDependency<differential_clock::raw::ApiContract>,
    observation: Arc<Mutex<Option<DifferentialObservation>>>,
}

impl DifferentialConsumer {
    fn new(observation: Arc<Mutex<Option<DifferentialObservation>>>) -> Self {
        Self {
            module_id: ModuleId::new("fabric.test.lifecycle.differential.consumer")
                .expect("module id"),
            document: ContractDependency::new(
                Requires::<DifferentialDocumentStore>::provisional()
                    .as_contract_requirement()
                    .clone(),
            ),
            clock: ContractDependency::new(
                SystemRequires::<DifferentialClock>::provisional()
                    .as_contract_requirement()
                    .clone(),
            ),
            observation,
        }
    }
}

impl ModuleRuntime for DifferentialConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }
    fn required_contract_declarations(&self) -> Vec<fabric::core::ContractRequirementDeclaration> {
        vec![
            self.document.declaration().clone(),
            self.clock.declaration().clone(),
        ]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }
    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.document
            .bind(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        self.clock
            .bind(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        Ok(())
    }
    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn start(&mut self) -> Result<(), ModuleError> {
        *self.observation.lock().expect("observation") = Some((
            self.document.read("read".to_owned()),
            self.document.write("key".to_owned(), "value".to_owned()),
            self.clock.now(),
            self.clock.label(),
        ));
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
struct DifferentialConsumerModule {
    observation: Arc<Mutex<Option<DifferentialObservation>>>,
}
impl DifferentialConsumerModule {
    fn new(observation: Arc<Mutex<Option<DifferentialObservation>>>) -> Self {
        Self { observation }
    }
}
impl Module for DifferentialConsumerModule {
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::from_runtime(&DifferentialConsumer::new(Arc::clone(&self.observation)))
    }
    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(DifferentialConsumer::new(Arc::clone(
            &self.observation,
        ))))
    }
}

#[derive(Clone)]
struct CanonicalAdapterConsumer {
    module_id: ModuleId,
    resource: ContractDependency<canonical_adapter_resource::raw::ApiContract>,
    system: ContractDependency<canonical_adapter_system::raw::ApiContract>,
    configured: ContractDependency<canonical_configured_resource::raw::ApiContract>,
    observation: Arc<Mutex<Option<CanonicalAdapterObservation>>>,
}

impl CanonicalAdapterConsumer {
    fn new(observation: Arc<Mutex<Option<CanonicalAdapterObservation>>>) -> Self {
        Self {
            module_id: ModuleId::new("fabric.test.lifecycle.canonical-adapter.consumer")
                .expect("static module id"),
            resource: ContractDependency::new(
                Requires::<CanonicalAdapterResource>::provisional()
                    .as_contract_requirement()
                    .clone(),
            ),
            system: ContractDependency::new(
                SystemRequires::<CanonicalAdapterSystem>::provisional()
                    .as_contract_requirement()
                    .clone(),
            ),
            configured: ContractDependency::new(
                Requires::<CanonicalConfiguredResource>::provisional()
                    .as_contract_requirement()
                    .clone(),
            ),
            observation,
        }
    }
}

impl ModuleRuntime for CanonicalAdapterConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric::core::ContractRequirementDeclaration> {
        vec![
            self.resource.declaration().clone(),
            self.system.declaration().clone(),
            self.configured.declaration().clone(),
        ]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.resource
            .bind(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        self.system
            .bind(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        self.configured
            .bind(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        *self.observation.lock().expect("observation") = Some(CanonicalAdapterObservation {
            first_resource: self.resource.read(),
            second_resource: self.resource.read(),
            system: self.system.now(),
            label: self.configured.label(),
        });
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
struct CanonicalAdapterConsumerModule {
    observation: Arc<Mutex<Option<CanonicalAdapterObservation>>>,
}

impl CanonicalAdapterConsumerModule {
    fn new(observation: Arc<Mutex<Option<CanonicalAdapterObservation>>>) -> Self {
        Self { observation }
    }
}

impl Module for CanonicalAdapterConsumerModule {
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::from_runtime(&CanonicalAdapterConsumer::new(Arc::clone(
            &self.observation,
        )))
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(CanonicalAdapterConsumer::new(Arc::clone(
            &self.observation,
        ))))
    }
}

fabric::adapter! {
    CanonicalSystemAdapter for CanonicalAdapterSystem {
        id: "test.canonical-system-adapter";
        runtime {
            fn now(&self) -> usize {
                42
            }
        }
    }
}

fabric::resource! {
    ApiOnlyConfiguredRelationResource {
        id: "fabric.test.lifecycle.api-only-configured-relation";

        config {
            namespace: String;
        }

        relations {
            requires {
                store: ApiOnlyResource;
                clock: ApiOnlySystem;
            }
        }

        api {
            fn label(&self) -> String;
        }
    }
}

impl FixtureState {
    fn new(events: Arc<Mutex<Vec<String>>>, fail_start: bool, fail_stop: bool) -> Self {
        Self {
            events,
            value: AtomicUsize::new(0),
            health: AtomicUsize::new(0),
            fail_start,
            fail_stop,
        }
    }

    fn event(&self, event: &str) {
        self.events.lock().expect("events").push(event.to_owned());
    }
}

fabric::resource! {
    StatefulFixtureResource {
        id: "fabric.test.lifecycle.stateful-resource";


        config {
            fixture: FixtureConfig;
        }

        api {
            fn increment(&self) -> usize;
            fn mark_degraded(&self);
            fn mark_unavailable(&self);
        }

        state {
            FixtureState = FixtureState::new(config.fixture.events.clone(), config.fixture.fail_start, config.fixture.fail_stop);
        }

        runtime {
            fn increment(&self) -> usize {
                self.state.get().value.fetch_add(1, Ordering::SeqCst) + 1
            }

            fn mark_unavailable(&self) {
                self.state.get().health.store(2, Ordering::SeqCst);
            }

            fn mark_degraded(&self) {
                self.state.get().health.store(1, Ordering::SeqCst);
            }
        }

        lifecycle {
            initialize {
                assert!(self.runtime_context.is_bound());
                self.state.get().event("resource.initialize");
                Ok(())
            }

            start {
                self.state.get().event("resource.start");
                if self.state.get().fail_start {
                    Err(ModuleError::new("resource start failure"))
                } else {
                    Ok(())
                }
            }

            stop {
                self.state.get().event("resource.stop");
                if self.state.get().fail_stop {
                    Err(ModuleError::new("resource stop failure"))
                } else {
                    Ok(())
                }
            }

            health: match self.state.get().health.load(Ordering::SeqCst) {
                0 => Health::Healthy,
                1 => Health::Degraded,
                _ => Health::Unavailable,
            };
        }
    }
}

fabric::system! {
    StatefulFixtureSystem {
        id: "fabric.test.lifecycle.stateful-system";


        config {
            fixture: FixtureConfig;
        }

        api {
            fn current(&self) -> usize;
        }

        state {
            FixtureState = FixtureState::new(config.fixture.events.clone(), config.fixture.fail_start, config.fixture.fail_stop);
        }

        runtime {
            fn current(&self) -> usize {
                self.state.get().value.load(Ordering::SeqCst)
            }
        }

        lifecycle {
            initialize {
                assert!(self.runtime_context.is_bound());
                self.state.get().event("system.initialize");
                Ok(())
            }

            start {
                self.state.get().event("system.start");
                Ok(())
            }

            stop {
                self.state.get().event("system.stop");
                Ok(())
            }
        }
    }
}

fabric::resource! {
    AdaptedFixtureResource {
        id: "fabric.test.lifecycle.adapted-resource";


        config {}

        api { fn current(&self) -> usize; }
    }
}

fn adapter_host_facility() -> HostFacilityId {
    HostFacilityId::new("fabric.test.lifecycle.adapter-state").expect("static host facility")
}

fabric::adapter! {
    StatefulFixtureResourceAdapter for AdaptedFixtureResource {
        id: "test.stateful-fixture-resource-adapter";

        config {
            fixture: FixtureConfig;
        }

        host: HostRequirement::new().require_facility(adapter_host_facility());

        state {
            FixtureState = FixtureState::new(config.fixture.events.clone(), config.fixture.fail_start, config.fixture.fail_stop);
        }

        runtime {
            fn current(&self) -> usize {
                self.state.get().value.fetch_add(1, Ordering::SeqCst)
            }
        }

        lifecycle {
            initialize {
                assert!(self.runtime_context.is_bound());
                self.state.get().event("resource-adapter.initialize");
                Ok(())
            }

            start {
                self.state.get().event("resource-adapter.start");
                Ok(())
            }

            stop {
                self.state.get().event("resource-adapter.stop");
                Ok(())
            }
        }
    }
}

fabric::system! {
    AdaptedFixtureSystem {
        id: "fabric.test.lifecycle.adapted-system";


        config {}

        api { fn current(&self) -> usize; }
    }
}

fabric::adapter! {
    StatefulFixtureSystemAdapter for AdaptedFixtureSystem {
        id: "test.stateful-fixture-system-adapter";

        config {
            fixture: FixtureConfig;
        }

        relations { requires { prerequisite: StatefulFixtureSystem(provisional); } }

        state {
            FixtureState = FixtureState::new(config.fixture.events.clone(), config.fixture.fail_start, config.fixture.fail_stop);
        }

        runtime {
            fn current(&self) -> usize {
                self.prerequisite.current()
                    + self.state.get().value.fetch_add(1, Ordering::SeqCst)
            }
        }

        lifecycle {
            initialize {
                assert!(self.runtime_context.is_bound());
                self.state.get().event("system-adapter.initialize");
                Ok(())
            }

            start {
                self.state.get().event("system-adapter.start");
                Ok(())
            }

            stop {
                self.state.get().event("system-adapter.stop");
                Ok(())
            }
        }
    }
}

fn host() -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("architecture"),
    )
}

fn resource_export() -> CompositionExport<stateful_fixture_resource::raw::ApiContract> {
    CompositionExport::new(
        ContractId::new("fabric.test.lifecycle.stateful-resource.export").expect("export id"),
        Requires::<StatefulFixtureResource>::provisional()
            .as_contract_requirement()
            .clone(),
    )
}

#[test]
fn canonical_adapters_export_the_target_semantic_api_without_subject_forwarding() {
    let resource_selection = CanonicalAdapterResource::select("store").expect("resource selection");
    let system_selection = CanonicalAdapterSystem::select().expect("system selection");
    let observation = Arc::new(Mutex::new(None));
    let events = Arc::new(Mutex::new(Vec::new()));
    let composition = Fabric::new("fabric.test.lifecycle.canonical-adapter")
        .expect("fabric")
        .resource(
            resource_selection
                .clone()
                .using(CanonicalResourceAdapter::new(
                    CanonicalResourceAdapterConfig {
                        events: Arc::clone(&events),
                    },
                ))
                .expect("canonical resource adapter"),
        )
        .system(
            system_selection
                .clone()
                .using(CanonicalSystemAdapter::new())
                .expect("canonical system adapter"),
        )
        .resource(
            CanonicalConfiguredResource::select(
                "configured",
                CanonicalConfiguredResourceConfig {
                    namespace: "tenant-a".to_owned(),
                },
            )
            .expect("configured semantic Resource")
            .using(CanonicalConfiguredAdapter::new(
                CanonicalConfiguredAdapterConfig {
                    prefix: "adapter-config".to_owned(),
                },
            ))
            .expect("configured canonical Adapter"),
        )
        .block("consumer", |block| {
            block.module(CanonicalAdapterConsumerModule::new(Arc::clone(
                &observation,
            )))
        })
        .expect("consumer block")
        .build()
        .expect("canonical adapters compose");

    let mut instance = composition
        .materialize_on("canonical-adapter", &host())
        .expect("canonical adapters materialize");
    assert_eq!(instance.lifecycle(), LifecycleState::Ready);
    instance.start().expect("canonical adapters start");
    assert_eq!(instance.lifecycle(), LifecycleState::Running);
    assert_eq!(
        observation.lock().expect("observation").clone(),
        Some(CanonicalAdapterObservation {
            first_resource: 1,
            second_resource: 2,
            system: 42,
            label: "adapter-config".to_owned(),
        })
    );
    assert_eq!(instance.core().report().health, Health::Degraded);
    instance.stop().expect("canonical adapters stop");
    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
    assert_eq!(
        events.lock().expect("events").as_slice(),
        ["initialize", "start", "stop"]
    );

    let mut fresh_generation = composition
        .materialize_on("canonical-adapter-fresh", &host())
        .expect("fresh canonical adapter generation");
    fresh_generation.start().expect("fresh start");
    assert_eq!(
        observation.lock().expect("observation").clone(),
        Some(CanonicalAdapterObservation {
            first_resource: 1,
            second_resource: 2,
            system: 42,
            label: "adapter-config".to_owned(),
        })
    );
    fresh_generation.stop().expect("fresh stop");
}

#[test]
fn differential_realizations_compose_one_semantic_provider_from_one_effective_adapter_provider() {
    let observation = Arc::new(Mutex::new(None));
    let composition = Fabric::new("fabric.test.lifecycle.differential")
        .expect("fabric")
        .resource(
            DifferentialDocumentStore::select("documents")
                .expect("selection")
                .using(DifferentialDocumentAdapter::new())
                .expect("adapter"),
        )
        .system(
            DifferentialClock::select()
                .expect("selection")
                .using(DifferentialClockAdapter::new())
                .expect("adapter"),
        )
        .block("consumer", |block| {
            block.module(DifferentialConsumerModule::new(Arc::clone(&observation)))
        })
        .expect("consumer")
        .build()
        .expect("differential composition");
    let mut instance = composition
        .materialize_on("differential", &host())
        .expect("materialize differential providers");
    instance.start().expect("start");
    assert_eq!(
        observation.lock().expect("observation").clone(),
        Some(("read".to_owned(), 5, 42, "direct".to_owned()))
    );
    instance.stop().expect("stop");
}

#[test]
fn stateful_resource_runtime_is_fresh_shared_and_lifecycle_aware() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let selection = StatefulFixtureResource::select(
        "primary",
        StatefulFixtureResourceConfig {
            fixture: FixtureConfig {
                events: Arc::clone(&events),
                fail_start: false,
                fail_stop: false,
            },
        },
    )
    .expect("resource selection");
    let export = resource_export();
    let composition = FabricBuilder::new("fabric.test.lifecycle.resource")
        .expect("builder")
        .block("runtime", |block| block.module(selection))
        .expect("block")
        .export(export.clone())
        .build()
        .expect("composition");

    let mut first = composition
        .materialize_core_on("fabric.test.lifecycle.resource.first", &host())
        .expect("first materialization");
    first.start().expect("first start");
    let service = first.export(&export).expect("resource export");
    assert_eq!(service.increment(), 1);
    assert_eq!(service.increment(), 2);
    assert_eq!(first.report().health, Health::Healthy);
    service.mark_degraded();
    assert_eq!(first.lifecycle(), LifecycleState::Running);
    assert_eq!(first.report().health, Health::Degraded);
    service.mark_unavailable();
    assert_eq!(first.lifecycle(), LifecycleState::Running);
    assert_eq!(first.report().health, Health::Unavailable);
    first.stop().expect("first stop");

    let mut second = composition
        .materialize_core_on("fabric.test.lifecycle.resource.second", &host())
        .expect("second materialization");
    second.start().expect("second start");
    let second_service = second.export(&export).expect("second export");
    assert_eq!(second_service.increment(), 1);
    second.stop().expect("second stop");

    assert_eq!(
        events.lock().expect("events").as_slice(),
        [
            "resource.initialize",
            "resource.start",
            "resource.stop",
            "resource.initialize",
            "resource.start",
            "resource.stop",
        ]
    );
}

#[test]
fn api_only_resource_and_system_are_valid_semantic_definitions_without_self_runtime() {
    let resource = ApiOnlyResource::select("primary").expect("resource selection");
    let system = ApiOnlySystem::select().expect("system selection");

    assert!(<ApiOnlyResource as ResourceDefinition>::materialize(&resource).is_none());
    assert!(<ApiOnlySystem as SystemDefinition>::materialize(&system).is_none());

    let resource_built = Fabric::new("fabric.test.lifecycle.api-only-resource")
        .expect("fabric")
        .resource(resource)
        .build()
        .expect("semantic resource declaration builds");
    let resource_error = resource_built
        .materialize_on("api-only-resource", &host())
        .expect_err("semantic resource without a live realization must not materialize");
    assert!(matches!(
        resource_error,
        CompositionError::MissingRuntimeMaterializer { .. }
    ));

    let system_built = Fabric::new("fabric.test.lifecycle.api-only-system")
        .expect("fabric")
        .system(system)
        .build()
        .expect("semantic system declaration builds");
    let system_error = system_built
        .materialize_on("api-only-system", &host())
        .expect_err("semantic system without a live realization must not materialize");
    assert!(matches!(
        system_error,
        CompositionError::MissingRuntimeMaterializer { .. }
    ));
}

#[test]
fn config_relations_and_api_do_not_imply_a_resource_owned_runtime() {
    let built = Fabric::new("fabric.test.lifecycle.api-only-facets")
        .expect("fabric")
        .resource(ApiOnlyResource::select("store").expect("store"))
        .system(ApiOnlySystem::select().expect("clock"))
        .resource(
            ApiOnlyConfiguredRelationResource::select(
                "configured",
                ApiOnlyConfiguredRelationResourceConfig {
                    namespace: "users".to_owned(),
                },
            )
            .expect("configured semantic selection"),
        )
        .build()
        .expect("semantic facets resolve declaratively");

    assert_eq!(built.manifest().resources().len(), 2);
    assert_eq!(built.manifest().systems().len(), 1);
    assert!(matches!(
        built.materialize_on("api-only-facets", &host()),
        Err(CompositionError::MissingRuntimeMaterializer { .. })
    ));
}

#[test]
fn stateful_resource_ready_stop_and_stop_failure_use_core_cleanup() {
    let ready_events = Arc::new(Mutex::new(Vec::new()));
    let ready = StatefulFixtureResource::select(
        "ready",
        StatefulFixtureResourceConfig {
            fixture: FixtureConfig {
                events: Arc::clone(&ready_events),
                fail_start: false,
                fail_stop: false,
            },
        },
    )
    .expect("resource selection");
    let mut ready_instance = FabricBuilder::new("fabric.test.lifecycle.ready-stop")
        .expect("builder")
        .block("runtime", |block| block.module(ready))
        .expect("block")
        .build()
        .expect("composition")
        .materialize_core_on("fabric.test.lifecycle.ready-stop.instance", &host())
        .expect("materialize");
    ready_instance.stop().expect("ready stop");
    assert_eq!(ready_instance.lifecycle(), LifecycleState::Stopped);
    assert_eq!(
        ready_events.lock().expect("events").as_slice(),
        ["resource.stop"]
    );

    let failing = StatefulFixtureResource::select(
        "failing",
        StatefulFixtureResourceConfig {
            fixture: FixtureConfig {
                events: Arc::new(Mutex::new(Vec::new())),
                fail_start: false,
                fail_stop: true,
            },
        },
    )
    .expect("resource selection");
    let mut failing_instance = FabricBuilder::new("fabric.test.lifecycle.stop-failure")
        .expect("builder")
        .block("runtime", |block| block.module(failing))
        .expect("block")
        .build()
        .expect("composition")
        .materialize_core_on("fabric.test.lifecycle.stop-failure.instance", &host())
        .expect("materialize");
    failing_instance.start().expect("start");
    let cleanup = failing_instance
        .stop()
        .expect_err("stop failure is observable");
    assert_eq!(cleanup.failures().len(), 1);
    assert_eq!(failing_instance.lifecycle(), LifecycleState::Stopped);
}

#[test]
fn startup_abandonment_reaches_high_level_authored_stop_hooks() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let first = StatefulFixtureResource::select(
        "first",
        StatefulFixtureResourceConfig {
            fixture: FixtureConfig {
                events: Arc::clone(&events),
                fail_start: false,
                fail_stop: false,
            },
        },
    )
    .expect("first selection");
    let failing = StatefulFixtureResource::select(
        "failing",
        StatefulFixtureResourceConfig {
            fixture: FixtureConfig {
                events: Arc::clone(&events),
                fail_start: true,
                fail_stop: false,
            },
        },
    )
    .expect("failing selection");
    let mut instance = FabricBuilder::new("fabric.test.lifecycle.startup-abandonment")
        .expect("builder")
        .block("runtime", |block| block.module(first).module(failing))
        .expect("block")
        .build()
        .expect("composition")
        .materialize_core_on(
            "fabric.test.lifecycle.startup-abandonment.instance",
            &host(),
        )
        .expect("materialize");

    assert!(instance.start().is_err());
    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
    assert_eq!(
        events.lock().expect("events").as_slice(),
        [
            "resource.initialize",
            "resource.initialize",
            "resource.start",
            "resource.start",
            "resource.stop",
            "resource.stop",
        ]
    );
}

#[test]
fn stateful_system_runtime_uses_the_same_lifecycle_authoring_path() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let system = StatefulFixtureSystem::select(StatefulFixtureSystemConfig {
        fixture: FixtureConfig {
            events: Arc::clone(&events),
            fail_start: false,
            fail_stop: false,
        },
    })
    .expect("system selection");
    let mut instance = FabricBuilder::new("fabric.test.lifecycle.system")
        .expect("builder")
        .block("runtime", |block| block.module(system))
        .expect("block")
        .build()
        .expect("composition")
        .materialize_core_on("fabric.test.lifecycle.system.instance", &host())
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop");
    assert_eq!(
        events.lock().expect("events").as_slice(),
        ["system.initialize", "system.start", "system.stop"]
    );
}

#[test]
fn stateful_resource_adapter_preserves_host_compatibility_and_shared_occurrence_state() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let adapted = AdaptedFixtureResource::select("primary", AdaptedFixtureResourceConfig {})
        .expect("resource selection")
        .using(StatefulFixtureResourceAdapter::new(
            StatefulFixtureResourceAdapterConfig {
                fixture: FixtureConfig {
                    events: Arc::clone(&events),
                    fail_start: false,
                    fail_stop: false,
                },
            },
        ))
        .expect("adapter");
    let composition = Fabric::new("fabric.test.lifecycle.resource-adapter")
        .expect("builder")
        .resource(adapted)
        .build()
        .expect("composition");
    let mut instance = composition
        .materialize_on(
            "fabric.test.lifecycle.resource-adapter.instance",
            &host().with_facility(adapter_host_facility()),
        )
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop");
    assert_eq!(
        events.lock().expect("events").as_slice(),
        [
            "resource-adapter.initialize",
            "resource-adapter.start",
            "resource-adapter.stop",
        ]
    );
}

#[test]
fn stateful_system_adapter_binds_declared_system_dependencies() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let prerequisite = StatefulFixtureSystem::select(StatefulFixtureSystemConfig {
        fixture: FixtureConfig {
            events: Arc::clone(&events),
            fail_start: false,
            fail_stop: false,
        },
    })
    .expect("prerequisite");
    let adapted = AdaptedFixtureSystem::select(AdaptedFixtureSystemConfig {})
        .expect("system selection")
        .using(StatefulFixtureSystemAdapter::new(
            StatefulFixtureSystemAdapterConfig {
                fixture: FixtureConfig {
                    events: Arc::clone(&events),
                    fail_start: false,
                    fail_stop: false,
                },
            },
        ))
        .expect("adapter");
    let composition = Fabric::new("fabric.test.lifecycle.system-adapter")
        .expect("builder")
        .system(prerequisite)
        .system(adapted)
        .build()
        .expect("composition");
    let mut instance = composition
        .materialize_on("fabric.test.lifecycle.system-adapter.instance", &host())
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop");
    assert!(
        events
            .lock()
            .expect("events")
            .iter()
            .any(|event| event == "system-adapter.stop")
    );
}

fabric::adapter! {
    RenamedIdentityWitness for CanonicalAdapterResource {
        id: "test.explicit-name-independent";
        runtime {
            fn read(&self) -> usize { 7 }
        }
    }
}

#[test]
fn adapter_definition_identity_is_explicit_and_config_independent() {
    let first = CanonicalConfiguredAdapter::new(CanonicalConfiguredAdapterConfig {
        prefix: "first".to_owned(),
    });
    let second = CanonicalConfiguredAdapter::new(CanonicalConfiguredAdapterConfig {
        prefix: "second".to_owned(),
    });

    assert_eq!(
        first.adapter_definition_id(),
        second.adapter_definition_id(),
        "Adapter Config must not alter definition identity",
    );
    assert_eq!(
        first.adapter_definition_id().as_str(),
        "test.canonical-configured-adapter",
    );
    assert_eq!(
        RenamedIdentityWitness::new()
            .adapter_definition_id()
            .as_str(),
        "test.explicit-name-independent",
        "macro identity must come from id:, not its Rust type name",
    );
}

#[test]
fn one_adapter_definition_reuses_its_identity_across_resource_occurrences() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let primary = CanonicalResourceAdapter::new(CanonicalResourceAdapterConfig {
        events: Arc::clone(&events),
    });
    let archive = CanonicalResourceAdapter::new(CanonicalResourceAdapterConfig { events });
    assert_eq!(
        primary.adapter_definition_id(),
        archive.adapter_definition_id()
    );

    let composition = Fabric::new("fabric.test.lifecycle.adapter-identity-occurrences")
        .expect("fabric")
        .resource(
            CanonicalAdapterResource::select("primary")
                .expect("primary selection")
                .using(primary)
                .expect("primary Adapter"),
        )
        .resource(
            CanonicalAdapterResource::select("archive")
                .expect("archive selection")
                .using(archive)
                .expect("archive Adapter"),
        )
        .build()
        .expect("composition");

    let module_ids = composition
        .manifest()
        .diagnostics()
        .module_declarations()
        .iter()
        .map(|declaration| declaration.module_id().clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        module_ids.len(),
        2,
        "each semantic occurrence retains its provider ModuleId"
    );

    let mut instance = composition
        .materialize_on("adapter-identity-occurrences", &host())
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop");
}
