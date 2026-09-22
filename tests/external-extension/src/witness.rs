use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use fabric::authoring::CompositionExt;
use fabric::*;
use fabric_test_adapter_clock_memory::MemoryClock;
use fabric_test_resource_clock::{Clock, ClockConfig, ClockError};

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReadClockInput;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReadClockOutput {
    tick: Result<u64, ClockError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TeardownInput;

#[derive(Clone, Debug, PartialEq, Eq)]
struct TeardownOutput;

fn test_host() -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("architecture"),
    )
}

struct ExternalLifecycleState {
    events: Arc<Mutex<Vec<String>>>,
    value: AtomicUsize,
}

impl ExternalLifecycleState {
    fn event(&self, event: &str) {
        self.events.lock().expect("events").push(event.to_owned());
    }
}

fabric::resource! {
    ExternalStatefulResource {
        id: "fabric.test.external.stateful-resource";

        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.external.stateful-resource.api";
                version: provisional;

                fn current(&self) -> usize;
            }
        }

        adapter Adapter {
            id: "fabric.test.external.stateful-resource.adapter";
            compatibility: provisional;

            fn current(&self) -> usize;
        }

        runtime {
            fn current(&self) -> usize {
                self.adapter.current()
            }
        }
    }
}

fn config_events() -> &'static Mutex<Vec<String>> {
    static EVENTS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    EVENTS.get_or_init(|| Mutex::new(Vec::new()))
}

#[derive(Clone, Debug)]
enum Backend {
    Memory,
    Sqlite { path: PathBuf },
    Redis { endpoint: String, database: u32 },
}

#[derive(Clone, Debug)]
struct BackendDefaults {
    label: String,
}

/// A creator-owned Config type can expose constructors and nested Rust choices
/// without a Fabric wrapper or a second configuration language.
#[derive(Clone)]
struct LocalKeyValueConfig {
    backend: Backend,
    defaults: BackendDefaults,
}

impl LocalKeyValueConfig {
    fn redis(endpoint: impl Into<String>, database: u32) -> Self {
        Self {
            backend: Backend::Redis {
                endpoint: endpoint.into(),
                database,
            },
            defaults: BackendDefaults {
                label: "local".to_owned(),
            },
        }
    }

    fn backend_label(&self) -> String {
        match &self.backend {
            Backend::Memory => "memory".to_owned(),
            Backend::Sqlite { path } => format!("sqlite:{}", path.display()),
            Backend::Redis { endpoint, database } => format!("redis:{endpoint}/{database}"),
        }
    }
}

#[derive(Clone)]
struct ResourceCreatorConfig {
    namespace: String,
}

impl ResourceCreatorConfig {
    fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
        }
    }
}

#[derive(Clone)]
struct SystemCreatorConfig {
    label: String,
    defaults: BackendDefaults,
}

impl SystemCreatorConfig {
    fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            defaults: BackendDefaults {
                label: "system".to_owned(),
            },
        }
    }
}

fabric::resource! {
    ConfiguredKeyValue {
        id: "fabric.test.external.configured-key-value";
        config: ResourceCreatorConfig;

        contracts {
            primary Api {
                id: "fabric.test.external.configured-key-value.api";
                fn namespace(&self) -> String;
            }
        }

        adapter Adapter {
            id: "fabric.test.external.configured-key-value.adapter";
            compatibility: provisional;
            fn namespace(&self) -> String;
        }

        runtime {
            fn namespace(&self) -> String {
                self.config().namespace.clone()
            }
        }

        lifecycle {
            initialize {
                config_events().lock().expect("events").push(format!(
                    "resource.initialize:{}",
                    self.config().namespace,
                ));
                Ok(())
            }

            start {
                config_events().lock().expect("events").push(format!(
                    "resource.start:{}",
                    self.config().namespace,
                ));
                Ok(())
            }

            stop {
                config_events().lock().expect("events").push(format!(
                    "resource.stop:{}",
                    self.config().namespace,
                ));
                Ok(())
            }

            health: Health::Healthy;
        }
    }
}

fabric::adapter! {
    LocalKeyValue
        for resource ConfiguredKeyValue
        implements ConfiguredKeyValueRealization
    {
        config: LocalKeyValueConfig;

        runtime {
            fn namespace(&self) -> String {
                format!(
                    "{}:{}",
                    self.config().defaults.label,
                    self.config().backend_label(),
                )
            }
        }

        lifecycle {
            initialize {
                config_events().lock().expect("events").push(format!(
                    "adapter.initialize:{}",
                    self.config().backend_label(),
                ));
                Ok(())
            }

            health: Health::Healthy;
        }
    }
}

