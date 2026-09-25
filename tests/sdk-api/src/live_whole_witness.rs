use fabric::prelude::*;
use futures::executor::block_on;

fabric::resource! {
    LiveStore {
        id: "fabric.test.live.store";
        config { value: u64; }
        api { fn get(&self) -> u64; }
        runtime { fn get(&self) -> u64 { self.config.value } }
    }
}

fabric::system! {
    LiveClock {
        id: "fabric.test.live.clock";
        config { tick: u64; }
        api { fn now(&self) -> u64; }
        runtime { fn now(&self) -> u64 { self.config.tick } }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LiveInput {
    name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LiveOutput {
    message: String,
}

fabric::component! {
    LiveApp {
        id: "fabric.test.live.app";
        api { fn greet(&self, input: LiveInput) -> LiveOutput; }
        runtime {
            prepare { Ok(()) }
            fn greet(&self, input: LiveInput) -> LiveOutput {
                LiveOutput { message: format!("hello, {}", input.name) }
            }
        }
    }
}

fabric::component! {
    DeclarationOnlyApp {
        id: "fabric.test.live.declaration-only";
        api { fn ping(&self); }
    }
}

fn live_composition(id: &str) -> Composition {
    Fabric::new(id)
        .expect("fabric")
        .resource(LiveStore::select("primary", LiveStoreConfig { value: 7 }).expect("store"))
        .system(LiveClock::select(LiveClockConfig { tick: 42 }).expect("clock"))
        .component(LiveApp::define())
        .build()
        .expect("composition")
}

#[test]
fn instance_observation_projects_declared_semantics_and_live_runtime_state() {
    let composition = live_composition("fabric.test.live.whole");
    let mut instance = composition
        .materialize("fabric.test.live.whole.instance")
        .expect("instance");

    let ready = instance.observe();
    assert_eq!(ready.lifecycle(), LifecycleState::Ready);
    assert_eq!(ready.health(), Health::Unavailable);
    assert_eq!(ready.resources().len(), 1);
    assert_eq!(ready.systems().len(), 1);
    assert_eq!(ready.components().len(), 1);
    assert_eq!(
        ready.resources()[0].realization().kind(),
        SemanticRealizationKind::SelfRealization
    );
    assert_eq!(
        ready.systems()[0].realization().kind(),
        SemanticRealizationKind::SelfRealization
    );
    assert_eq!(ready.resources()[0].realization().local().len(), 1);
    assert_eq!(ready.systems()[0].realization().local().len(), 1);
    assert_eq!(
        ready.components()[0].desired_participation(),
        ComponentDesiredState::Enabled
    );
    assert_eq!(
        ready.components()[0].observed_participation(),
        ComponentObservedParticipation::Absent
    );

    instance.start().expect("start");
    let after_start = instance.observe();
    assert_eq!(after_start.lifecycle(), LifecycleState::Running);
    assert_eq!(after_start.health(), Health::Healthy);
    assert_eq!(
        after_start.components()[0].observed_participation(),
        ComponentObservedParticipation::Absent,
        "start() does not reconcile Component participation"
    );

    let app = instance.component::<LiveApp>().expect("bound app");
    assert!(
        block_on(app.greet(LiveInput {
            name: "before".to_owned()
        }))
        .is_err(),
        "declaration lookup succeeds before participation, but invocation is unavailable"
    );

    let outcome = app.reconcile().expect("reconcile app");
    assert_eq!(outcome.desired(), ComponentDesiredState::Enabled);
    assert_eq!(
        outcome.observed_before(),
        ComponentObservedParticipation::Absent
    );
    assert!(matches!(
        outcome.result(),
        ComponentReconciliationResult::Materialized(ComponentObservedParticipation::Active)
    ));

    let live = instance.observe();
    assert_eq!(
        live.components()[0].observed_participation(),
        ComponentObservedParticipation::Active
    );
    assert_eq!(live.components()[0].health(), Some(Health::Healthy));
    let response = block_on(app.greet(LiveInput {
        name: "Fabric".to_owned(),
    }))
    .expect("typed invocation");
    assert_eq!(response.message, "hello, Fabric");

    app.disable().expect("disable desired state");
    let divergent = app.observe();
    assert_eq!(
        divergent.desired_participation(),
        ComponentDesiredState::Disabled
    );
    assert_eq!(
        divergent.observed_participation(),
        ComponentObservedParticipation::Active
    );
    let disabled = app.reconcile().expect("disable reconcile");
    assert!(matches!(
        disabled.result(),
        ComponentReconciliationResult::Dematerialized(_)
    ));
    assert_eq!(
        app.observe().observed_participation(),
        ComponentObservedParticipation::Absent
    );

    instance.stop().expect("stop");
    assert_eq!(instance.observe().lifecycle(), LifecycleState::Stopped);
}

#[test]
fn component_participation_is_instance_scoped() {
    let composition = live_composition("fabric.test.live.multi-instance");
    let mut first = composition
        .materialize("fabric.test.live.multi-instance.first")
        .expect("first");
    let second = composition
        .materialize("fabric.test.live.multi-instance.second")
        .expect("second");

    first.start().expect("start first");
    let first_app = first.component::<LiveApp>().expect("first app");
    first_app.reconcile().expect("first reconcile");

    assert_eq!(
        first.observe().components()[0].observed_participation(),
        ComponentObservedParticipation::Active
    );
    assert_eq!(
        second.observe().components()[0].observed_participation(),
        ComponentObservedParticipation::Absent
    );
    assert_ne!(first_app.generation(), second.generation());
}

#[test]
fn declaration_only_component_is_observable_but_not_invocable() {
    let composition = Fabric::new("fabric.test.live.declaration-only")
        .expect("fabric")
        .component(DeclarationOnlyApp::define())
        .build()
        .expect("composition");
    let instance = composition
        .materialize("fabric.test.live.declaration-only.instance")
        .expect("instance");

    let observation = instance.observe();
    assert_eq!(
        observation.components()[0].initial_participation(),
        ComponentDesiredState::Disabled
    );
    assert_eq!(
        observation.components()[0].observed_participation(),
        ComponentObservedParticipation::Absent
    );
    let declaration = instance
        .component::<DeclarationOnlyApp>()
        .expect("declaration handle");
    assert!(declaration.reconcile().is_err());
}

#[test]
fn resource_system_only_instances_have_semantic_live_observation_without_component_host() {
    let composition = Fabric::new("fabric.test.live.resource-system-only")
        .expect("fabric")
        .resource(LiveStore::select("primary", LiveStoreConfig { value: 1 }).expect("store"))
        .system(LiveClock::select(LiveClockConfig { tick: 2 }).expect("clock"))
        .build()
        .expect("composition");
    let instance = composition
        .materialize("fabric.test.live.resource-system-only.instance")
        .expect("instance");
    let observation = instance.observe();

    assert!(instance.components().is_none());
    assert_eq!(observation.resources().len(), 1);
    assert_eq!(observation.systems().len(), 1);
    assert!(observation.components().is_empty());
}
