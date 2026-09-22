#[cfg(test)]
mod tests {
    use fabric::authoring::SystemDefinition;
    use fabric::prelude::*;
    use fabric_test_system_operations::{
        AdaptedOperations, AdaptedOperationsConfig, FixedOperationsAdapter,
        FixedOperationsAdapterConfig, IncompatibleSchemaOperationsAdapter,
        IncompatibleSchemaOperationsAdapterConfig, TestOperations, TestOperationsConfig,
        operations_system_id,
    };

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Marker {
        value: u64,
    }

    fabric::system! {
        pub AuditSystem {
            id: "example.audit";
            version: "0.1.0";
            config { seed: u64; }
            api {
                fn marker(&self) -> u64;
            }
            runtime { fn marker(&self) -> u64 { self.config.seed } }
        }
    }

    fabric::system! {
        pub DerivedAuditSystem {
            id: "example.derived-audit";
            version: "0.1.0";
            config { offset: u64; }
            relations { requires { base: AuditSystem(version = "^0.1"); } }
            api {
                fn marker(&self) -> u64;
            }
            runtime { fn marker(&self) -> u64 { self.base.marker() + self.config.offset } }
        }
    }

    fabric::component! {
        pub AuditConsumer {
            id: "example.audit-consumer";
            system { audit: AuditSystem(version = "^0.1"); }
            operations {
                observe {
                    id: "example.audit-consumer.observe";
                    input: () = "example.audit-consumer.observe.input";
                    output: Marker = "example.audit-consumer.observe.output";
                    handler |dependencies, _input: ()| async move {
                        Ok(Marker { value: dependencies.audit.marker() })
                    };
                }
            }
        }
    }

    #[test]
    fn system_selection_manifest_realization_and_uniqueness_are_declarative() {
        let audit = AuditSystem::select(AuditSystemConfig { seed: 7 }).expect("selection");
        let built = Fabric::new("example.system")
            .expect("composition id")
            .system(audit)
            .build()
            .expect("build");
        assert_eq!(built.manifest().systems().len(), 1);
        assert_eq!(
            built.manifest().systems()[0].system_id(),
            &AuditSystem::system_id()
        );
        assert_eq!(
            built.manifest().systems()[0].schema().system(),
            &AuditSystem::system_id()
        );

        let duplicate = Fabric::new("example.system.duplicate")
            .expect("composition id")
            .system(TestOperations::select(TestOperationsConfig::new(1, 1)).expect("first"))
            .system(TestOperations::select(TestOperationsConfig::new(2, 1)).expect("second"))
            .build()
            .expect_err("one SystemId must be coherent per Composition");
        assert!(matches!(
            duplicate,
            FabricBuildError::Composition(CompositionError::DuplicateModuleId { .. })
        ));

        let realized = AdaptedOperations::select(AdaptedOperationsConfig::default())
            .expect("selection")
            .using(FixedOperationsAdapter::new(FixedOperationsAdapterConfig {
                value: 7,
            }))
            .expect("compatible adapter");
        assert!(
            realized
                .system()
                .module_id()
                .as_str()
                .contains("fabric.system")
        );

        let incompatible = AdaptedOperations::select(AdaptedOperationsConfig::default())
            .expect("selection")
            .using(IncompatibleSchemaOperationsAdapter::new(
                IncompatibleSchemaOperationsAdapterConfig { value: 7 },
            ));
        assert!(matches!(
            incompatible,
            Err(SystemCompatibilityError::SchemaIncompatible { .. })
        ));
        assert_eq!(operations_system_id().as_str(), "fabric.test.operations");
    }

    #[test]
    fn component_and_system_dependencies_receive_typed_system_contracts() {
        let audit = AuditSystem::select(AuditSystemConfig { seed: 7 }).expect("selection");
        let derived =
            DerivedAuditSystem::select(DerivedAuditSystemConfig { offset: 3 }).expect("selection");
        let built = Fabric::new("example.system.dependencies")
            .expect("composition id")
            .system(audit)
            .system(derived)
            .component(AuditConsumer::define(AuditConsumerConfig {}))
            .build()
            .expect("resolved dependencies");

        let mut instance = built
            .materialize_named("example.system.dependencies.local")
            .expect("materialize");
        instance.start().expect("start");
        let components = instance.components().expect("component host");
        components
            .materialize::<AuditConsumer>()
            .expect("materialize Component");
        let output = futures::executor::block_on(
            components.invoke_external(&audit_consumer::operations::observe(), ()),
        )
        .expect("invoke");
        assert_eq!(output, Marker { value: 7 });
        instance.stop().expect("stop instance");
    }
}