fabric::system! {
    ConfiguredSystem {
        id: "fabric.test.external.configured-system";
        config: SystemCreatorConfig;

        contracts {
            primary Api {
                id: "fabric.test.external.configured-system.api";
                fn label(&self) -> String;
            }
        }

        adapter Adapter {
            id: "fabric.test.external.configured-system.adapter";
            compatibility: provisional;
            fn label(&self) -> String;
        }

        runtime {
            fn label(&self) -> String {
                self.config().label.clone()
            }
        }

        lifecycle {
            initialize {
                config_events().lock().expect("events").push(format!(
                    "system.initialize:{}:{}",
                    self.config().defaults.label,
                    self.config().label,
                ));
                Ok(())
            }
        }
    }
}

fabric::adapter! {
    ConfiguredSystemAdapter
        for system ConfiguredSystem
        implements ConfiguredSystemRealization
    {
        config {
            endpoint: String;
        }

        runtime {
            fn label(&self) -> String {
                self.config().endpoint.clone()
            }
        }

        lifecycle {
            initialize {
                config_events().lock().expect("events").push(format!(
                    "system-adapter.initialize:{}",
                    self.config().endpoint,
                ));
                Ok(())
            }
        }
    }
}

fabric::system! {
    UnconfiguredSystem {
        id: "fabric.test.external.unconfigured-system";

        contracts {
            primary Api {
                id: "fabric.test.external.unconfigured-system.api";
            }
        }

        adapter Adapter {
            id: "fabric.test.external.unconfigured-system.adapter";
            compatibility: provisional;
        }

        runtime {}
    }
}

fabric::adapter! {
    UnconfiguredSystemAdapter
        for system UnconfiguredSystem
        implements UnconfiguredSystemRealization
    {
        runtime {}
    }
}

#[derive(Default)]
struct CleanMemoryState {
    values: Mutex<BTreeMap<Vec<u8>, Vec<u8>>>,
}

// A third-party KeyValue-style semantic definition using the 0.4.1 normal
// path: versioned semantics, inherited contract version, and no empty config
// or schema ceremony.
fabric::resource! {
    CleanKeyValueStore {
        id: "fabric.test.external.clean-key-value";
        version: "0.1.0";

        contracts {
            primary Api {
                id: "fabric.test.external.clean-key-value.api";

                fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, String>;
                fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), String>;
            }
        }

        adapter Adapter {
            id: "fabric.test.external.clean-key-value.adapter";
            compatibility: "^1";

            fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, String>;
            fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), String>;
        }

        runtime {
            fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, String> {
                self.adapter.get(key)
            }

            fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), String> {
                self.adapter.put(key, value)
            }
        }
    }
}

fabric::adapter! {
    CleanMemoryStore
        for resource CleanKeyValueStore
        implements CleanKeyValueStoreRealization
    {
        version: "1.0.0";

        state {
            CleanMemoryState = CleanMemoryState::default();
        }

        runtime {
            fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, String> {
                Ok(self.state.get().values.lock().expect("values").get(&key).cloned())
            }

            fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), String> {
                self.state.get().values.lock().expect("values").insert(key, value);
                Ok(())
            }
        }
    }
}

// A deliberate compatibility override remains available for an implementation
// that supports a range rather than only its compiled target version.
fabric::adapter! {
    ExplicitSupportMemoryStore
        for resource CleanKeyValueStore
        implements CleanKeyValueStoreRealization
    {
        supports: "^0.1";
        version: "1.0.0";

        runtime {
            fn get(&self, _key: Vec<u8>) -> Result<Option<Vec<u8>>, String> {
                Ok(None)
            }

            fn put(&self, _key: Vec<u8>, _value: Vec<u8>) -> Result<(), String> {
                Ok(())
            }
        }
    }
}

// A second declaration proves omitted version and config select the safe,
// provisional and empty defaults.
fabric::resource! {
    ImplicitProvisionalResource {
        id: "fabric.test.external.implicit-provisional";

        contracts {
            primary Api {
                id: "fabric.test.external.implicit-provisional.api";
            }
        }

        runtime {}
    }
}

fabric::resource! {
    RelationVolume {
        id: "fabric.test.external.relation-volume";
        contracts { primary Api { id: "fabric.test.external.relation-volume.api"; fn amount(&self) -> u64; } }
        runtime { fn amount(&self) -> u64 { 7 } }
    }
}

