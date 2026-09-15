use fabric_sdk::prelude::*;
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

fabric_sdk::component! {
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
    instance.stop();
}
