//! An external-crate witness: adapters here target adaptable definitions
//! exported by the separate macro-context fixture crate.

use fabric::*;
use fabric_test_macro_context::{
    DifferentialImportedStore as ImportedDifferentialStore,
    DifferentialImportedSystem as ImportedDifferentialSystem,
};
use fabric_test_macro_context::{ImportedCanonicalStore as ImportedStore, ImportedCanonicalSystem};
use fabric_test_macro_context::{ImportedValue, RootKey, RootVersionedStore, RootVersionedSystem};

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

#[allow(dead_code)]
#[derive(Clone)]
struct DifferentialReadInput;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct DifferentialReadOutput {
    read: u64,
    write: u64,
}

#[allow(dead_code)]
#[derive(Clone)]
struct DifferentialClockInput;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct DifferentialClockOutput {
    now: u64,
    label: u64,
}

component! {
    ImportedCanonicalProbe {
        id: "fabric.test.macro-context.imported-canonical-probe";

        relations {
            requires {
                store: ImportedStore(provisional);
                system: ImportedCanonicalSystem(provisional);
            }
        }
        api { fn read(&self, input: ImportedReadInput) -> ImportedReadOutput; }
        runtime {
            fn read(&self, input: ImportedReadInput) -> ImportedReadOutput {
                let _ = input;
                ImportedReadOutput {
                    store: self.relations().store.get(),
                    system: self.relations().system.now(),
                }
            }
        }
    }
}

component! {
    DifferentialImportedResourceProbe {
        id: "fabric.test.macro-context.differential-imported-resource-probe";

        relations { requires { store: ImportedDifferentialStore(provisional); } }
        api { fn read(&self, input: DifferentialReadInput) -> DifferentialReadOutput; }
        runtime {
            fn read(&self, input: DifferentialReadInput) -> DifferentialReadOutput {
                let _ = input;
                DifferentialReadOutput {
                    read: self.relations().store.read(4),
                    write: self.relations().store.write(4, 9),
                }
            }
        }
    }
}

component! {
    DifferentialImportedSystemProbe {
        id: "fabric.test.macro-context.differential-imported-system-probe";

        relations { requires { clock: ImportedDifferentialSystem(provisional); } }
        api { fn read(&self, input: DifferentialClockInput) -> DifferentialClockOutput; }
        runtime {
            fn read(&self, input: DifferentialClockInput) -> DifferentialClockOutput {
                let _ = input;
                DifferentialClockOutput {
                    now: self.relations().clock.now(),
                    label: self.relations().clock.label(),
                }
            }
        }
    }
}

#[cfg(test)]
use fabric_test_macro_context::{
    RootStatefulStore, reset_root_adapter_stop, root_adapter_stopped, root_api_identities,
};

adapter! {
    ExternalRootStore for RootVersionedStore {
        id: "test.external-root-store";
        runtime {
            fn get(&self, key: RootKey) -> ImportedValue {
                ImportedValue(key.0 + 1)
            }
        }
    }
}

adapter! {
    ImportedDifferentialStoreAdapter for ImportedDifferentialStore {
        id: "test.imported-differential-store-adapter";
        runtime {
            fn write(&self, _key: u64, value: u64) -> u64 { value }
            fn read_raw(&self, key: u64) -> u64 { key }
        }
    }
}

adapter! {
    ImportedDifferentialSystemAdapter for ImportedDifferentialSystem {
        id: "test.imported-differential-system-adapter";
        runtime {
            fn label(&self) -> u64 { 7 }
            fn raw_now(&self) -> u64 { 41 }
        }
    }
}

adapter! {
    ImportedStoreAdapter for ImportedStore {
        id: "test.imported-store-adapter";
        runtime {
            fn get(&self) -> u64 { 17 }
        }
    }
}

adapter! {
    FullyQualifiedStoreAdapter for fabric_test_macro_context::ImportedCanonicalStore {
        id: "test.fully-qualified-store-adapter";
        runtime {
            fn get(&self) -> u64 { 18 }
        }
    }
}

adapter! {
    ReexportedStoreAdapter for facade::ImportedCanonicalStore {
        id: "test.reexported-store-adapter";
        runtime {
            fn get(&self) -> u64 { 19 }
        }
    }
}

adapter! {
    ImportedSystemAdapter for ImportedCanonicalSystem {
        id: "test.imported-system-adapter";
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
            id: "test.nested-system-adapter";
            runtime {
                fn now(&self) -> u64 { 29 }
            }
        }
    }
}

adapter! {
    ExternalRootSystem for RootVersionedSystem {
        id: "test.external-root-system";
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
        .materialize_on("fabric.test.macro-context.external.instance", &host)
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
    let differential_resource_selection =
        ImportedDifferentialStore::select("differential").expect("differential resource selection");
    let differential_resource = differential_resource_selection
        .clone()
        .using(ImportedDifferentialStoreAdapter::new())
        .expect("imported differential resource adapter");
    let differential_system_selection =
        ImportedDifferentialSystem::select().expect("differential system selection");
    let differential_system = differential_system_selection
        .clone()
        .using(ImportedDifferentialSystemAdapter::new())
        .expect("imported differential system adapter");
    let built = Fabric::new("fabric.test.macro-context.imported-canonical")
        .expect("fabric")
        .component(
            ImportedCanonicalProbe::define()
                .select_resource_provider(&resource_selection)
                .select_system_provider(&system_selection),
        )
        .component(
            DifferentialImportedResourceProbe::define()
                .select_resource_provider(&differential_resource_selection),
        )
        .component(
            DifferentialImportedSystemProbe::define()
                .select_system_provider(&differential_system_selection),
        )
        .resource(resource)
        .system(system)
        .resource(differential_resource)
        .system(differential_system)
        .build()
        .expect("build");
    let host = HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("operating system"),
        HostArchitecture::new("x86_64").expect("architecture"),
    );
    let mut instance = built
        .materialize_on(
            "fabric.test.macro-context.imported-canonical.instance",
            &host,
        )
        .expect("materialize imported canonical targets");
    instance.start().expect("start imported canonical targets");
    let components = instance.components().expect("component host");
    components
        .materialize::<ImportedCanonicalProbe>()
        .expect("materialize semantic API consumer");
    let output = futures::executor::block_on(
        components.invoke_external(&imported_canonical_probe::api::read(), ImportedReadInput),
    )
    .expect("invoke semantic APIs through imported adapters");
    assert_eq!(
        output,
        ImportedReadOutput {
            store: 17,
            system: 23,
        }
    );
    components
        .materialize::<DifferentialImportedResourceProbe>()
        .expect("materialize imported differential resource consumer");
    let differential_resource_output = futures::executor::block_on(components.invoke_external(
        &differential_imported_resource_probe::api::read(),
        DifferentialReadInput,
    ))
    .expect("invoke mediated and delegated Resource API methods");
    assert_eq!(
        differential_resource_output,
        DifferentialReadOutput { read: 5, write: 9 }
    );
    components
        .materialize::<DifferentialImportedSystemProbe>()
        .expect("materialize imported differential System consumer");
    let differential_system_output = futures::executor::block_on(components.invoke_external(
        &differential_imported_system_probe::api::read(),
        DifferentialClockInput,
    ))
    .expect("invoke mediated and delegated System API methods");
    assert_eq!(
        differential_system_output,
        DifferentialClockOutput { now: 42, label: 7 }
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
