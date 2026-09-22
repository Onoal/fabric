//! An external-crate witness: adapters here target adaptable definitions
//! exported by the separate macro-context fixture crate.

use fabric::*;
use fabric_test_macro_context::{
    ImportedValue, RootKey, RootVersionedStore, RootVersionedStoreRealization, RootVersionedSystem,
    RootVersionedSystemRealization,
};

#[cfg(test)]
use fabric_test_macro_context::{
    RootStatefulStore, RootStatefulStoreConfig, RootVersionedStoreConfig,
    RootVersionedSystemConfig, reset_root_adapter_stop, root_adapter_stopped,
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

    let store = RootVersionedStore::select("primary", RootVersionedStoreConfig {})
        .expect("store selection")
        .using(RootStatefulStore::new(RootStatefulStoreConfig {}))
        .expect("stateful root adapter selection");
    let system = RootVersionedSystem::select(RootVersionedSystemConfig {})
        .expect("system selection")
        .using(ExternalRootSystem::new(ExternalRootSystemConfig {}))
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

    let _ = ExternalRootStore::new(ExternalRootStoreConfig {});
}