fabric::system! {
    RelationClock {
        id: "fabric.test.external.relation-clock";
        contracts { primary Api { id: "fabric.test.external.relation-clock.api"; fn now(&self) -> u64; } }
        runtime { fn now(&self) -> u64 { 3 } }
    }
}

fabric::resource! {
    RelationResourceProbe {
        id: "fabric.test.external.relation-resource-probe";
        config { offset: u64; }
        relations { requires { volume: RelationVolume; clock: RelationClock; } }
        contracts { primary Api { id: "fabric.test.external.relation-resource-probe.api"; fn total(&self) -> u64; } }
        runtime { fn total(&self) -> u64 { self.volume.amount() + self.clock.now() + self.config().offset } }
        lifecycle { initialize { let _ = self.clock.now(); Ok(()) } }
    }
}

fabric::system! {
    RelationSystemProbe {
        id: "fabric.test.external.relation-system-probe";
        config { offset: u64; }
        relations { requires { volume: RelationVolume; clock: RelationClock; } }
        contracts { primary Api { id: "fabric.test.external.relation-system-probe.api"; fn total(&self) -> u64; } }
        runtime { fn total(&self) -> u64 { self.volume.amount() + self.clock.now() + self.config().offset } }
    }
}

fabric::resource! {
    RelationAdapterTarget {
        id: "fabric.test.external.relation-adapter-target";
        contracts { primary Api { id: "fabric.test.external.relation-adapter-target.api"; fn total(&self) -> u64; } }
        adapter Adapter { id: "fabric.test.external.relation-adapter-target.adapter"; compatibility: provisional; fn total(&self) -> u64; }
        runtime { fn total(&self) -> u64 { self.adapter.total() } }
    }
}

fabric::adapter! {
    RelationAwareAdapter for resource RelationAdapterTarget implements RelationAdapterTargetRealization {
        config { offset: u64; }
        relations { requires { volume: RelationVolume; clock: RelationClock; } }
        runtime { fn total(&self) -> u64 { self.volume.amount() + self.clock.now() + self.config().offset } }
        lifecycle { initialize { let _ = self.volume.amount(); Ok(()) } }
    }
}

struct ExternalStatefulService {
    state: RuntimeState<ExternalLifecycleState>,
}

impl ExternalStatefulResourceRealization for ExternalStatefulService {
    fn current(&self) -> usize {
        self.state.get().value.fetch_add(1, Ordering::SeqCst)
    }
}

fabric::component! {
    EcosystemClockProbe {
        id: "fabric.test.ecosystem.clock-probe";

        config {}

        requires {
            clock: Clock(version = "^1");
        }

        operations {
            read {
                id: "fabric.test.ecosystem.clock-probe.read";
                input: ReadClockInput = "fabric.test.ecosystem.clock-probe.read.input";
                output: ReadClockOutput = "fabric.test.ecosystem.clock-probe.read.output";
                handler |dependencies, input: ReadClockInput| async move {
                    let _ = input;
                    Ok(ReadClockOutput {
                        tick: dependencies.clock.current_tick().map(|tick| tick.value()),
                    })
                };
            }
        }
    }
}

fabric::component! {
    ExternalTeardownComponent {
        id: "fabric.test.external.teardown-component";

        config {
            events: Arc<Mutex<Vec<String>>>;
        }

        operations {
            noop {
                id: "fabric.test.external.teardown-component.noop";
                input: TeardownInput = "fabric.test.external.teardown-component.noop.input";
                output: TeardownOutput = "fabric.test.external.teardown-component.noop.output";
                handler |_input: TeardownInput| async move { Ok(TeardownOutput) };
            }
        }

        teardown {
            config.events.lock().expect("events").push("component-teardown".to_owned());
            Ok(())
        }
    }
}

