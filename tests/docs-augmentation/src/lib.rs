//! Compile-verified counterparts for the public augmentation concept examples.

#[cfg(test)]
mod tests {
    use fabric::authoring::*;
    use fabric::core::{ContractId, ContractKey, ModuleDeclaration, ModuleId, ModuleRuntime};
    use fabric_test_adapter_clock_memory::MemoryClock;
    use fabric_test_component_greeter::{Greeter, GreeterConfig};
    use fabric_test_resource_clock::{Clock, ClockConfig};
    use fabric_test_system_operations::{TestOperations, TestOperationsConfig};

    #[derive(Clone)]
    struct ResourceReadback;
    #[derive(Clone)]
    struct ResourceReadbackContract;

    impl ResourceAugmentationDefinition<Clock> for ResourceReadback {
        type Config = ();
        type Contract = ResourceReadbackContract;

        fn contract_key() -> ContractKey<Self::Contract> {
            ContractKey::provisional(
                ContractId::new("docs.augmentation.resource.readback").expect("id"),
            )
        }
    }

    #[derive(Clone)]
    struct ResourceReadbackSupport;

    impl ResourceAugmentationSupportDefinition<Clock, ResourceReadback> for ResourceReadbackSupport {
        fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
            ModuleDeclaration::new(provider_module_id)
        }

