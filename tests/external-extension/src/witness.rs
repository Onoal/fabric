use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
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
    let key_value = CleanKeyValueStore::select("primary", CleanKeyValueStoreConfig {})
        .expect("selection")
        .using(CleanMemoryStore::new(CleanMemoryStoreConfig {}))
        .expect("adapter support derives from target schema");
    let implicit =
        ImplicitProvisionalResource::select("implicit", ImplicitProvisionalResourceConfig {})
            .expect("implicit provisional selection");
    let explicit_support =
        CleanKeyValueStore::select("explicit-support", CleanKeyValueStoreConfig {})
            .expect("selection")
            .using(ExplicitSupportMemoryStore::new(
                ExplicitSupportMemoryStoreConfig {},
            ))
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
