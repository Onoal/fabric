// Keep the Instance guide's normal import form in this compile fixture.
#[allow(unused_imports)]
use fabric::prelude::*;

fabric::component! {
    pub Greeter {
        id: "example.instance.greeter";
        runtime { prepare { Ok(()) } }
    }
}

fabric::resource! {
    pub NoteStore {
        id: "example.instance.note-store";
        version: "0.1.0";
        config { label: String; }
        api {
            fn label(&self) -> String;
        }
        runtime {
            fn label(&self) -> String { self.config.label.clone() }
        }
    }
}

#[cfg(test)]
fn component_fabric() -> Fabric {
    Fabric::new("example.instance")
        .expect("valid CompositionId")
        .component(Greeter::define())
}

#[test]
fn materialization_lifecycle_report_and_component_boundary_are_distinct() {
    let composition = component_fabric().build().expect("valid Composition");
    let mut instance = composition
        .materialize_named("example.instance.local")
        .expect("materialize");

    assert_eq!(instance.lifecycle(), LifecycleState::Ready);
    assert!(instance.components().is_some());
    let ready_report = instance.report();
    assert_eq!(ready_report.composition_id, composition.id().clone());
    assert_eq!(ready_report.instance_id, instance.instance_id().clone());
    assert_eq!(ready_report.generation, instance.generation());
    assert_eq!(ready_report.lifecycle, LifecycleState::Ready);

    instance.start().expect("start");
    assert_eq!(instance.lifecycle(), LifecycleState::Running);
    instance
        .components()
        .expect("ComponentInstanceBinding host")
        .materialize::<Greeter>()
        .expect("materialize ComponentInstanceBinding after Instance start");
    instance.stop().expect("stop instance");
    assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
    assert!(
        instance.start().is_err(),
        "Stopped Instances do not restart"
    );
}

#[test]
fn one_composition_creates_independent_instances_and_fresh_generations() {
    let composition = component_fabric().build().expect("valid Composition");
    let first = composition
        .materialize_named("example.instance.first")
        .expect("first Instance");
    let second = composition
        .materialize_named("example.instance.second")
        .expect("second Instance");

    assert_ne!(first.instance_id(), second.instance_id());
    assert_ne!(first.generation(), second.generation());
}

#[test]
fn rematerializing_the_same_instance_id_mints_a_fresh_generation() {
    let composition = component_fabric().build().expect("valid Composition");
    let first = composition
        .materialize_named("example.instance.local")
        .expect("first Instance");
    let second = composition
        .materialize_named("example.instance.local")
        .expect("second Instance");

    assert_eq!(first.instance_id(), second.instance_id());
    assert_ne!(first.generation(), second.generation());
}

#[test]
fn resource_only_instance_has_no_component_surface() {
    let store = NoteStore::select(
        "primary",
        NoteStoreConfig {
            label: "primary".to_owned(),
        },
    )
    .expect("valid ResourceName");
    let composition = Fabric::new("example.instance.resource-only")
        .expect("valid CompositionId")
        .resource(store)
        .build()
        .expect("valid Composition");
    let instance = composition
        .materialize_named("example.instance.resource-only.local")
        .expect("materialize");

    assert!(instance.components().is_none());
}
