use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use fabric::authoring::ComponentDefinition;
use fabric::prelude::*;
use fabric_core::{
    CompositionError, Health, Module, ModuleBindings, ModuleContract, ModuleDeclaration,
    ModuleError, ModuleId, ModuleRuntime,
};
use fabric_test_adapter_clock_memory::{HostBoundClockAdapter, host_bound_clock_facility};
use fabric_test_component_greeter::{Greeter, GreeterConfig};
use fabric_test_resource_clock::{Clock, ClockConfig};

fn component_composition(id: &str) -> Composition {
    Fabric::new(id)
        .expect("fabric")
        .component(Greeter::define(GreeterConfig {}).initially_disabled())
        .build()
        .expect("composition")
}

#[test]
fn plan_is_inspectable_non_live_effective_truth_before_instance_exists() {
    let composition = component_composition("fabric.test.plan.inspect");
    let profile = MaterializationProfile::new("diagnostic").expect("profile");
    let host = HostDescriptor::native()
        .with_facility(HostFacilityId::new("fabric.test.plan.host").expect("facility"));

    let plan = composition
        .plan_with_profile_on(&profile, &host)
        .expect("plan");

    assert_eq!(plan.composition_id(), composition.id());
    assert_eq!(plan.materialization_profile(), &profile);
    assert_eq!(plan.host(), Some(&host));
    assert_eq!(plan.resources().count(), composition.resources().count());
    assert_eq!(plan.systems().count(), composition.systems().count());
    assert_eq!(plan.relations(), composition.relations());

    let planned_components = plan.components().collect::<Vec<_>>();
    assert_eq!(planned_components.len(), 1);
    assert_eq!(
        planned_components[0].component_id(),
        &Greeter::component_id()
    );
    assert_eq!(
        planned_components[0].initial_participation(),
        ComponentDesiredState::Disabled
    );
}

#[test]
fn host_validation_happens_before_a_valid_plan_is_returned() {
    let adapted = Clock::select("primary", ClockConfig::default())
        .expect("clock")
        .using(HostBoundClockAdapter::new(17))
        .expect("host-bound adapter");

    let composition = Fabric::new("fabric.test.plan.host-validation")
        .expect("fabric")
        .resource(adapted)
        .build()
        .expect("composition");

    assert!(matches!(
        composition.plan(),
        Err(CompositionError::HostDescriptorRequired { .. })
    ));
    assert!(matches!(
        composition.plan_on(&HostDescriptor::native()),
        Err(CompositionError::HostIncompatible { .. })
    ));

    let host = HostDescriptor::native().with_facility(host_bound_clock_facility());
    let plan = composition.plan_on(&host).expect("host-compatible plan");
    assert_eq!(plan.host(), Some(&host));
    assert_eq!(plan.resources().count(), 1);
    assert_eq!(
        plan.resources()
            .next()
            .expect("resource")
            .realization()
            .host_requirement(),
        Some(&HostRequirement::new().require_facility(host_bound_clock_facility()))
    );
}

#[test]
fn same_composition_can_prepare_multiple_plans_and_materialize_isolated_instances() {
    let composition = component_composition("fabric.test.plan.multi");
    let alpha = MaterializationProfile::new("alpha").expect("alpha");
    let beta = MaterializationProfile::new("beta").expect("beta");
    let alpha_host = HostDescriptor::native();
    let beta_host = HostDescriptor::native()
        .with_facility(HostFacilityId::new("fabric.test.plan.beta").expect("facility"));

    let alpha_plan = composition
        .plan_with_profile_on(&alpha, &alpha_host)
        .expect("alpha plan");
    let beta_plan = composition
        .plan_with_profile_on(&beta, &beta_host)
        .expect("beta plan");

    assert_eq!(composition.components().count(), 1);
    assert_eq!(alpha_plan.components().count(), 1);
    assert_eq!(beta_plan.components().count(), 1);
    assert_eq!(alpha_plan.materialization_profile(), &alpha);
    assert_eq!(beta_plan.materialization_profile(), &beta);
    assert_eq!(alpha_plan.host(), Some(&alpha_host));
    assert_eq!(beta_plan.host(), Some(&beta_host));

    let first = alpha_plan
        .materialize("fabric.test.plan.multi.alpha")
        .expect("alpha instance");
    let second = beta_plan
        .materialize("fabric.test.plan.multi.beta")
        .expect("beta instance");

    assert_eq!(first.materialization_plan(), alpha_plan.provenance());
    assert_eq!(second.materialization_plan(), beta_plan.provenance());
    assert_eq!(first.materialization_profile(), &alpha);
    assert_eq!(second.materialization_profile(), &beta);
    assert_ne!(first.generation(), second.generation());

    let first_component = first.component::<Greeter>().expect("first greeter");
    let second_component = second.component::<Greeter>().expect("second greeter");
    assert_eq!(
        first_component.desired().expect("first desired"),
        ComponentDesiredState::Disabled
    );
    assert_eq!(
        second_component.desired().expect("second desired"),
        ComponentDesiredState::Disabled
    );

    first_component
        .set_desired(ComponentDesiredState::Enabled)
        .expect("enable first");
    assert_eq!(
        first_component.desired().expect("first updated"),
        ComponentDesiredState::Enabled
    );
    assert_eq!(
        second_component.desired().expect("second remains initial"),
        ComponentDesiredState::Disabled
    );
    assert_eq!(
        alpha_plan
            .components()
            .next()
            .expect("planned component")
            .initial_participation(),
        ComponentDesiredState::Disabled
    );
}

