#[cfg(test)]
mod tests {
    use fabric::prelude::*;
    use fabric_test_adapter_external::{
        ExternalCounterAdapter, ExternalCounterAdapterConfig, ExternalOperationsAdapter,
        ExternalOperationsAdapterConfig, HostBoundExternalCounterAdapter,
        HostBoundExternalCounterAdapterConfig, external_counter_host_facility,
    };
    use fabric_test_resource_counter::{AdaptedCounter, AdaptedCounterConfig};
    use fabric_test_system_operations::{
        AdaptedOperations, AdaptedOperationsConfig, IncompatibleSchemaOperationsAdapter,
        IncompatibleSchemaOperationsAdapterConfig,
    };

    #[test]
    fn adapter_realizations_are_selected_and_host_checked_declaratively() {
        let resource = AdaptedCounter::select("primary", AdaptedCounterConfig {})
            .expect("selection")
            .using(ExternalCounterAdapter::new(ExternalCounterAdapterConfig {
                value: 7,
            }))
            .expect("compatible resource Adapter");
        let built = Fabric::new("example.adapter.resource")
            .expect("composition")
            .resource(resource)
            .build()
            .expect("build");
        assert_eq!(built.manifest().resources().len(), 1);

        let system = AdaptedOperations::select(AdaptedOperationsConfig::default())
            .expect("selection")
            .using(ExternalOperationsAdapter::new(
                ExternalOperationsAdapterConfig { value: 9 },
            ))
            .expect("compatible system Adapter");
        assert_eq!(
            system.system().module_id(),
            system.adapter().provider_module_id()
        );

        let incompatible = AdaptedOperations::select(AdaptedOperationsConfig::default())
            .expect("selection")
            .using(IncompatibleSchemaOperationsAdapter::new(
                IncompatibleSchemaOperationsAdapterConfig { value: 1 },
            ));
        assert!(matches!(
            incompatible,
            Err(SystemCompatibilityError::SchemaIncompatible { .. })
        ));

        let host_bound = AdaptedCounter::select("host-bound", AdaptedCounterConfig {})
            .expect("selection")
            .using(HostBoundExternalCounterAdapter::new(
                HostBoundExternalCounterAdapterConfig { value: 4 },
            ))
            .expect("compatible Adapter");
        let built = Fabric::new("example.adapter.host")
            .expect("composition")
            .resource(host_bound)
            .build()
            .expect("build");
        let host = HostDescriptor::native();
        assert!(
            built
                .materialize_named_on("example.adapter.host.missing", &host)
                .is_err()
        );
        let mut instance = built
            .materialize_named_on(
                "example.adapter.host.matching",
                &host.with_facility(external_counter_host_facility()),
            )
            .expect("compatible Host");
        instance.start().expect("start");
        instance.stop().expect("stop instance");
    }
}
