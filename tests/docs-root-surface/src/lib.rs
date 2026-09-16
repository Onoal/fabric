//! Compile and behavior witnesses for Fabric's root-first public surface.

#[cfg(test)]
mod root_wildcard {
    use fabric::*;
    use fabric_test_adapter_clock_memory::MemoryClock;
    use fabric_test_resource_clock::{Clock, ClockConfig};

    fabric::resource! {
        pub RootStore {
            id: "docs.root.store";
            schema: provisional;
            config {}
            contracts { primary Api {
                id: "docs.root.store.api"; version: provisional;
                fn value(&self) -> u64;
            }}
            runtime { fn value(&self) -> u64 { 1 } }
        }
    }

    fabric::system! {
        pub RootSystem {
            id: "docs.root.system";
            schema: provisional;
            config {}
            contracts { primary Api {
                id: "docs.root.system.api"; version: provisional;
                fn value(&self) -> u64;
            }}
            runtime { fn value(&self) -> u64 { 2 } }
        }
    }

    fabric::component! {
        pub RootComponent {
            id: "docs.root.component";
            config {}
            operations {
                ping {
                    id: "docs.root.component.ping";
                    input: () = "docs.root.component.ping.input";
                    output: () = "docs.root.component.ping.output";
                    handler |_input: ()| async move { Ok(()) };
                }
            }
        }
    }

    #[derive(Clone)]
    struct Readback;
    #[derive(Clone)]
    struct ReadbackContract;

    impl ResourceAugmentationDefinition<RootStore> for Readback {
        type Config = ();
        type Contract = ReadbackContract;

        fn contract_key() -> ContractKey<Self::Contract> {
            ContractKey::provisional(ContractId::new("docs.root.store.readback").expect("id"))
        }
    }

    #[test]
    fn wildcard_root_supports_normal_authoring() {
        let store = RootStore::select("primary", RootStoreConfig {}).expect("selection");
        let augmentation =
            ResourceAugmentation::<RootStore, Readback>::attach(&store, ()).expect("attachment");
        let system = RootSystem::select(RootSystemConfig {}).expect("selection");
        let component = RootComponent::define(RootComponentConfig {});

        let built = Fabric::new("docs.root")
            .expect("composition")
            .resource(store)
            .resource_augmentation(augmentation)
            .system(system)
            .component(component)
            .build()
            .expect("build");
        assert_eq!(built.manifest().resources().len(), 1);
        assert_eq!(built.manifest().systems().len(), 1);
        assert_eq!(built.manifest().components().len(), 1);
        assert_eq!(built.manifest().resource_augmentations().len(), 1);

        let clock = Clock::select("clock", ClockConfig::default()).expect("selection");
        let realized = clock
            .using(MemoryClock::new(1))
            .expect("adapter realization");
        let _host = HostRequirement::new();
        let _: ResourceId = RootStore::resource_id();
        let _: SystemId = RootSystem::system_id();
        let _: ComponentId = RootComponent::component_id();
        let _: OperationKey<(), ()> = root_component::operations::ping();
        let _: ResourceSchemaDescriptor = RootStore::schema();
        let _: SystemSchemaDescriptor = RootSystem::schema();
        let _ = realized;
    }
}

#[cfg(test)]
mod selective_root {
    use fabric::{
        ComponentId, ContractId, ContractKey, Fabric, HostRequirement, ResourceId, SystemId,
    };

    #[test]
    fn normal_semantic_identifiers_are_selectively_importable() {
        let _ = Fabric::new("docs.root.selective").expect("composition");
        let _ = ResourceId::new("docs.root.resource").expect("resource id");
        let _ = SystemId::new("docs.root.system").expect("system id");
        let _ = ComponentId::new("docs.root.component").expect("component id");
        let _ = ContractKey::<()>::provisional(ContractId::new("docs.root.contract").expect("id"));
        let _ = HostRequirement::new();
    }
}

#[cfg(test)]
mod prelude_compatibility {
    use fabric::prelude::*;

    #[test]
    fn prelude_remains_a_normal_authoring_convenience() {
        let built = Fabric::new("docs.root.prelude")
            .expect("composition")
            .build()
            .expect("build");
        assert!(built.manifest().resources().is_empty());
    }
}

#[cfg(test)]
mod experimental_projection {
    use std::sync::Arc;

    use fabric::experimental::projection::{
        MaterializedProjection, ProjectionContract, ProjectionError, ProjectionService, Projector,
    };

    struct Project;

    impl Projector<&'static str, &'static str, &'static str> for Project {
        fn project(
            &self,
            request: &&'static str,
            projection: &&'static str,
        ) -> Result<MaterializedProjection<&'static str>, ProjectionError> {
            if request == projection {
                Ok(MaterializedProjection::new(*projection))
            } else {
                Err(ProjectionError::invalid_input("mismatch"))
            }
        }
    }

    struct Service;

    impl ProjectionService<&'static str, &'static str, &'static str> for Service {
        fn prepare(
            &self,
            requests: &[&'static str],
            policies: &[&'static str],
        ) -> Result<&'static str, ProjectionError> {
            Project
                .project(&requests[0], &policies[0])
                .map(|value| value.into_parts().0)
        }
    }

    #[test]
    fn umbrella_exposes_experimental_projection_only_through_its_namespace() {
        let contract = ProjectionContract::new(Arc::new(Service));
        assert_eq!(contract.prepare(&["queue"], &["queue"]), Ok("queue"));
    }
}