#[test]
fn one_plan_is_reusable_and_each_materialization_gets_fresh_generation() {
    let composition = component_composition("fabric.test.plan.reuse");
    let plan = composition.plan().expect("plan");

    let first = plan
        .materialize("fabric.test.plan.reuse.first")
        .expect("first instance");
    let second = plan
        .materialize("fabric.test.plan.reuse.second")
        .expect("second instance");

    assert_eq!(first.materialization_plan(), plan.provenance());
    assert_eq!(second.materialization_plan(), plan.provenance());
    assert_ne!(first.instance_id(), second.instance_id());
    assert_ne!(first.generation(), second.generation());
    assert_eq!(
        first.observe().materialization_plan(),
        second.observe().materialization_plan()
    );
}

#[test]
fn existing_materialize_apis_are_sugar_over_plan_provenance() {
    let composition = component_composition("fabric.test.plan.sugar");
    let profile = MaterializationProfile::new("profiled").expect("profile");
    let host = HostDescriptor::native();

    let default_instance = composition
        .materialize("fabric.test.plan.sugar.default")
        .expect("default instance");
    let host_instance = composition
        .materialize_on("fabric.test.plan.sugar.host", &host)
        .expect("host instance");
    let profile_instance = composition
        .materialize_with_profile("fabric.test.plan.sugar.profile", &profile)
        .expect("profile instance");
    let profile_host_instance = composition
        .materialize_with_profile_on("fabric.test.plan.sugar.profile-host", &profile, &host)
        .expect("profile host instance");

    assert_eq!(
        default_instance.materialization_plan(),
        composition.plan().expect("plan").provenance()
    );
    assert_eq!(
        host_instance.materialization_plan(),
        composition.plan_on(&host).expect("host plan").provenance()
    );
    assert_eq!(profile_instance.materialization_profile(), &profile);
    assert_eq!(
        profile_host_instance.materialization_plan(),
        composition
            .plan_with_profile_on(&profile, &host)
            .expect("profile host plan")
            .provenance()
    );
}

#[test]
fn plan_creation_does_not_materialize_runtime_state() {
    let materialize_count = Arc::new(AtomicUsize::new(0));
    let module = CountingModule::new(
        "fabric.test.plan.side-effect.module",
        Arc::clone(&materialize_count),
    );

    let composition = Fabric::new("fabric.test.plan.side-effect")
        .expect("fabric")
        .block("runtime", |block| block.module(module))
        .expect("runtime block")
        .build()
        .expect("composition");

    let plan = composition.plan().expect("plan");
    assert_eq!(materialize_count.load(Ordering::SeqCst), 0);

    let first = plan
        .materialize("fabric.test.plan.side-effect.first")
        .expect("first live instance");
    assert_eq!(materialize_count.load(Ordering::SeqCst), 1);

    let second = plan
        .materialize("fabric.test.plan.side-effect.second")
        .expect("second live instance");
    assert_eq!(materialize_count.load(Ordering::SeqCst), 2);
    assert_ne!(first.generation(), second.generation());
}

#[derive(Clone)]
struct CountingModule {
    module_id: ModuleId,
    materialize_count: Arc<AtomicUsize>,
}

impl CountingModule {
    fn new(module_id: &str, materialize_count: Arc<AtomicUsize>) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            materialize_count,
        }
    }
}

impl Module for CountingModule {
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::new(self.module_id.clone())
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        self.materialize_count.fetch_add(1, Ordering::SeqCst);
        Some(Box::new(CountingRuntime {
            module_id: self.module_id.clone(),
        }))
    }
}

struct CountingRuntime {
    module_id: ModuleId,
}

impl ModuleRuntime for CountingRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, _bindings: &ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}
