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

component! {
    DifferentialImportedResourceProbe {
        id: "fabric.test.macro-context.differential-imported-resource-probe";

        requires {
            store: ImportedDifferentialStore(provisional);
        }

        operations {
            read {
                id: "fabric.test.macro-context.differential-imported-resource-probe.read";
                input: DifferentialReadInput = "fabric.test.macro-context.differential-imported-resource-probe.read.input";
                output: DifferentialReadOutput = "fabric.test.macro-context.differential-imported-resource-probe.read.output";
                handler |dependencies, _input: DifferentialReadInput| async move {
                    Ok(DifferentialReadOutput {
                        read: dependencies.store.read(4),
                        write: dependencies.store.write(4, 9),
                    })
                };
            }
        }
    }
}

component! {
    DifferentialImportedSystemProbe {
        id: "fabric.test.macro-context.differential-imported-system-probe";

        system {
            clock: ImportedDifferentialSystem(provisional);
        }

        operations {
            read {
                id: "fabric.test.macro-context.differential-imported-system-probe.read";
                input: DifferentialClockInput = "fabric.test.macro-context.differential-imported-system-probe.read.input";
                output: DifferentialClockOutput = "fabric.test.macro-context.differential-imported-system-probe.read.output";
                handler |dependencies, _input: DifferentialClockInput| async move {
                    Ok(DifferentialClockOutput {
                        now: dependencies.clock.now(),
                        label: dependencies.clock.label(),
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
    ExternalRootStore for RootVersionedStore {
        runtime {
            fn get(&self, key: RootKey) -> ImportedValue {
                ImportedValue(key.0 + 1)
            }
        }
    }
}

adapter! {
    ImportedDifferentialStoreAdapter for ImportedDifferentialStore {
        runtime {
            fn write(&self, _key: u64, value: u64) -> u64 { value }
            fn read_raw(&self, key: u64) -> u64 { key }
        }
    }
}

adapter! {
    ImportedDifferentialSystemAdapter for ImportedDifferentialSystem {
        runtime {
            fn label(&self) -> u64 { 7 }
            fn raw_now(&self) -> u64 { 41 }
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
    ExternalRootSystem for RootVersionedSystem {
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
            ImportedCanonicalProbe::define(ImportedCanonicalProbeConfig {})
                .select_resource_provider(&resource_selection)
                .select_system_provider(&system_selection),
        )
        .component(
            DifferentialImportedResourceProbe::define(DifferentialImportedResourceProbeConfig {})
                .select_resource_provider(&differential_resource_selection),
        )
        .component(
            DifferentialImportedSystemProbe::define(DifferentialImportedSystemProbeConfig {})
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
    components
        .materialize::<DifferentialImportedResourceProbe>()
        .expect("materialize imported differential resource consumer");
    let differential_resource_output = futures::executor::block_on(components.invoke_external(
        &differential_imported_resource_probe::operations::read(),
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
        &differential_imported_system_probe::operations::read(),
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
