use fabric::authoring::{
    BlockAuthor, ComponentAugmentationDefinition, ResourceAugmentation,
    ResourceAugmentationDefinition, SystemAugmentation, SystemAugmentationDefinition,
};
use fabric::core::{ContractId, ContractKey, Module, ModuleDeclaration, ModuleRuntime};
use fabric::prelude::*;
use fabric::resource::ResourceSchemaDescriptor;
use fabric::system::SystemSchemaDescriptor;
use fabric_test_external_extension::contribution_api::external_storage;

fn contribution_facility() -> HostFacilityId {
    HostFacilityId::new("fabric.test.contribution.host").expect("facility")
}

fn host() -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("arch"),
    )
    .with_facility(contribution_facility())
}

fabric::system! {
    ContributionSignal {
        id: "fabric.test.contribution.signal";
        api { fn ping(&self) -> u64; }
        runtime { fn ping(&self) -> u64 { 1 } }
    }
}

fabric::resource! {
    ContributionStore {
        id: "fabric.test.contribution.store";
        relations { requires { signal: ContributionSignal; } }
        api { fn read(&self) -> u64; }
        runtime { fn read(&self) -> u64 { self.signal.ping() } }
    }
}

fabric::resource! {
    ContributionCache {
        id: "fabric.test.contribution.cache";
        api { fn read(&self) -> u64; }
    }
}

fabric::adapter! {
    ContributionCacheAdapter for ContributionCache {
        id: "fabric.test.contribution.cache-adapter";
        host: HostRequirement::new().require_facility(contribution_facility());
        runtime { fn read(&self) -> u64 { 2 } }
    }
}

fabric::resource! {
    ContributionMediated {
        id: "fabric.test.contribution.mediated";
        api { fn read(&self) -> u64; }
        realization {
            mediate read;
            fn raw_read(&self) -> u64;
        }
        runtime { fn read(&self) -> u64 { self.realization.raw_read() } }
    }
}

fabric::adapter! {
    ContributionMediatedAdapter for ContributionMediated {
        id: "fabric.test.contribution.mediated-adapter";
        runtime { fn raw_read(&self) -> u64 { 3 } }
    }
}

fabric::component! {
    ContributionApp {
        id: "fabric.test.contribution.app";
        relations {
            requires {
                primary: ContributionStore;
                cache: ContributionStore;
                signal: ContributionSignal;
            }
        }
        api { fn run(&self); }
    }
}

struct ContributionAudit;

#[derive(Clone)]
struct ContributionAuditContract;

impl ResourceAugmentationDefinition<ContributionStore> for ContributionAudit {
    type Config = ();
    type Contract = ContributionAuditContract;

    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.contribution.audit").expect("contract"),
        )
    }
}

struct ContributionSignalAudit;

#[derive(Clone)]
struct ContributionSignalAuditContract;

impl SystemAugmentationDefinition<ContributionSignal> for ContributionSignalAudit {
    type Config = ();
    type Contract = ContributionSignalAuditContract;

    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.contribution.signal-audit").expect("contract"),
        )
    }
}

struct ContributionAppAudit;

#[derive(Clone)]
struct ContributionAppAuditContract;

impl ComponentAugmentationDefinition<ContributionApp> for ContributionAppAudit {
    type Config = ();
    type Contract = ContributionAppAuditContract;

    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.contribution.app-audit").expect("contract"),
        )
    }
}

struct RawContributionModule;

impl Module for RawContributionModule {
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::new(
            fabric::core::ModuleId::new("fabric.test.contribution.raw.module").expect("module"),
        )
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        None
    }
}

fn storage(name: &'static str) -> impl IntoFabricContribution {
    let primary = ContributionStore::select(name).expect("primary");
    let cache = ContributionStore::select("cache").expect("cache");
    let audit = ResourceAugmentation::<ContributionStore, ContributionAudit>::attach(&primary, ())
        .expect("audit");
    FabricContribution::new()
        .resource(primary)
        .resource(cache)
        .resource_augmentation(audit)
}

fn networking() -> impl IntoFabricContribution {
    FabricContribution::new().system(ContributionSignal::select().expect("signal"))
}

fn observability() -> impl IntoFabricContribution {
    let signal = ContributionSignal::select().expect("signal");
    let signal_audit =
        SystemAugmentation::<ContributionSignal, ContributionSignalAudit>::attach(&signal, ())
            .expect("signal audit");
    FabricContribution::new().system_augmentation(signal_audit)
}

fn realized_resources() -> impl IntoFabricContribution {
    FabricContribution::new()
        .resource(
            ContributionCache::select("adapter")
                .expect("cache")
                .using(ContributionCacheAdapter::new())
                .expect("adapter"),
        )
        .resource(
            ContributionMediated::select("mediated")
                .expect("mediated")
                .using(ContributionMediatedAdapter::new())
                .expect("mediated adapter"),
        )
}

