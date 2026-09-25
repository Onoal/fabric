use fabric::prelude::*;
use fabric_test_adapter_clock_memory::{HostBoundClockAdapter, host_bound_clock_facility};
use fabric_test_component_greeter::{Greeter, GreeterConfig};
use fabric_test_resource_clock::{Clock, ClockConfig};

fabric::resource! {
    LocalStore {
        id: "fabric.test.instance.local-store";
        config { label: String; }
        api {
            fn label(&self) -> String;
        }
        runtime {
            fn label(&self) -> String {
                self.config.label.clone()
            }
        }
    }
}

fn component_composition(id: &str) -> Composition {
    Fabric::new(id)
        .expect("fabric")
        .component(Greeter::define(GreeterConfig {}))
        .build()
        .expect("composition")
}

#[test]
fn root_instance_is_high_level_and_core_instance_remains_raw() {
    fn accepts_sdk(_: Option<fabric::Instance>) {}
    fn accepts_core(_: Option<fabric::core::Instance>) {}

    accepts_sdk(None);
    accepts_core(None);

    let composition = component_composition("fabric.test.instance.type");
    let instance: fabric::Instance = composition
        .materialize("fabric.test.instance.type.local")
        .expect("instance");
    assert_eq!(instance.composition_id(), composition.id());
    assert!(instance.components().is_some());
}

#[test]
fn canonical_materialization_lifecycle_and_terminal_generation_hold() {
    let composition = component_composition("fabric.test.instance.lifecycle");
    let mut instance = composition
        .materialize("fabric.test.instance.lifecycle.local")
        .expect("instance");

    assert_eq!(instance.lifecycle(), LifecycleState::Ready);
    instance.start().expect("start");
    assert_eq!(instance.lifecycle(), LifecycleState::Running);
    instance.stop().expect("stop");
    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
    assert!(
        instance.start().is_err(),
        "stopped generations are terminal"
    );
}

#[test]
fn composition_reuse_and_rematerialization_mint_fresh_generations() {
    let composition = component_composition("fabric.test.instance.reuse");
    let mut first = composition
        .materialize("fabric.test.instance.reuse.same")
        .expect("first");
    let first_generation = first.generation();
    first.stop().expect("stop first");

    let second = composition
        .materialize("fabric.test.instance.reuse.same")
        .expect("second");
    let third = composition
        .materialize("fabric.test.instance.reuse.other")
        .expect("third");

    assert_eq!(first.instance_id(), second.instance_id());
    assert_ne!(first_generation, second.generation());
    assert_ne!(second.instance_id(), third.instance_id());
    assert_ne!(second.generation(), third.generation());
}

#[test]
fn instance_operates_after_source_composition_is_dropped() {
    let mut instance = {
        let composition = component_composition("fabric.test.instance.drop");
        composition
            .materialize("fabric.test.instance.drop.local")
            .expect("instance")
    };

    assert_eq!(instance.lifecycle(), LifecycleState::Ready);
    instance.start().expect("start after composition drop");
    instance.stop().expect("stop after composition drop");
}

#[test]
fn contribution_authored_composition_materializes_to_the_same_facade() {
    fn greeter_contribution() -> impl IntoFabricContribution {
        FabricContribution::new().component(Greeter::define(GreeterConfig {}))
    }

    let composition = Fabric::new("fabric.test.instance.contribution")
        .expect("fabric")
        .with(greeter_contribution())
        .build()
        .expect("composition");
    let mut instance = composition
        .materialize("fabric.test.instance.contribution.local")
        .expect("instance");

    assert!(instance.components().is_some());
    instance.start().expect("start");
    instance.stop().expect("stop");
}

#[test]
fn resource_only_instance_has_no_component_surface() {
    let store = LocalStore::select(
        "primary",
        LocalStoreConfig {
            label: "primary".to_owned(),
        },
    )
    .expect("store");
    let composition = Fabric::new("fabric.test.instance.resource-only")
        .expect("fabric")
        .resource(store)
        .build()
        .expect("composition");
    let instance = composition
        .materialize("fabric.test.instance.resource-only.local")
        .expect("instance");

    assert!(instance.components().is_none());
}

#[test]
fn host_aware_materialization_keeps_existing_validation() {
    let clock = Clock::select("primary", ClockConfig::default())
        .expect("clock")
        .using(HostBoundClockAdapter::new(9))
        .expect("adapter");
    let composition = Fabric::new("fabric.test.instance.host")
        .expect("fabric")
        .resource(clock)
        .build()
        .expect("composition");

    assert!(
        composition
            .materialize("fabric.test.instance.host.no-host")
            .is_err()
    );
    assert!(
        composition
            .materialize_on(
                "fabric.test.instance.host.missing",
                &HostDescriptor::native()
            )
            .is_err()
    );

    let host = HostDescriptor::native().with_facility(host_bound_clock_facility());
    let mut instance = composition
        .materialize_on("fabric.test.instance.host.ok", &host)
        .expect("host-compatible instance");
    instance.start().expect("start");
    instance.stop().expect("stop");
}
