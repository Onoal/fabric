//! An external-crate witness: adapters here target adaptable definitions
//! exported by the separate macro-context fixture crate.

use fabric::*;
use fabric_test_macro_context::{
    ImportedValue, RootKey, RootVersionedStore, RootVersionedStoreRealization, RootVersionedSystem,
    RootVersionedSystemRealization,
};

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
