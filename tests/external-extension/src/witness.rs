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