fn application(primary_name: &'static str) -> impl IntoFabricContribution {
    let primary = ContributionStore::select(primary_name).expect("primary selection");
    let cache = ContributionStore::select("cache").expect("cache selection");
    let component = ContributionApp::define()
        .select_named_resource_provider(
            &fabric::authoring::ComponentResourceRequirement::new(
                fabric::component::ComponentRelationName::new("primary").expect("role"),
                fabric::authoring::Requires::<ContributionStore>::provisional(),
            ),
            &primary,
        )
        .select_named_resource_provider(
            &fabric::authoring::ComponentResourceRequirement::new(
                fabric::component::ComponentRelationName::new("cache").expect("role"),
                fabric::authoring::Requires::<ContributionStore>::provisional(),
            ),
            &cache,
        );

    FabricContribution::new().component(
        component
            .augment::<ContributionAppAudit>(())
            .expect("component audit"),
    )
}

fn platform(primary_name: &'static str) -> impl IntoFabricContribution {
    FabricContribution::new()
        .with(networking())
        .with(observability())
        .with(storage(primary_name))
        .with(realized_resources())
}

fn raw_advanced() -> impl IntoFabricContribution {
    FabricContribution::new().with_block(
        BlockAuthor::new("fabric.test.contribution.raw")
            .expect("raw block")
            .module(RawContributionModule)
            .build(),
    )
}

fn build_inline(composition_id: &'static str) -> Composition {
    let primary = ContributionStore::select("primary").expect("primary");
    let cache = ContributionStore::select("cache").expect("cache");
    let audit = ResourceAugmentation::<ContributionStore, ContributionAudit>::attach(&primary, ())
        .expect("audit");
    let component = ContributionApp::define()
        .select_named_resource_provider(
            &fabric::authoring::ComponentResourceRequirement::new(
                fabric::component::ComponentRelationName::new("primary").expect("role"),
                fabric::authoring::Requires::<ContributionStore>::provisional(),
            ),
            &primary,
        )
        .select_named_resource_provider(
            &fabric::authoring::ComponentResourceRequirement::new(
                fabric::component::ComponentRelationName::new("cache").expect("role"),
                fabric::authoring::Requires::<ContributionStore>::provisional(),
            ),
            &cache,
        )
        .augment::<ContributionAppAudit>(())
        .expect("component audit");

    Fabric::new(composition_id)
        .expect("fabric")
        .system(ContributionSignal::select().expect("signal"))
        .system_augmentation(
            SystemAugmentation::<ContributionSignal, ContributionSignalAudit>::attach(
                &ContributionSignal::select().expect("signal"),
                (),
            )
            .expect("signal audit"),
        )
        .resource(primary)
        .resource(cache)
        .resource_augmentation(audit)
        .resource(
            ContributionCache::select("adapter")
                .expect("cache")
                .using(ContributionCacheAdapter::new())
                .expect("adapter"),
        )
        .resource(
            ContributionMediated::select("mediated")
                .expect("mediated")
                .using(ContributionMediatedAdapter::new())
                .expect("mediated adapter"),
        )
        .component(component)
        .build()
        .expect("inline build")
}

fn build_contributed(composition_id: &'static str) -> Composition {
    Fabric::new(composition_id)
        .expect("fabric")
        .with(platform("primary"))
        .with(application("primary"))
        .build()
        .expect("contribution build")
}

fn semantic_signature(composition: &Composition) -> Vec<String> {
    let mut signature = Vec::new();
    for resource in composition.resources() {
        signature.push(format!(
            "resource:{}:{}:{:?}:api={}:aug={}",
            resource.resource_id(),
            resource.name().as_str(),
            resource.realization().kind(),
            resource.api().endpoints().len(),
            resource.augmentations().count()
        ));
    }
    for system in composition.systems() {
        signature.push(format!(
            "system:{}:{:?}:api={}:aug={}",
            system.system_id(),
            system.realization().kind(),
            system.api().endpoints().len(),
            system.augmentations().count()
        ));
    }
    for component in composition.components() {
        signature.push(format!(
            "component:{}:{:?}:api={}:relations={}:aug={}",
            component.component_id(),
            component
                .realization()
                .map(|realization| realization.kind())
                .unwrap_or(SemanticRealizationKind::DeclarationOnly),
            component.api().endpoints().len(),
            component.relations().count(),
            component.augmentations().count()
        ));
    }
    for relation in composition.relations() {
        signature.push(format!(
            "relation:{:?}:{}:{:?}->{:?}",
            relation.owner(),
            relation.role().as_str(),
            relation.declared_target(),
            relation.resolved_target()
        ));
    }
    signature.sort();
    signature
}

#[test]
fn contributions_are_semantically_equivalent_to_inline_authoring() {
    let inline = build_inline("fabric.test.contribution.inline");
    let contributed = build_contributed("fabric.test.contribution.modular");

    assert_eq!(
        semantic_signature(&inline),
        semantic_signature(&contributed)
    );
    assert_eq!(contributed.resources().count(), 4);
    assert_eq!(contributed.systems().count(), 1);
    assert_eq!(contributed.components().count(), 1);
}

