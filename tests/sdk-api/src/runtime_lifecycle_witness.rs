use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use fabric::authoring::{CompositionExt, FabricBuilder};
use fabric::core::{CompositionExport, ContractId, ModuleError};
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

        schema: provisional;

        config {
            fixture: FixtureConfig;
        }

        contracts {
            primary Api {
                id: "fabric.test.lifecycle.stateful-resource.api";
                version: provisional;

                fn increment(&self) -> usize;
                fn mark_degraded(&self);
                fn mark_unavailable(&self);
            }
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

        schema: provisional;

        config {
            fixture: FixtureConfig;
        }

        contracts {
            primary Api {
                id: "fabric.test.lifecycle.stateful-system.api";
                version: provisional;

                fn current(&self) -> usize;
            }
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

        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.lifecycle.adapted-resource.api";
                version: provisional;

                fn current(&self) -> usize;
            }
        }

        adapter Adapter {
            id: "fabric.test.lifecycle.adapted-resource.adapter";
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

fn adapter_host_facility() -> HostFacilityId {
    HostFacilityId::new("fabric.test.lifecycle.adapter-state").expect("static host facility")
}

fabric::adapter! {
    StatefulFixtureResourceAdapter for resource AdaptedFixtureResource implements AdaptedFixtureResourceRealization {
        schema: provisional;
        realization: provisional;

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

        schema: provisional;

        config {}

        contracts {
            primary Api {
                id: "fabric.test.lifecycle.adapted-system.api";
                version: provisional;

                fn current(&self) -> usize;
            }
        }

        adapter Adapter {
            id: "fabric.test.lifecycle.adapted-system.adapter";
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

fabric::adapter! {
    StatefulFixtureSystemAdapter for system AdaptedFixtureSystem implements AdaptedFixtureSystemRealization {
        schema: provisional;
        realization: provisional;

        config {
            fixture: FixtureConfig;
        }

        system {
            prerequisite: StatefulFixtureSystem(provisional);
        }

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

fn adapted_resource_export() -> CompositionExport<adapted_fixture_resource::raw::ApiContract> {
    CompositionExport::new(
        ContractId::new("fabric.test.lifecycle.adapted-resource.export").expect("export id"),
        Requires::<AdaptedFixtureResource>::provisional()
            .as_contract_requirement()
            .clone(),
    )
}

fn adapted_system_export() -> CompositionExport<adapted_fixture_system::raw::ApiContract> {
    CompositionExport::new(
        ContractId::new("fabric.test.lifecycle.adapted-system.export").expect("export id"),
        SystemRequires::<AdaptedFixtureSystem>::provisional()
            .as_contract_requirement()
            .clone(),
    )
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
        .materialize_named_on("fabric.test.lifecycle.resource.first", &host())
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
        .materialize_named_on("fabric.test.lifecycle.resource.second", &host())
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
        .materialize_named_on("fabric.test.lifecycle.ready-stop.instance", &host())
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
        .materialize_named_on("fabric.test.lifecycle.stop-failure.instance", &host())
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
        .materialize_named_on(
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
        .materialize_named_on("fabric.test.lifecycle.system.instance", &host())
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
    let (resource, adapter, selection) = adapted.into_raw_parts();
    let export = adapted_resource_export();
    let composition = FabricBuilder::new("fabric.test.lifecycle.resource-adapter")
        .expect("builder")
        .block("runtime", |block| block.module(resource).module(adapter))
        .expect("block")
        .select_provider(selection)
        .export(export.clone())
        .build()
        .expect("composition");
    let mut instance = composition
        .materialize_named_on(
            "fabric.test.lifecycle.resource-adapter.instance",
            &host().with_facility(adapter_host_facility()),
        )
        .expect("materialize");
    instance.start().expect("start");
    let service = instance.export(&export).expect("export");
    assert_eq!(service.current(), 0);
    assert_eq!(service.current(), 1);
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
    let (system, adapter, selection) = adapted.into_raw_parts();
    let export = adapted_system_export();
    let composition = FabricBuilder::new("fabric.test.lifecycle.system-adapter")
        .expect("builder")
        .block("runtime", |block| {
            block.module(prerequisite).module(system).module(adapter)
        })
        .expect("block")
        .select_provider(selection)
        .export(export.clone())
        .build()
        .expect("composition");
    let mut instance = composition
        .materialize_named_on("fabric.test.lifecycle.system-adapter.instance", &host())
        .expect("materialize");
    instance.start().expect("start");
    let service = instance.export(&export).expect("export");
    assert_eq!(service.current(), 0);
    assert_eq!(service.current(), 1);
    instance.stop().expect("stop");
    assert!(
        events
            .lock()
            .expect("events")
            .iter()
            .any(|event| event == "system-adapter.stop")
    );
}
