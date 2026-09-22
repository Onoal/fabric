//! An external-crate witness: adapters here target adaptable definitions
//! exported by the separate macro-context fixture crate.

use fabric::*;
use fabric_test_macro_context::{ImportedCanonicalStore as ImportedStore, ImportedCanonicalSystem};
use fabric_test_macro_context::{
    ImportedValue, RootKey, RootVersionedStore, RootVersionedStoreRealization, RootVersionedSystem,
    RootVersionedSystemRealization,
};

mod facade {
    pub use fabric_test_macro_context::ImportedCanonicalStore;
}

#[allow(dead_code)]
#[derive(Clone)]
struct ImportedReadInput;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct ImportedReadOutput {
    store: u64,
    system: u64,
}

component! {
    ImportedCanonicalProbe {
        id: "fabric.test.macro-context.imported-canonical-probe";

        requires {
            store: ImportedStore(provisional);
        }

        system {
            system: ImportedCanonicalSystem(provisional);
        }

        operations {
            read {
                id: "fabric.test.macro-context.imported-canonical-probe.read";
                input: ImportedReadInput = "fabric.test.macro-context.imported-canonical-probe.read.input";
                output: ImportedReadOutput = "fabric.test.macro-context.imported-canonical-probe.read.output";
                handler |dependencies, _input: ImportedReadInput| async move {
                    Ok(ImportedReadOutput {
                        store: dependencies.store.get(),
                        system: dependencies.system.now(),
                    })
                };
            }
        }
    }
}

#[cfg(test)]
use fabric_test_macro_context::{
    RootStatefulStore, reset_root_adapter_stop, root_adapter_stopped, root_api_identities,
};

adapter! {
    ExternalRootStore
        for resource RootVersionedStore
        implements RootVersionedStoreRealization
    {
        version: "1.0.0";

        runtime {
            fn get(&self, key: RootKey) -> ImportedValue {
                ImportedValue(key.0 + 1)
            }
        }
    }
}

adapter! {
    ImportedStoreAdapter for ImportedStore {
        runtime {
            fn get(&self) -> u64 { 17 }
        }
    }
}

adapter! {
    FullyQualifiedStoreAdapter for fabric_test_macro_context::ImportedCanonicalStore {
        runtime {
            fn get(&self) -> u64 { 18 }
        }
    }
}

adapter! {
    ReexportedStoreAdapter for facade::ImportedCanonicalStore {
        runtime {
            fn get(&self) -> u64 { 19 }
        }
    }
}

adapter! {
    ImportedSystemAdapter for ImportedCanonicalSystem {
        runtime {
            fn now(&self) -> u64 { 23 }
        }
    }
}

mod nested_adapters {
    use fabric::*;
    use fabric_test_macro_context::ImportedCanonicalSystem as NestedSystem;

    adapter! {
        pub NestedSystemAdapter for NestedSystem {
            runtime {
                fn now(&self) -> u64 { 29 }
            }
        }
    }
}

adapter! {
    ExternalRootSystem
        for system RootVersionedSystem
        implements RootVersionedSystemRealization
    {
        version: "1.0.0";

        runtime {
            fn marker(&self) -> ImportedValue {
                ImportedValue(7)
            }
        }
    }
}

#[test]
fn external_adapters_consume_crate_root_adaptable_definitions() {
    reset_root_adapter_stop();

    let store = RootVersionedStore::select("primary")
        .expect("store selection")
        .using(RootStatefulStore::new())
        .expect("stateful root adapter selection");
    let system = RootVersionedSystem::select()
        .expect("system selection")
        .using(ExternalRootSystem::new())
        .expect("external system adapter selection");

    let built = Fabric::new("fabric.test.macro-context.external")
        .expect("fabric")
        .resource(store)
        .system(system)
        .build()
        .expect("build");
    let host = HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("operating system"),
        HostArchitecture::new("x86_64").expect("architecture"),
    );
    let mut instance = built
        .materialize_named_on("fabric.test.macro-context.external.instance", &host)
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop");

    assert!(root_adapter_stopped());

    let _ = ExternalRootStore::new();
}

#[test]
fn imported_and_reexported_canonical_targets_materialize_without_module_path_inference() {
    let resource_selection = ImportedStore::select("imported").expect("resource selection");
    let resource = resource_selection
        .clone()
        .using(ImportedStoreAdapter::new())
        .expect("imported canonical resource adapter");
    let system_selection = ImportedCanonicalSystem::select().expect("system selection");
    let system = system_selection
        .clone()
        .using(ImportedSystemAdapter::new())
        .expect("imported canonical system adapter");
    let built = Fabric::new("fabric.test.macro-context.imported-canonical")
        .expect("fabric")
        .component(
            ImportedCanonicalProbe::define(ImportedCanonicalProbeConfig {})
                .select_resource_provider(&resource_selection)
                .select_system_provider(&system_selection),
        )
        .resource(resource)
        .system(system)
        .build()
        .expect("build");
    let host = HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("operating system"),
        HostArchitecture::new("x86_64").expect("architecture"),
    );
    let mut instance = built
        .materialize_named_on(
            "fabric.test.macro-context.imported-canonical.instance",
            &host,
        )
        .expect("materialize imported canonical targets");
    instance.start().expect("start imported canonical targets");
    let components = instance.components().expect("component host");
    components
        .materialize::<ImportedCanonicalProbe>()
        .expect("materialize semantic API consumer");
    let output = futures::executor::block_on(components.invoke_external(
        &imported_canonical_probe::operations::read(),
        ImportedReadInput,
    ))
    .expect("invoke semantic APIs through imported adapters");
    assert_eq!(
        output,
        ImportedReadOutput {
            store: 17,
            system: 23,
        }
    );
    instance.stop().expect("stop imported canonical targets");

    let _ = FullyQualifiedStoreAdapter::new();
    let _ = ReexportedStoreAdapter::new();
    let _ = nested_adapters::NestedSystemAdapter::new();
}

#[test]
fn owner_derived_api_identities_are_stable_across_macro_locations() {
    let root = root_api_identities();
    let module = fabric_test_macro_context::capability::api_identities();
    let deep = fabric_test_macro_context::outer::middle::inner::api_identities();

    assert_eq!(
        root.0,
        "fabric.resource.api.fabric.test.macro-context.root-store"
    );
    assert_eq!(
        root.1,
        "fabric.system.api.fabric.test.macro-context.root-system"
    );
    assert_eq!(
        module.0,
        "fabric.resource.api.fabric.test.macro-context.module-store"
    );
    assert_eq!(
        module.1,
        "fabric.system.api.fabric.test.macro-context.module-system"
    );
    assert_eq!(
        deep.0,
        "fabric.resource.api.fabric.test.macro-context.deep-store"
    );
    assert_eq!(
        deep.1,
        "fabric.system.api.fabric.test.macro-context.deep-system"
    );
    assert_ne!(root.0, root.1, "resource and system API domains differ");
}
