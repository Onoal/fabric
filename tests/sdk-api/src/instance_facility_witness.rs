use std::sync::{Arc, Mutex};

use fabric::prelude::*;
use fabric_test_component_greeter::{Greeter, GreeterConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
enum FacilityEvent {
    Attached(LifecycleState),
    Starting(LifecycleState),
    Started(LifecycleState),
    Stopping(LifecycleState),
    Stopped(LifecycleState),
    Detached(LifecycleState),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FacilityRecord {
    name: String,
    instance_id: String,
    generation: InstanceGeneration,
    profile: String,
    event: FacilityEvent,
}

#[derive(Clone)]
struct RecordingInstanceFacility {
    name: InstanceFacilityName,
    records: Arc<Mutex<Vec<FacilityRecord>>>,
}

impl RecordingInstanceFacility {
    fn new(name: &str, records: Arc<Mutex<Vec<FacilityRecord>>>) -> Self {
        Self {
            name: InstanceFacilityName::new(name).expect("facility name"),
            records,
        }
    }

    fn record(&self, context: &InstanceFacilityContext, event: FacilityEvent) {
        self.records
            .lock()
            .expect("facility records")
            .push(FacilityRecord {
                name: self.name.as_str().to_owned(),
                instance_id: context.instance_id().as_str().to_owned(),
                generation: context.generation(),
                profile: context.materialization_profile().name().as_str().to_owned(),
                event,
            });
    }
}

impl InstanceFacility for RecordingInstanceFacility {
    fn name(&self) -> &InstanceFacilityName {
        &self.name
    }

    fn attached(&mut self, context: &InstanceFacilityContext) {
        self.record(context, FacilityEvent::Attached(context.lifecycle()));
    }

    fn starting(&mut self, context: &InstanceFacilityContext) {
        self.record(context, FacilityEvent::Starting(context.lifecycle()));
    }

    fn started(&mut self, context: &InstanceFacilityContext) {
        self.record(context, FacilityEvent::Started(context.lifecycle()));
    }

    fn stopping(&mut self, context: &InstanceFacilityContext) {
        self.record(context, FacilityEvent::Stopping(context.lifecycle()));
    }

    fn stopped(&mut self, context: &InstanceFacilityContext) {
        self.record(context, FacilityEvent::Stopped(context.lifecycle()));
    }

    fn detached(&mut self, context: &InstanceFacilityContext) {
        self.record(context, FacilityEvent::Detached(context.lifecycle()));
    }
}

fn facility_composition(id: &str) -> Composition {
    Fabric::new(id)
        .expect("fabric")
        .component(Greeter::define(GreeterConfig {}))
        .build()
        .expect("composition")
}

fn facility_names(instance: &Instance) -> Vec<String> {
    instance
        .observe()
        .facilities()
        .iter()
        .map(|facility| facility.name().as_str().to_owned())
        .collect()
}

#[test]
fn ready_attachment_observes_truthful_lifecycle_without_semantic_changes() {
    let composition = facility_composition("fabric.test.facility.ready");
    let plan = composition.plan().expect("plan");
    let before_components = composition
        .components()
        .map(|component| component.component_id().clone())
        .collect::<Vec<_>>();
    let before_plan = plan.provenance().clone();
    let mut instance = plan
        .materialize("fabric.test.facility.ready.instance")
        .expect("instance");
    let instance_id = instance.instance_id().clone();
    let generation = instance.generation();
    let health = instance.observe().health();
    let records = Arc::new(Mutex::new(Vec::new()));

    instance
        .attach_facility(RecordingInstanceFacility::new(
            "diagnostics",
            Arc::clone(&records),
        ))
        .expect("attach ready");

    assert_eq!(facility_names(&instance), vec!["diagnostics"]);
    assert_eq!(instance.instance_id(), &instance_id);
    assert_eq!(instance.generation(), generation);
    assert_eq!(instance.materialization_plan(), &before_plan);
    assert_eq!(instance.observe().health(), health);
    assert_eq!(
        composition
            .components()
            .map(|component| component.component_id().clone())
            .collect::<Vec<_>>(),
        before_components
    );

    instance.start().expect("start");
    instance.stop().expect("stop");

    let records = records.lock().expect("records").clone();
    assert_eq!(
        records
            .iter()
            .map(|record| &record.event)
            .collect::<Vec<_>>(),
        vec![
            &FacilityEvent::Attached(LifecycleState::Ready),
            &FacilityEvent::Starting(LifecycleState::Ready),
            &FacilityEvent::Started(LifecycleState::Running),
            &FacilityEvent::Stopping(LifecycleState::Running),
            &FacilityEvent::Stopped(LifecycleState::Stopped),
        ]
    );
    assert!(records.iter().all(|record| record.generation == generation));
    assert!(records.iter().all(|record| record.profile == "default"));
}

#[test]
fn running_attachment_reports_current_state_without_fabricating_history() {
    let mut instance = facility_composition("fabric.test.facility.running")
        .materialize("fabric.test.facility.running.instance")
        .expect("instance");
    let records = Arc::new(Mutex::new(Vec::new()));

    instance.start().expect("start before attach");
    instance
        .attach_facility(RecordingInstanceFacility::new(
            "profiler",
            Arc::clone(&records),
        ))
        .expect("attach running");
    instance.stop().expect("stop");

    let events = records
        .lock()
        .expect("records")
        .iter()
        .map(|record| record.event.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        events,
        vec![
            FacilityEvent::Attached(LifecycleState::Running),
            FacilityEvent::Stopping(LifecycleState::Running),
            FacilityEvent::Stopped(LifecycleState::Stopped),
        ]
    );
}

#[test]
fn stopped_attachment_is_rejected_for_terminal_generation() {
    let mut instance = facility_composition("fabric.test.facility.stopped")
        .materialize("fabric.test.facility.stopped.instance")
        .expect("instance");
    let generation = instance.generation();
    let records = Arc::new(Mutex::new(Vec::new()));

    instance.start().expect("start");
    instance.stop().expect("stop");
    let error = instance
        .attach_facility(RecordingInstanceFacility::new("late", Arc::clone(&records)))
        .expect_err("stopped attachment");

    assert!(matches!(
        error,
        InstanceFacilityError::InstanceStopped { name }
            if name.as_str() == "late"
    ));
    assert_eq!(instance.generation(), generation);
    assert!(records.lock().expect("records").is_empty());
}

#[test]
fn duplicate_identity_is_rejected_and_detach_is_operational_only() {
    let mut instance = facility_composition("fabric.test.facility.duplicate")
        .materialize("fabric.test.facility.duplicate.instance")
        .expect("instance");
    let generation = instance.generation();
    let plan = instance.materialization_plan().clone();
    let records = Arc::new(Mutex::new(Vec::new()));
    let name = InstanceFacilityName::new("diagnostics").expect("name");

    instance
        .attach_facility(RecordingInstanceFacility::new(
            "diagnostics",
            Arc::clone(&records),
        ))
        .expect("attach first");
    let duplicate = instance
        .attach_facility(RecordingInstanceFacility::new(
            "diagnostics",
            Arc::clone(&records),
        ))
        .expect_err("duplicate");
    assert!(matches!(
        duplicate,
        InstanceFacilityError::DuplicateFacility { name } if name.as_str() == "diagnostics"
    ));

    instance.detach_facility(&name).expect("detach");
    assert!(facility_names(&instance).is_empty());
    assert_eq!(instance.generation(), generation);
    assert_eq!(instance.materialization_plan(), &plan);
    assert_eq!(
        records
            .lock()
            .expect("records")
            .iter()
            .map(|record| record.event.clone())
            .collect::<Vec<_>>(),
        vec![
            FacilityEvent::Attached(LifecycleState::Ready),
            FacilityEvent::Detached(LifecycleState::Ready),
        ]
    );
}

#[test]
fn multiple_facilities_have_deterministic_order_and_private_state() {
    let mut instance = facility_composition("fabric.test.facility.multiple")
        .materialize("fabric.test.facility.multiple.instance")
        .expect("instance");
    let first = Arc::new(Mutex::new(Vec::new()));
    let second = Arc::new(Mutex::new(Vec::new()));

    instance
        .attach_facility(RecordingInstanceFacility::new("alpha", Arc::clone(&first)))
        .expect("attach alpha");
    instance
        .attach_facility(RecordingInstanceFacility::new("beta", Arc::clone(&second)))
        .expect("attach beta");
    instance.start().expect("start");
    instance.stop().expect("stop");

    assert_eq!(facility_names(&instance), vec!["alpha", "beta"]);
    assert_eq!(first.lock().expect("first").len(), 5);
    assert_eq!(second.lock().expect("second").len(), 5);
    assert_eq!(
        instance
            .observe()
            .facilities()
            .iter()
            .map(|facility| facility.name().as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "beta"]
    );
}

#[test]
fn same_plan_can_materialize_distinct_operational_shells() {
    let composition = facility_composition("fabric.test.facility.same-plan");
    let profile = MaterializationProfile::new("diagnostic").expect("profile");
    let plan = composition
        .plan_with_profile(&profile)
        .expect("diagnostic plan");
    let plan_before = plan.provenance().clone();
    let composition_components = composition
        .components()
        .map(|component| component.component_id().clone())
        .collect::<Vec<_>>();
    let first_records = Arc::new(Mutex::new(Vec::new()));
    let second_records = Arc::new(Mutex::new(Vec::<FacilityRecord>::new()));

    let mut first = plan
        .materialize("fabric.test.facility.same-plan.a")
        .expect("first");
    let mut second = plan
        .materialize("fabric.test.facility.same-plan.b")
        .expect("second");
    first
        .attach_facility(RecordingInstanceFacility::new(
            "diagnostics",
            Arc::clone(&first_records),
        ))
        .expect("attach first only");

    first.start().expect("start first");
    second.start().expect("start second");
    first.stop().expect("stop first");
    second.stop().expect("stop second");

    assert_eq!(plan.provenance(), &plan_before);
    assert_eq!(first.materialization_plan(), &plan_before);
    assert_eq!(second.materialization_plan(), &plan_before);
    assert_ne!(first.instance_id(), second.instance_id());
    assert_ne!(first.generation(), second.generation());
    assert_eq!(facility_names(&first), vec!["diagnostics"]);
    assert!(facility_names(&second).is_empty());
    assert_eq!(first_records.lock().expect("first records").len(), 5);
    assert!(second_records.lock().expect("second records").is_empty());
    assert_eq!(
        composition
            .components()
            .map(|component| component.component_id().clone())
            .collect::<Vec<_>>(),
        composition_components
    );
    assert_eq!(composition.resources().count(), 0);
    assert_eq!(composition.systems().count(), 0);
}

#[test]
fn facility_names_are_operational_not_semantic_identifiers() {
    assert!(InstanceFacilityName::new("diagnostics").is_ok());
    assert!(InstanceFacilityName::new("").is_err());
    assert!(InstanceFacilityName::new(" system ").is_err());
    assert!(InstanceFacilityName::new("component/name").is_err());
}