#[test]
fn external_resource_adapter_and_component_compose_through_the_canonical_sdk_path() {
    let clock_selection = Clock::select("primary", ClockConfig::default()).expect("clock");
    let clock = clock_selection
        .clone()
        .using(MemoryClock::new(41))
        .expect("clock adapter");
    let built = Fabric::new("fabric.test.ecosystem")
        .expect("fabric")
        .component(
            EcosystemClockProbe::define(EcosystemClockProbeConfig {})
                .select_resource_provider(&clock_selection),
        )
        .resource(clock)
        .build()
        .expect("build");

    assert_eq!(built.manifest().resources().len(), 1);
    assert_eq!(
        built.manifest().resources()[0].resource_id().as_str(),
        "fabric.test.clock"
    );
    assert_eq!(built.manifest().resources()[0].name().as_str(), "primary");

    let mut instance = built
        .materialize_named_on("fabric.test.ecosystem.instance", &test_host())
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<EcosystemClockProbe>()
        .expect("materialize component");

    let first = futures::executor::block_on(
        components.invoke_external(&ecosystem_clock_probe::operations::read(), ReadClockInput),
    )
    .expect("first read");
    let second = futures::executor::block_on(
        components.invoke_external(&ecosystem_clock_probe::operations::read(), ReadClockInput),
    )
    .expect("second read");
    assert_eq!(first.tick.expect("first clock tick"), 41);
    assert_eq!(second.tick.expect("second clock tick"), 42);

    components
        .dematerialize::<EcosystemClockProbe>()
        .expect("dematerialize component");
    instance.stop().expect("stop instance");
}

#[test]
fn external_clean_normal_authoring_derives_schema_support_and_provisional_defaults() {
    let key_value = CleanKeyValueStore::select("primary")
        .expect("selection")
        .using(CleanMemoryStore::new())
        .expect("adapter support derives from target schema");
    let implicit =
        ImplicitProvisionalResource::select("implicit").expect("implicit provisional selection");
    let explicit_support = CleanKeyValueStore::select("explicit-support")
        .expect("selection")
        .using(ExplicitSupportMemoryStore::new())
        .expect("explicit support");
    let built = Fabric::new("fabric.test.external.clean-normal-authoring")
        .expect("fabric")
        .resource(key_value)
        .resource(implicit)
        .resource(explicit_support)
        .build()
        .expect("build");
    assert_eq!(built.manifest().resources().len(), 3);
    assert!(matches!(
        ImplicitProvisionalResource::schema().identity(),
        fabric::resource::ResourceSchemaIdentity::Provisional
    ));
    let mut instance = built
        .materialize_named_on(
            "fabric.test.external.clean-normal-authoring.instance",
            &test_host(),
        )
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop");
}

#[test]
fn external_config_authoring_keeps_semantic_and_realization_config_separate() {
    config_events().lock().expect("events").clear();
    let _memory = Backend::Memory;
    let _sqlite = Backend::Sqlite {
        path: PathBuf::from("/tmp/key-value.sqlite"),
    };

    let resource = ConfiguredKeyValue::select("users", ResourceCreatorConfig::new("users"))
        .expect("configured resource")
        .using(LocalKeyValue::new(LocalKeyValueConfig::redis(
            "redis.internal",
            4,
        )))
        .expect("configured adapter");
    let system = ConfiguredSystem::select(SystemCreatorConfig::new("telemetry"))
        .expect("configured system")
        .using(ConfiguredSystemAdapter::new(
            ConfiguredSystemAdapterConfig {
                endpoint: "https://metrics.internal".to_owned(),
            },
        ))
        .expect("configured system adapter");
    let unconfigured_system = UnconfiguredSystem::select()
        .expect("unconfigured system")
        .using(UnconfiguredSystemAdapter::new())
        .expect("unconfigured system adapter");

    let built = Fabric::new("fabric.test.external.universal-config")
        .expect("fabric")
        .resource(resource)
        .system(system)
        .system(unconfigured_system)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_named_on(
            "fabric.test.external.universal-config.instance",
            &test_host(),
        )
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop");

    let events = config_events().lock().expect("events");
    for expected in [
        "resource.initialize:users",
        "resource.start:users",
        "resource.stop:users",
        "adapter.initialize:redis:redis.internal/4",
        "system.initialize:system:telemetry",
        "system-adapter.initialize:https://metrics.internal",
    ] {
        assert!(
            events.iter().any(|event| event == expected),
            "missing {expected}"
        );
    }
}

#[test]
fn no_config_normal_authoring_needs_no_config_value() {
    let resource = CleanKeyValueStore::select("memory")
        .expect("unconfigured resource")
        .using(CleanMemoryStore::new())
        .expect("unconfigured adapter");
    let system = UnconfiguredSystem::select()
        .expect("unconfigured system")
        .using(UnconfiguredSystemAdapter::new())
        .expect("unconfigured system adapter");

    let built = Fabric::new("fabric.test.external.no-config")
        .expect("fabric")
        .resource(resource)
        .system(system)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_named_on("fabric.test.external.no-config.instance", &test_host())
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop");
}

