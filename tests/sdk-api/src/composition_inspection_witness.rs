use fabric::authoring::{
    ResourceAugmentation, ResourceAugmentationDefinition, ResourceAugmentationSupportDefinition,
};
use fabric::core::{ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric::prelude::*;

fn inspection_facility() -> HostFacilityId {
    HostFacilityId::new("fabric.test.inspection.host").expect("facility")
}

fabric::system! {
    Network {
        id: "fabric.test.inspection.network";
        version: "1.0.0";
        api { fn ping(&self) -> u64; }
        runtime { fn ping(&self) -> u64 { 1 } }
    }
}

fabric::system! {
    Scheduler {
        id: "fabric.test.inspection.scheduler";
        relations { requires { network: Network(version = "^1"); } }
        api { fn tick(&self) -> u64; }
    }
}

fabric::resource! {
    Store {
        id: "fabric.test.inspection.store";
        version: "1.2.0";
        relations { requires { network: Network(version = "^1"); } }
        api {
            fn get(&self) -> String;
            fn put(&self, value: String);
        }
    }
}

fabric::adapter! {
    HostStoreAdapter for Store {
        id: "fabric.test.inspection.host-store";
        host: HostRequirement::new().require_facility(inspection_facility());
        relations { requires { scheduler: Scheduler; } }
        runtime {
            fn get(&self) -> String { "value".to_owned() }
            fn put(&self, _value: String) {}
        }
    }
}

fabric::component! {
    App {
        id: "fabric.test.inspection.app";
        relations {
            requires {
                storage: Store(version = "^1");
                clock: Network(version = "^1");
                audit_clock: Network(version = "^1");
            }
        }
        api { fn run(&self); }
    }
}

struct StoreAudit;

#[derive(Clone)]
struct StoreAuditSupport;

#[derive(Clone)]
struct StoreAuditContract;

impl ResourceAugmentationDefinition<Store> for StoreAudit {
    type Config = ();
    type Contract = StoreAuditContract;

    fn contract_key() -> fabric::core::ContractKey<Self::Contract> {
        fabric::core::ContractKey::provisional(
            fabric::core::ContractId::new("fabric.test.inspection.store-audit".to_owned())
                .expect("contract"),
        )
    }
}

impl ResourceAugmentationSupportDefinition<Store, StoreAudit> for StoreAuditSupport {
    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(provider_module_id.clone()).with_host_requirement(
            fabric::core::HostMaterializationRequirement::new(
                provider_module_id,
                HostRequirement::new().require_facility(inspection_facility()),
            ),
        )
    }

    fn materialize(
        &self,
        _attachment: &ResourceAugmentation<Store, StoreAudit>,
        _provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        None
    }
}

#[test]
fn composition_explains_semantic_truth_before_materialization() {
    let network = Network::select().expect("network");
    let scheduler = Scheduler::select().expect("scheduler");
    let primary = Store::select("primary").expect("primary");
    let archive = Store::select("archive").expect("archive");
    let audited = ResourceAugmentation::<Store, StoreAudit>::attach(&primary, ())
        .expect("audit")
        .using(StoreAuditSupport);
    let app = App::define().select_named_resource_provider(
        &fabric::authoring::ComponentResourceRequirement::new(
            fabric::component::ComponentRelationName::new("storage").expect("role"),
            fabric::authoring::Requires::<Store>::versioned(
                fabric::core::ContractVersionRequirement::parse("^1").expect("version"),
            ),
        ),
        &primary,
    );

    let composition = Fabric::new("fabric.test.inspection")
        .expect("fabric")
        .system(network)
        .system(scheduler)
        .resource(primary.using(HostStoreAdapter::new()).expect("adapter"))
        .resource(archive)
        .resource_augmentation(audited)
        .component(app)
        .build()
        .expect("build");

    let resources = composition.resources().collect::<Vec<_>>();
    assert_eq!(resources.len(), 2);
    let primary = resources
        .iter()
        .find(|resource| resource.name().as_str() == "primary")
        .expect("primary");
    assert_eq!(
        primary
            .api()
            .endpoints()
            .iter()
            .map(|endpoint| endpoint.name())
            .collect::<Vec<_>>(),
        vec!["get", "put"]
    );
    assert_eq!(
        primary.realization().kind(),
        SemanticRealizationKind::AdapterDirect
    );
    assert_eq!(
        primary
            .realization()
            .adapter_definition_id()
            .expect("adapter")
            .as_str(),
        "fabric.test.inspection.host-store"
    );
    assert!(primary.realization().host_requirement().is_some());
    assert_eq!(primary.augmentations().count(), 1);
    assert!(
        primary
            .augmentations()
            .next()
            .expect("augmentation")
            .host_requirement()
            .is_some()
    );

    let systems = composition.systems().collect::<Vec<_>>();
    assert!(
        systems
            .iter()
            .any(|system| system.api().endpoints()[0].name() == "ping")
    );
    assert!(
        systems
            .iter()
            .find(|system| system.system_id().as_str() == "fabric.test.inspection.scheduler")
            .expect("scheduler")
            .relations()
            .any(|relation| relation.role().as_str() == "network")
    );

    let component = composition.components().next().expect("component");
    assert_eq!(component.api().endpoints()[0].name(), "run");
    let component_roles = component
        .relations()
        .map(|relation| relation.role().as_str())
        .collect::<Vec<_>>();
    assert!(component_roles.contains(&"storage"));
    assert!(component_roles.contains(&"clock"));
    assert!(component_roles.contains(&"audit_clock"));

    let storage = composition
        .relations()
        .iter()
        .find(|relation| relation.role().as_str() == "storage")
        .expect("storage binding");
    assert_eq!(
        storage.resolved_target(),
        &SemanticRelationTargetOccurrence::Resource {
            resource_id: ResourceId::new("fabric.test.inspection.store").expect("resource"),
            resource_name: ResourceName::new("primary").expect("name"),
        }
    );
    assert_eq!(primary.required_by().count(), 1);
    assert!(composition.relations().iter().any(|relation| matches!(
        relation.owner(),
        SemanticRelationOwner::AdapterRealization { .. }
    )));
}
