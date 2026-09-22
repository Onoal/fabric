#[cfg(test)]
mod tests {
    use fabric::prelude::*;
    use fabric_test_resource_counter::{
        AdaptedCounter, AdaptedCounterConfig, DirectCounter, DirectCounterConfig,
        FixedCounterAdapter, FixedCounterAdapterConfig,
    };

    fabric::resource! {
        pub NoteStore {
            id: "example.note-store";
            version: "0.1.0";
            config { label: String; }
            api {
                fn label(&self) -> String;
            }
            runtime { fn label(&self) -> String { self.config.label.clone() } }
        }
    }

    #[test]
    fn named_occurrences_and_adapter_realization_are_declarative() {
        let primary =
            DirectCounter::select("primary", DirectCounterConfig { value: 1 }).expect("name");
        let cache = DirectCounter::select("cache", DirectCounterConfig { value: 2 }).expect("name");
        assert_ne!(primary.name(), cache.name());

        let built = Fabric::new("example.resources")
            .expect("id")
            .resource(primary)
            .resource(cache)
            .build()
            .expect("build");
        assert_eq!(built.manifest().resources().len(), 2);

        let note_store = NoteStore::select(
            "notes",
            NoteStoreConfig {
                label: "notes".to_owned(),
            },
        )
        .expect("name");
        assert_eq!(note_store.name().as_str(), "notes");

        let realized = AdaptedCounter::select("primary", AdaptedCounterConfig {})
            .expect("name")
            .using(FixedCounterAdapter::new(FixedCounterAdapterConfig {
                value: 7,
            }))
            .expect("adapter");
        assert_eq!(realized.resource().name().as_str(), "primary");
    }
}