        fn materialize(
            &self,
            _: &ResourceAugmentation<Clock, ResourceReadback>,
            _: ModuleId,
        ) -> Option<Box<dyn ModuleRuntime>> {
            None
        }
    }

    #[derive(Clone)]
    struct SystemObservation;
    #[derive(Clone)]
    struct SystemObservationContract;

    impl SystemAugmentationDefinition<TestOperations> for SystemObservation {
        type Config = ();
        type Contract = SystemObservationContract;

        fn contract_key() -> ContractKey<Self::Contract> {
            ContractKey::provisional(
                ContractId::new("docs.augmentation.system.observation").expect("id"),
            )
        }
    }

    #[derive(Clone)]
    struct SystemObservationSupport;

    impl SystemAugmentationSupportDefinition<TestOperations, SystemObservation>
        for SystemObservationSupport
    {
        fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
            ModuleDeclaration::new(provider_module_id)
        }

        fn materialize(
            &self,
            _: &SystemAugmentation<TestOperations, SystemObservation>,
            _: ModuleId,
        ) -> Option<Box<dyn ModuleRuntime>> {
            None
        }
    }

    #[derive(Clone)]
    struct Audit;
    #[derive(Clone)]
    struct AuditContract;
    #[derive(Clone)]
    struct Trace;
    #[derive(Clone)]
    struct TraceContract;

    impl ComponentAugmentationDefinition<Greeter> for Audit {
        type Config = ();
        type Contract = AuditContract;

        fn contract_key() -> ContractKey<Self::Contract> {
            ContractKey::provisional(
                ContractId::new("docs.augmentation.component.audit").expect("id"),
            )
        }
    }

    impl ComponentAugmentationDefinition<Greeter> for Trace {
        type Config = ();
        type Contract = TraceContract;

        fn contract_key() -> ContractKey<Self::Contract> {
            ContractKey::provisional(
                ContractId::new("docs.augmentation.component.trace").expect("id"),
            )
        }
    }

    #[derive(Clone)]
    struct AuditSupport;
    #[derive(Clone)]
    struct TraceSupport;

    impl ComponentAugmentationSupportDefinition<Greeter, Audit> for AuditSupport {
        fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
            ModuleDeclaration::new(provider_module_id)
        }

        fn materialize(&self, _: &(), _: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
            None
        }

        fn prepare(
            &self,
            _: &(),
            _: &fabric::component::ComponentParticipationScope,
        ) -> Result<(), fabric::component::ComponentError> {
            Ok(())
        }
    }

    impl ComponentAugmentationSupportDefinition<Greeter, Trace> for TraceSupport {
        fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
            ModuleDeclaration::new(provider_module_id)
        }

        fn materialize(&self, _: &(), _: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
            None
        }

        fn prepare(
            &self,
            _: &(),
            _: &fabric::component::ComponentParticipationScope,
        ) -> Result<(), fabric::component::ComponentError> {
            Ok(())
        }
    }

    #[test]
    fn resource_and_system_examples_compile_with_target_bound_requirements() {
        let resource = Clock::select("primary", ClockConfig::default()).expect("selection");
        let resource_support =
            ResourceAugmentation::<Clock, ResourceReadback>::attach(&resource, ())
                .expect("attachment")
                .using(ResourceReadbackSupport);
        let resource_requirement = resource_support
            .require_from(&resource)
            .expect("target bound");
        let resource_built = Fabric::new("docs.augmentation.resource")
            .expect("id")
            .resource(resource.using(MemoryClock::new(1)).expect("adapter"))
            .resource_augmentation(resource_support)
            .build()
            .expect("build");
        assert_eq!(resource_built.manifest().resource_augmentations().len(), 1);
        let _ = resource_requirement;

        let bare_resource = Clock::select("secondary", ClockConfig::default()).expect("selection");
        let bare_resource_attachment =
            ResourceAugmentation::<Clock, ResourceReadback>::attach(&bare_resource, ())
                .expect("bare attachment");
        let bare_resource_built = Fabric::new("docs.augmentation.resource.bare")
            .expect("id")
            .resource(bare_resource.using(MemoryClock::new(2)).expect("adapter"))
            .resource_augmentation(bare_resource_attachment)
            .build()
            .expect("bare attachment remains declarative truth");
        assert_eq!(
            bare_resource_built
                .manifest()
                .resource_augmentations()
                .len(),
            1
        );

        let system = TestOperations::select(TestOperationsConfig::new(1, 1)).expect("selection");
        let system_support =
            SystemAugmentation::<TestOperations, SystemObservation>::attach(&system, ())
                .expect("attachment")
                .using(SystemObservationSupport);
        let system_requirement = system_support.require_from(&system).expect("target bound");
        let system_built = Fabric::new("docs.augmentation.system")
            .expect("id")
            .system(system)
            .system_augmentation(system_support)
            .build()
            .expect("build");
        assert_eq!(system_built.manifest().system_augmentations().len(), 1);
        let _ = system_requirement;

        let bare_system =
            TestOperations::select(TestOperationsConfig::new(2, 1)).expect("selection");
        let bare_system_attachment =
            SystemAugmentation::<TestOperations, SystemObservation>::attach(&bare_system, ())
                .expect("bare attachment");
        let bare_system_built = Fabric::new("docs.augmentation.system.bare")
            .expect("id")
            .system(bare_system)
            .system_augmentation(bare_system_attachment)
            .build()
            .expect("bare attachment remains declarative truth");
        assert_eq!(bare_system_built.manifest().system_augmentations().len(), 1);
    }

    #[test]
    fn component_multi_augmentation_and_bare_attachment_examples_compile() {
        let audit = Greeter::define(GreeterConfig {})
            .augment::<Audit>(())
            .expect("attachment")
            .using(AuditSupport);
        let audit_requirement = audit.requirement();
        let trace = audit
            .into_set()
            .augment::<Trace>(())
            .expect("attachment")
            .using(TraceSupport);
        let trace_requirement = trace.requirement();
        let built = Fabric::new("docs.augmentation.component")
            .expect("id")
            .component(trace)
            .build()
            .expect("build");
        assert_eq!(built.manifest().component_augmentations().len(), 2);
        let _ = (audit_requirement, trace_requirement);

        let bare = Greeter::define(GreeterConfig {})
            .augment::<Audit>(())
            .expect("bare attachment")
            .into_set();
        let bare_built = Fabric::new("docs.augmentation.component.bare")
            .expect("id")
            .component(bare)
            .build()
            .expect("bare attachment remains declarative truth");
        assert_eq!(bare_built.manifest().component_augmentations().len(), 1);
    }
}