#[test]
fn external_relations_bind_resource_system_and_adapter_dependencies() {
    let volume = RelationVolume::select("primary").expect("volume");
    let clock = RelationClock::select().expect("clock");
    let resource =
        RelationResourceProbe::select("resource", RelationResourceProbeConfig { offset: 1 })
            .expect("resource probe");
    let system =
        RelationSystemProbe::select(RelationSystemProbeConfig { offset: 2 }).expect("system probe");
    let adapted = RelationAdapterTarget::select("adapter")
        .expect("adapter target")
        .using(RelationAwareAdapter::new(RelationAwareAdapterConfig {
            offset: 3,
        }))
        .expect("adapter relation target");

    let built = Fabric::new("fabric.test.external.relations")
        .expect("fabric")
        .resource(volume)
        .resource(resource)
        .resource(adapted)
        .system(clock)
        .system(system)
        .build()
        .expect("relations resolve");
    let mut instance = built
        .materialize_named_on("fabric.test.external.relations.instance", &test_host())
        .expect("relations bind");
    instance.start().expect("relations start");
    instance.stop().expect("relations stop");
}

#[test]
fn external_component_macro_owns_participation_local_teardown_without_native_runtime_api() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let built = Fabric::new("fabric.test.external.component-teardown")
        .expect("fabric")
        .component(ExternalTeardownComponent::define(
            ExternalTeardownComponentConfig {
                events: Arc::clone(&events),
            },
        ))
        .build()
        .expect("build");
    let mut instance = built
        .materialize_named_on(
            "fabric.test.external.component-teardown.instance",
            &test_host(),
        )
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<ExternalTeardownComponent>()
        .expect("materialize component");
    components
        .dematerialize::<ExternalTeardownComponent>()
        .expect("dematerialize component");
    assert_eq!(
        events.lock().expect("events").as_slice(),
        ["component-teardown"]
    );
    instance.stop().expect("stop");
}

#[test]
fn external_sdk_stateful_adapter_uses_public_runtime_authoring_without_module_runtime() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let runtime = StatefulRuntimeAuthoring::new(
        {
            let events = Arc::clone(&events);
            move || ExternalLifecycleState {
                events: Arc::clone(&events),
                value: AtomicUsize::new(0),
            }
        },
        |state| {
            external_stateful_resource::realization::raw::AdapterContract::new(Arc::new(
                ExternalStatefulService { state },
            ))
        },
    )
    .with_initialize(|state, context| {
        assert!(context.is_bound());
        state.get().event("initialize");
        Ok(())
    })
    .with_start(|state, _| {
        state.get().event("start");
        Ok(())
    })
    .with_stop(|state, _| {
        state.get().event("stop");
        Ok(())
    })
    .with_health(|_, _| Health::Healthy);
    let adapter = StatefulAdapterDefinition::<
        ExternalStatefulResource,
        AdapterResourceSchemaSupport,
        ExternalLifecycleState,
        external_stateful_resource::realization::raw::AdapterContract,
    >::new(
        AdapterResourceSchemaSupport::provisional(ExternalStatefulResource::resource_id()),
        HostRequirement::new(),
        external_stateful_resource::realization::raw::provisional_contract_key(),
        runtime,
    );
    let resource = ExternalStatefulResource::select("primary", ExternalStatefulResourceConfig {})
        .expect("selection")
        .using(adapter)
        .expect("adapter");
    let built = Fabric::new("fabric.test.external.stateful-runtime")
        .expect("fabric")
        .resource(resource)
        .build()
        .expect("build");
    let mut instance = built
        .composition()
        .materialize_named_on(
            "fabric.test.external.stateful-runtime.instance",
            &test_host(),
        )
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop");
    assert_eq!(
        events.lock().expect("events").as_slice(),
        ["initialize", "start", "stop"]
    );
}

#[test]
fn stateful_external_witness_does_not_author_raw_module_runtime() {
    let source = include_str!("witness.rs");
    let witness = source
        .split(
            "fn external_sdk_stateful_adapter_uses_public_runtime_authoring_without_module_runtime",
        )
        .nth(1)
        .expect("stateful witness")
        .split("#[test]")
        .next()
        .expect("witness end");
    assert!(witness.contains("StatefulRuntimeAuthoring"));
    assert!(!witness.contains("ModuleRuntime"));
}
