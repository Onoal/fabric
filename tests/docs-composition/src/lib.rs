// Keep the Composition guide's normal import form in this compile fixture.
#[cfg(test)]
use fabric::authoring::Requires;
#[cfg(test)]
use fabric::authoring::component::ComponentResourceRequirement;
#[cfg(test)]
use fabric::component::declaration::ComponentRelationName;
#[cfg(test)]
use fabric::core::ContractVersionRequirement;
#[allow(unused_imports)]
use fabric::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreObservation {
    pub primary: String,
    pub cache: String,
}

fabric::resource! {
    pub NoteStore {
        id: "example.note-store";
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

fabric::component! {
    pub StoreProbe {
        id: "example.store-probe";
        relations {
            requires {
                primary_store: NoteStore(version = "^0.1");
                cache_store: NoteStore(version = "^0.1");
            }
        }
        api { fn inspect(&self) -> StoreObservation; }
        runtime {
            fn inspect(&self) -> StoreObservation {
                StoreObservation {
                    primary: self.relations().primary_store.label(),
                    cache: self.relations().cache_store.label(),
                }
            }
        }
    }
}

#[cfg(test)]
fn primary() -> ResourceSelection<NoteStore> {
    NoteStore::select(
        "primary",
        NoteStoreConfig {
            label: "primary".to_owned(),
        },
    )
    .expect("valid ResourceName")
}

#[cfg(test)]
fn cache() -> ResourceSelection<NoteStore> {
    NoteStore::select(
        "cache",
        NoteStoreConfig {
            label: "cache".to_owned(),
        },
    )
    .expect("valid ResourceName")
}

#[test]
fn minimal_composition_builds_declaration_truth() {
    let built = Fabric::new("example.minimal")
        .expect("valid CompositionId")
        .build()
        .expect("valid declaration");

    assert_eq!(built.composition().id().as_str(), "example.minimal");
    assert!(built.manifest().resources().is_empty());
}

#[test]
fn selected_resource_occurrences_bind_component_roles_independently() {
    let primary = primary();
    let cache = cache();
    let built = Fabric::new("example.store-composition")
        .expect("valid CompositionId")
        .resource(primary.clone())
        .resource(cache.clone())
        .component(
            StoreProbe::define()
                .select_named_resource_provider(
                    &ComponentResourceRequirement::new(
                        ComponentRelationName::new("primary_store").expect("relation name"),
                        Requires::<NoteStore>::versioned(
                            ContractVersionRequirement::parse("^0.1").expect("version"),
                        ),
                    ),
                    &primary,
                )
                .select_named_resource_provider(
                    &ComponentResourceRequirement::new(
                        ComponentRelationName::new("cache_store").expect("relation name"),
                        Requires::<NoteStore>::versioned(
                            ContractVersionRequirement::parse("^0.1").expect("version"),
                        ),
                    ),
                    &cache,
                ),
        )
        .build()
        .expect("unambiguous selected providers");

    let bindings = built.manifest().component_resource_bindings();
    assert_eq!(bindings[0].requirement_name().as_str(), "primary_store");
    assert_eq!(bindings[0].resource_name().as_str(), "primary");
    assert_eq!(bindings[1].requirement_name().as_str(), "cache_store");
    assert_eq!(bindings[1].resource_name().as_str(), "cache");

    let mut instance = built
        .materialize_named("example.store-composition.local")
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<StoreProbe>()
        .expect("materialize ComponentInstanceBinding");
    let observation =
        futures::executor::block_on(components.invoke_external(&store_probe::api::inspect(), ()))
            .expect("runtime invocation");
    assert_eq!(
        observation,
        StoreObservation {
            primary: "primary".to_owned(),
            cache: "cache".to_owned(),
        }
    );
    components
        .dematerialize::<StoreProbe>()
        .expect("dematerialize ComponentInstanceBinding");
    instance.stop().expect("stop instance");
}

#[cfg(test)]
fn local_store_stack() -> Fabric {
    Fabric::new("example.reusable")
        .expect("valid CompositionId")
        .resource(primary())
}

#[test]
fn ordinary_rust_can_reuse_partial_fabric_authoring_before_build() {
    let built = local_store_stack()
        .component(StoreProbe::define())
        .build()
        .expect("one available provider can satisfy both requirements");

    assert_eq!(built.manifest().resources().len(), 1);
}
