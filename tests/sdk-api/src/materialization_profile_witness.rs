use fabric::authoring::ComponentDefinition;
use fabric::prelude::*;
use fabric_test_component_greeter::{Greeter, GreeterConfig};

fn profile_composition(id: &str) -> Composition {
    Fabric::new(id)
        .expect("fabric")
        .component(Greeter::define(GreeterConfig {}))
        .build()
        .expect("composition")
}

#[test]
fn default_materialization_uses_canonical_default_profile() {
    let composition = profile_composition("fabric.test.profile.default");
    let instance = composition
        .materialize("fabric.test.profile.default.instance")
        .expect("instance");
    let observation = instance.observe();

    assert_eq!(
        instance.materialization_profile(),
        &MaterializationProfile::default_profile()
    );
    assert_eq!(
        observation.materialization_profile(),
        &MaterializationProfile::default_profile()
    );
    assert_eq!(
        instance.materialization_profile().name().as_str(),
        "default"
    );
    assert_eq!(composition.components().count(), 1);
}

#[test]
fn same_composition_materializes_with_distinct_profile_provenance_and_hosts() {
    let composition = profile_composition("fabric.test.profile.explicit");
    let alpha = MaterializationProfile::new("alpha").expect("alpha profile");
    let beta = MaterializationProfile::new("beta").expect("beta profile");
    let alpha_host = HostDescriptor::native();
    let beta_host = HostDescriptor::native()
        .with_facility(HostFacilityId::new("fabric.test.profile.diagnostic").expect("facility"));

    let first = composition
        .materialize_with_profile_on("fabric.test.profile.alpha", &alpha, &alpha_host)
        .expect("alpha instance");
    let second = composition
        .materialize_with_profile_on("fabric.test.profile.beta", &beta, &beta_host)
        .expect("beta instance");

    assert_eq!(first.composition_id(), composition.id());
    assert_eq!(second.composition_id(), composition.id());
    assert_ne!(first.instance_id(), second.instance_id());
    assert_ne!(first.generation(), second.generation());
    assert_eq!(first.materialization_profile(), &alpha);
    assert_eq!(second.materialization_profile(), &beta);
    assert_eq!(first.observe().materialization_profile(), &alpha);
    assert_eq!(second.observe().materialization_profile(), &beta);

    let component_ids = composition
        .components()
        .map(|component| component.component_id().clone())
        .collect::<Vec<_>>();
    assert_eq!(component_ids, vec![Greeter::component_id()]);
    assert_eq!(composition.resources().count(), 0);
    assert_eq!(composition.systems().count(), 0);
}

#[test]
fn profile_provenance_does_not_mutate_composition_or_share_live_state() {
    let composition = profile_composition("fabric.test.profile.isolation");
    let alpha = MaterializationProfile::new("diagnostic").expect("diagnostic profile");
    let beta = MaterializationProfile::new("production").expect("production profile");
    let first = composition
        .materialize_with_profile("fabric.test.profile.isolation.alpha", &alpha)
        .expect("alpha instance");
    let second = composition
        .materialize_with_profile("fabric.test.profile.isolation.beta", &beta)
        .expect("beta instance");

    let first_app = first.component::<Greeter>().expect("first greeter");
    let second_app = second.component::<Greeter>().expect("second greeter");
    let second_initial = second_app.desired().expect("second desired");
    let first_next = match first_app.desired().expect("first desired") {
        ComponentDesiredState::Enabled => ComponentDesiredState::Disabled,
        ComponentDesiredState::Disabled => ComponentDesiredState::Enabled,
    };

    assert_eq!(
        first_app
            .set_desired(first_next)
            .expect("set first desired"),
        first_next
    );
    assert_eq!(
        first_app.desired().expect("first desired after"),
        first_next
    );
    assert_eq!(
        second_app.desired().expect("second remains isolated"),
        second_initial
    );
    assert_eq!(composition.components().count(), 1);
    assert_eq!(composition.relations().len(), 0);
}

#[test]
fn invalid_profile_identity_is_bounded() {
    assert!(MaterializationProfile::new("").is_err());
    assert!(MaterializationProfile::new(" profile ").is_err());
    assert!(MaterializationProfile::new("profile/with/slash").is_err());
}
