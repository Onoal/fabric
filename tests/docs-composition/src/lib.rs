// Keep the Composition guide's normal import form in this compile fixture.
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
        schema: provisional;
        config { label: String; }
        contracts {
            primary Api {
                id: "example.note-store.api";
                version: provisional;
                fn label(&self) -> String;
            }
        }
        runtime {
            fn label(&self) -> String { self.config.label.clone() }
        }
    }
}

fabric::component! {
    pub StoreProbe {
        id: "example.store-probe";
        config {}
        requires {
            primary_store: NoteStore(provisional);
            cache_store: NoteStore(provisional);
        }
        operations {
            inspect {
                id: "example.store-probe.inspect";
                input: () = "example.store-probe.inspect.input";
                output: StoreObservation = "example.store-probe.inspect.output";
                handler |dependencies, _input: ()| async move {
                    Ok(StoreObservation {
                        primary: dependencies.primary_store.label(),
                        cache: dependencies.cache_store.label(),
                    })
                };
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
            StoreProbe::define(StoreProbeConfig {})
                .select_named_resource_provider(
                    &store_probe::requirements::primary_store(),
                    &primary,
                )
                .select_named_resource_provider(&store_probe::requirements::cache_store(), &cache),
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
        .expect("materialize Component");
    let observation = futures::executor::block_on(
        components.invoke_external(&store_probe::operations::inspect(), ()),
    )
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
        .expect("dematerialize Component");
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
        .component(StoreProbe::define(StoreProbeConfig {}))
        .build()
        .expect("one available provider can satisfy both requirements");

    assert_eq!(built.manifest().resources().len(), 1);
}