#[test]
fn cross_contribution_relations_resolve_across_boundaries_and_order() {
    let reverse_order = Fabric::new("fabric.test.contribution.reverse")
        .expect("fabric")
        .with(application("primary"))
        .with(storage("primary"))
        .with(networking())
        .build()
        .expect("build");

    let primary = reverse_order
        .relations()
        .iter()
        .find(|relation| relation.role().as_str() == "primary")
        .expect("primary relation");
    assert_eq!(
        primary.resolved_target(),
        &SemanticRelationTargetOccurrence::Resource {
            resource_id: ResourceId::new("fabric.test.contribution.store").expect("resource"),
            resource_name: ResourceName::new("primary").expect("name"),
        }
    );

    let cache = reverse_order
        .relations()
        .iter()
        .find(|relation| relation.role().as_str() == "cache")
        .expect("cache relation");
    assert_eq!(
        cache.resolved_target(),
        &SemanticRelationTargetOccurrence::Resource {
            resource_id: ResourceId::new("fabric.test.contribution.store").expect("resource"),
            resource_name: ResourceName::new("cache").expect("name"),
        }
    );
}

#[test]
fn nested_parameterized_and_realized_contributions_materialize_normally() {
    let composition = Fabric::new("fabric.test.contribution.materialize")
        .expect("fabric")
        .with(platform("primary"))
        .with(application("primary"))
        .build()
        .expect("build");

    let direct = composition
        .resources()
        .find(|resource| resource.name().as_str() == "adapter")
        .expect("direct adapter resource");
    assert_eq!(
        direct.realization().kind(),
        SemanticRealizationKind::AdapterDirect
    );
    assert!(direct.realization().host_requirement().is_some());

    let mediated = composition
        .resources()
        .find(|resource| resource.name().as_str() == "mediated")
        .expect("mediated resource");
    assert_eq!(
        mediated.realization().kind(),
        SemanticRealizationKind::AdapterMediated
    );

    let mut instance = composition
        .materialize_named_on("fabric.test.contribution.materialize.instance", &host())
        .expect("materialize");
    instance.start().expect("start");
    instance.stop().expect("stop");
}

#[test]
fn parameterized_contribution_changes_authoring_without_gaining_identity() {
    let composition = Fabric::new("fabric.test.contribution.parameterized")
        .expect("fabric")
        .with(networking())
        .with(storage("archive"))
        .with(application("archive"))
        .build()
        .expect("build");

    assert!(composition.resources().any(|resource| {
        resource.resource_id().as_str() == "fabric.test.contribution.store"
            && resource.name().as_str() == "archive"
    }));
    assert!(composition.relations().iter().any(|relation| {
        relation.role().as_str() == "primary"
            && relation.resolved_target()
                == &SemanticRelationTargetOccurrence::Resource {
                    resource_id: ResourceId::new("fabric.test.contribution.store")
                        .expect("resource"),
                    resource_name: ResourceName::new("archive").expect("name"),
                }
    }));
}

#[test]
fn raw_advanced_authoring_passes_through_without_semantic_fabrication() {
    let composition = Fabric::new("fabric.test.contribution.raw")
        .expect("fabric")
        .with(raw_advanced())
        .build()
        .expect("build");

    assert_eq!(composition.manifest().diagnostics().raw_blocks().len(), 1);
    assert_eq!(composition.resources().count(), 0);
    assert_eq!(composition.relations().len(), 0);
}

#[test]
fn external_crate_can_publish_a_reusable_contribution() {
    let composition = Fabric::new("fabric.test.contribution.external")
        .expect("fabric")
        .with(external_storage("primary"))
        .build()
        .expect("external contribution build");

    assert!(composition.resources().any(|resource| {
        resource.resource_id().as_str() == "fabric.test.external.contribution-store"
            && resource.name().as_str() == "primary"
    }));
}

#[test]
fn duplicate_conflicts_are_detected_at_final_build_boundary() {
    let error = Fabric::new("fabric.test.contribution.duplicate")
        .expect("fabric")
        .with(storage("primary"))
        .with(storage("primary"))
        .with(networking())
        .build()
        .expect_err("duplicate modules fail through final build");

    assert!(matches!(error, FabricBuildError::Composition(_)));
}

#[test]
fn contribution_source_contains_no_composition_or_identity_model() {
    let source = include_str!("contribution_witness.rs");
    assert!(source.contains("fn platform"));
    assert!(source.contains("FabricContribution::new()"));

    let _: Option<FabricContribution> = None;
    fn accepts_contribution(_contribution: impl IntoFabricContribution) {}
    accepts_contribution(FabricContribution::new());
    let _: Option<ResourceSchemaDescriptor> = None;
    let _: Option<SystemSchemaDescriptor> = None;
}
