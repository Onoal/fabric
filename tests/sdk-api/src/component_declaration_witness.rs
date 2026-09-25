use fabric::*;

#[derive(Clone)]
struct ExternalComponentConfig {
    label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Greeting(String);

#[derive(Clone, Debug, PartialEq, Eq)]
struct GreetingReply(String);

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeclarationError;

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeResult(String);

#[derive(Default)]
struct RuntimeState(std::sync::atomic::AtomicUsize);

#[derive(Clone)]
struct RuntimeLifecycleConfig {
    teardown_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

#[derive(Default)]
struct RuntimeLifecycleState(std::sync::atomic::AtomicUsize);

#[derive(Clone)]
struct AdapterPreparationConfig(std::sync::Arc<std::sync::atomic::AtomicUsize>);

#[derive(Clone)]
struct AdapterParticipationConfig {
    state_allocations: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    prepares: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    teardowns: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

#[derive(Default)]
struct AdapterParticipationState(std::sync::atomic::AtomicUsize);

#[derive(Clone)]
struct ReplacementComponentConfig {
    self_prepares: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    self_teardowns: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

#[derive(Clone)]
struct ReplacementAdapterConfig {
    adapter_prepares: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    adapter_teardowns: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

resource! {
    DeclarationStore {
        id: "fabric.test.component-declaration.store";
        api { fn get(&self, key: String) -> Option<String>; }
    }
}

system! {
    DeclarationClock {
        id: "fabric.test.component-declaration.clock";
        api { fn now(&self) -> u64; }
    }
}

resource! {
    CanonicalRuntimeStore {
        id: "fabric.test.component-declaration.runtime-store";
        api { fn get(&self, key: String) -> Option<String>; }
        runtime { fn get(&self, key: String) -> Option<String> { Some(format!("store:{key}")) } }
    }
}

system! {
    CanonicalRuntimeClock {
        id: "fabric.test.component-declaration.runtime-clock";
        api { fn now(&self) -> u64; }
        runtime { fn now(&self) -> u64 { 7 } }
    }
}

resource! {
    VersionedDeclarationStore {
        id: "fabric.test.component-declaration.versioned-store";
        version: "1.2.0";
        api { fn get(&self, key: String) -> Option<String>; }
    }
}

component! {
    EmptyDeclaration {
        id: "fabric.test.component-declaration.empty";
    }
}

component! {
    AdapterRealizedDeclaration {
        id: "fabric.test.component-declaration.adapter-realized";
        config { prefix: String; }
        api { fn greet(&self, name: String) -> String; }
    }
}

adapter! {
    AdapterRealizedDeclarationMemory for AdapterRealizedDeclaration {
        id: "test.adapter-realized-declaration-memory";
        config { suffix: String; }
        runtime {
            state { RuntimeState = RuntimeState::default(); }
            fn greet(&self, name: String) -> String {
                self.state().get().0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                format!("{}:{name}:{}", self.component_config().prefix, self.config().suffix)
            }
            prepare { Ok(()) }
            teardown { Ok(()) }
        }
    }
}

component! {
    AdapterParticipationAudit {
        id: "fabric.test.component-declaration.adapter-participation-audit";
        config { prefix: String; }
        relations {
            requires {
                component_store: CanonicalRuntimeStore;
                component_clock: CanonicalRuntimeClock;
            }
        }
        api { fn inspect(&self, key: String) -> String; }
    }
}

adapter! {
    AdapterParticipationAuditRuntime for AdapterParticipationAudit {
        id: "test.adapter-participation-audit-runtime";
        config: AdapterParticipationConfig;
        relations {
            requires {
                adapter_store: CanonicalRuntimeStore;
                adapter_clock: CanonicalRuntimeClock;
            }
        }
        runtime {
            state {
                AdapterParticipationState = {
                    config
                        .state_allocations
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    AdapterParticipationState::default()
                };
            }
            fn inspect(&self, key: String) -> String {
                let count = self
                    .state()
                    .get()
                    .0
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    + 1;
                format!(
                    "{}:{}:{}:{}:{}:{count}",
                    self.component_config().prefix,
                    self.component_relations()
                        .component_store
                        .get(key.clone())
                        .expect("component relation"),
                    self.component_relations().component_clock.now(),
                    self.relations()
                        .adapter_store
                        .get(key)
                        .expect("adapter relation"),
                    self.relations().adapter_clock.now(),
                )
            }
            prepare {
                self.config()
                    .prepares
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
            teardown {
                self.config()
                    .teardowns
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        }
    }
}

component! {
    ReplacementAudit {
        id: "fabric.test.component-declaration.replacement-audit";
        config: ReplacementComponentConfig;
        runtime {
            prepare {
                self.config()
                    .self_prepares
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
            teardown {
                self.config()
                    .self_teardowns
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        }
    }
}

adapter! {
    ReplacementAuditAdapter for ReplacementAudit {
        id: "test.replacement-audit-adapter";
        config: ReplacementAdapterConfig;
        runtime {
            prepare {
                self.config()
                    .adapter_prepares
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
            teardown {
                self.config()
                    .adapter_teardowns
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        }
    }
}

mod imported_component_target {
    use fabric::*;

    component! {
        pub ImportedTarget {
            id: "fabric.test.component-declaration.imported-target";
            api { fn source(&self) -> &'static str; }
        }
    }
}

mod component_target_facade {
    pub use super::imported_component_target::ImportedTarget as ReexportedTarget;
}

use component_target_facade::ReexportedTarget as ImportedComponentTarget;

adapter! {
    ImportedComponentTargetAdapter for ImportedComponentTarget {
        id: "test.imported-component-target-adapter";
        runtime { fn source(&self) -> &'static str { "imported" } }
    }
}

adapter! {
    UnsupportedComponentAdapterSupport for AdapterPreparedAutonomous {
        id: "test.unsupported-component-adapter-support";
        supports: provisional;
        runtime { prepare { Ok(()) } }
    }
}

component! {
    AdapterPreparedAutonomous {
        id: "fabric.test.component-declaration.adapter-prepared-autonomous";
    }
}

adapter! {
    AdapterPreparedAutonomousRuntime for AdapterPreparedAutonomous {
        id: "test.adapter-prepared-autonomous-runtime";
        config: AdapterPreparationConfig;
        runtime {
            prepare {
                self.config().0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
            teardown { Ok(()) }
        }
    }
}

component! {
    DefaultOrAdapter {
        id: "fabric.test.component-declaration.default-or-adapter";
        api { fn source(&self) -> &'static str; }
        runtime { fn source(&self) -> &'static str { "self" } }
    }
}

adapter! {
    DefaultOrAdapterExternal for DefaultOrAdapter {
        id: "test.default-or-adapter-external";
        runtime { fn source(&self) -> &'static str { "adapter" } }
    }
}

component! {
    CanonicalLifecycleRuntime {
        id: "fabric.test.component-declaration.lifecycle-runtime";
        config: RuntimeLifecycleConfig;
        api { fn next(&self) -> usize; }
        runtime {
            state { RuntimeLifecycleState = RuntimeLifecycleState::default(); }
            fn next(&self) -> usize {
                self.state().get().0.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
            }
            prepare { Ok(()) }
            teardown {
                self.config().teardown_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        }
    }
}

component! {
    CanonicalRelatedRuntime {
        id: "fabric.test.component-declaration.related-runtime";
        relations {
            requires {
                primary: CanonicalRuntimeStore;
                cache: CanonicalRuntimeStore;
                clock: CanonicalRuntimeClock;
            }
        }
        api { fn inspect(&self, key: String) -> String; }
        runtime {
            fn inspect(&self, key: String) -> String {
                let primary = self.relations().primary.get(key.clone()).expect("primary");
                let cache = self.relations().cache.get(key).expect("cache");
                format!("{primary}:{cache}:{}", self.relations().clock.now())
            }
        }
    }
}

component! {
    CanonicalRuntimeDeclaration {
        id: "fabric.test.component-declaration.canonical-runtime";
        config { prefix: String; }
        api {
            fn ping(&self);
            fn greet(&self, input: String) -> RuntimeResult;
            fn join(&self, left: String, right: String) -> String;
            fn domain(&self, input: String) -> Result<String, DeclarationError>;
        }
        runtime {
            state { RuntimeState = RuntimeState::default(); }
            fn ping(&self) {
                self.state().get().0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            fn greet(&self, input: String) -> RuntimeResult {
                RuntimeResult(format!("{}:{input}", self.config().prefix))
            }
            fn join(&self, left: String, right: String) -> String { format!("{left}{right}") }
            fn domain(&self, input: String) -> Result<String, DeclarationError> { Ok(input) }
            prepare { Ok(()) }
            teardown { Ok(()) }
        }
    }
}

component! {
    CompatibleRelatedDeclaration {
        id: "fabric.test.component-declaration.compatible-related";
        relations {
            requires {
                store: VersionedDeclarationStore(version = "^1");
            }
        }
    }
}

component! {
    ConfiguredDeclaration {
        id: "fabric.test.component-declaration.configured";
        config { label: String; }
    }
}

component! {
    ExternalConfiguredDeclaration {
        id: "fabric.test.component-declaration.external-configured";
        config: ExternalComponentConfig;
    }
}

component! {
    RelatedDeclaration {
        id: "fabric.test.component-declaration.related";
        relations {
            requires {
                primary: DeclarationStore;
                cache: DeclarationStore;
                clock: DeclarationClock;
            }
        }
    }
}

component! {
    ApiOnlyDeclaration {
        id: "fabric.test.component-declaration.api-only";
        api {
            fn ping(&self);
            fn revision(&self) -> u64;
            fn greet(&self, input: Greeting) -> GreetingReply;
            fn pair(&self, first: String, second: String) -> String;
            fn fallible(&self, input: String) -> Result<String, DeclarationError>;
        }
    }
}

component! {
    CompleteDeclaration {
        id: "fabric.test.component-declaration.complete";
        config { label: String; }
        relations {
            requires {
                store: DeclarationStore;
                clock: DeclarationClock;
            }
        }
        api { fn inspect(&self, input: String) -> Result<String, DeclarationError>; }
    }
}

#[test]
fn canonical_component_declarations_are_runtime_free_and_incremental() {
    let empty = EmptyDeclaration::define();
    assert_eq!(empty.declaration().operations(), []);
    assert!(empty.declaration().relations().is_empty());

    let inline = ConfiguredDeclaration::define(ConfiguredDeclarationConfig {
        label: "inline".to_owned(),
    });
    assert_eq!(inline.config().label, "inline");

    let external = ExternalConfiguredDeclaration::define(ExternalComponentConfig {
        label: "external".to_owned(),
    });
    assert_eq!(external.config().label, "external");
}

#[test]
fn canonical_component_relations_are_generic_and_roles_remain_distinct() {
    let declaration = RelatedDeclaration::define().declaration().clone();
    assert_eq!(declaration.relations().len(), 3);
    assert_eq!(declaration.relations()[0].name().as_str(), "primary");
    assert_eq!(declaration.relations()[1].name().as_str(), "cache");
    assert_ne!(
        declaration.relations()[0].name(),
        declaration.relations()[1].name(),
        "two local roles may target the same semantic Resource"
    );
    assert_eq!(
        declaration.relations()[0].requirement().id(),
        declaration.relations()[1].requirement().id()
    );

    let compatible = CompatibleRelatedDeclaration::define().declaration().clone();
    assert_eq!(compatible.relations().len(), 1);
    assert_eq!(
        compatible.relations()[0]
            .requirement()
            .compatibility()
            .to_string(),
        "^1"
    );
}

#[test]
fn canonical_component_api_lowers_to_deterministic_operation_metadata_without_handlers() {
    let declaration = ApiOnlyDeclaration::define().declaration().clone();
    assert_eq!(declaration.operations().len(), 5);
    let ping = api_only_declaration::api::ping();
    let _: fabric::component::OperationKey<(), ()> = ping;
    let revision = api_only_declaration::api::revision();
    let _: fabric::component::OperationKey<(), u64> = revision;
    let greet = api_only_declaration::api::greet();
    assert_eq!(
        greet.id().as_str(),
        "fabric.test.component-declaration.api-only.api.greet"
    );
    assert_eq!(
        greet.input_type().as_str(),
        "fabric.test.component-declaration.api-only.api.greet.input"
    );
    assert_eq!(
        greet.output_type().as_str(),
        "fabric.test.component-declaration.api-only.api.greet.output"
    );
    let pair = api_only_declaration::api::pair();
    let _: fabric::component::OperationKey<(String, String), String> = pair;
    let fallible = api_only_declaration::api::fallible();
    let _: fabric::component::OperationKey<String, Result<String, DeclarationError>> = fallible;
}

#[test]
fn canonical_component_declaration_axes_compose_without_runtime_attachment() {
    let complete = CompleteDeclaration::define(CompleteDeclarationConfig {
        label: "complete".to_owned(),
    });
    assert_eq!(complete.config().label, "complete");
    assert_eq!(complete.declaration().relations().len(), 2);
    assert_eq!(complete.declaration().operations().len(), 1);
    assert!(complete.into_self_realization().is_none());
}

#[test]
fn canonical_component_api_declaration_has_no_local_runtime_attachment() {
    let built = Fabric::new("fabric.test.component-declaration.runtime-free")
        .expect("fabric")
        .component(ApiOnlyDeclaration::define())
        .build()
        .expect("declaration-only component builds");
    let mut instance = built
        .materialize("fabric.test.component-declaration.runtime-free.instance")
        .expect("host-known declaration materializes");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    assert!(matches!(
        components.materialize::<ApiOnlyDeclaration>(),
        Err(fabric::component::ComponentError::MissingComponentParticipationRealization(_))
    ));
    instance.stop().expect("stop");
}

#[test]
fn canonical_component_runtime_registers_semantic_api_without_operation_ceremony() {
    let built = Fabric::new("fabric.test.component-declaration.canonical-runtime")
        .expect("fabric")
        .component(CanonicalRuntimeDeclaration::define(
            CanonicalRuntimeDeclarationConfig {
                prefix: "hello".to_owned(),
            },
        ))
        .build()
        .expect("build");
    let mut instance = built
        .materialize("fabric.test.component-declaration.canonical-runtime.instance")
        .expect("instance");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<CanonicalRuntimeDeclaration>()
        .expect("canonical self realization");
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &canonical_runtime_declaration::api::greet(),
            "Ada".to_owned(),
        ))
        .expect("invoke"),
        RuntimeResult("hello:Ada".to_owned())
    );
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &canonical_runtime_declaration::api::join(),
            ("a".to_owned(), "b".to_owned()),
        ))
        .expect("invoke"),
        "ab"
    );
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &canonical_runtime_declaration::api::domain(),
            "ok".to_owned(),
        ))
        .expect("Fabric invocation"),
        Ok("ok".to_owned())
    );
    instance.stop().expect("stop");
}

#[test]
fn canonical_component_runtime_receives_typed_resource_and_system_relations() {
    let store = CanonicalRuntimeStore::select("primary").expect("store");
    let clock = CanonicalRuntimeClock::select().expect("clock");
    let built = Fabric::new("fabric.test.component-declaration.related-runtime")
        .expect("fabric")
        .resource(store)
        .system(clock)
        .component(CanonicalRelatedRuntime::define())
        .build()
        .expect("build with target-driven relation carriers");
    let mut instance = built
        .materialize("fabric.test.component-declaration.related-runtime.instance")
        .expect("instance");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<CanonicalRelatedRuntime>()
        .expect("materialize canonical relation runtime");
    assert_eq!(
        futures::executor::block_on(
            components
                .invoke_external(&canonical_related_runtime::api::inspect(), "key".to_owned(),)
        )
        .expect("invoke"),
        "store:key:store:key:7"
    );
    instance.stop().expect("stop");
}

#[test]
fn canonical_component_runtime_state_is_participation_local_and_tears_down_once() {
    let teardowns = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let built = Fabric::new("fabric.test.component-declaration.lifecycle-runtime")
        .expect("fabric")
        .component(CanonicalLifecycleRuntime::define(RuntimeLifecycleConfig {
            teardown_count: std::sync::Arc::clone(&teardowns),
        }))
        .build()
        .expect("build");
    let mut instance = built
        .materialize("fabric.test.component-declaration.lifecycle-runtime.instance")
        .expect("instance");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<CanonicalLifecycleRuntime>()
        .expect("first participation");
    assert_eq!(
        futures::executor::block_on(
            components.invoke_external(&canonical_lifecycle_runtime::api::next(), (),)
        )
        .expect("first next"),
        1
    );
    components
        .dematerialize::<CanonicalLifecycleRuntime>()
        .expect("dematerialize");
    assert_eq!(teardowns.load(std::sync::atomic::Ordering::SeqCst), 1);
    components
        .materialize::<CanonicalLifecycleRuntime>()
        .expect("fresh participation");
    assert_eq!(
        futures::executor::block_on(
            components.invoke_external(&canonical_lifecycle_runtime::api::next(), (),)
        )
        .expect("fresh next"),
        1,
        "state is fresh for a new participation"
    );
    instance.stop().expect("host stop");
    assert_eq!(teardowns.load(std::sync::atomic::Ordering::SeqCst), 2);
}

#[test]
fn canonical_component_adapter_realizes_a_declaration_through_participation() {
    let selected = AdapterRealizedDeclaration::define(AdapterRealizedDeclarationConfig {
        prefix: "component".to_owned(),
    })
    .using(AdapterRealizedDeclarationMemory::new(
        AdapterRealizedDeclarationMemoryConfig {
            suffix: "adapter".to_owned(),
        },
    ))
    .expect("canonical ComponentInstanceBinding adapter selection");
    let built = Fabric::new("fabric.test.component-declaration.adapter-realized")
        .expect("fabric")
        .component(selected)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_on(
            "fabric.test.component-declaration.adapter-realized.instance",
            &HostDescriptor::native(),
        )
        .expect("instance");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<AdapterRealizedDeclaration>()
        .expect("adapter realization");
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &adapter_realized_declaration::api::greet(),
            "Ada".to_owned(),
        ))
        .expect("invoke"),
        "component:Ada:adapter"
    );
    instance.stop().expect("stop");
}

#[test]
fn explicit_component_adapter_replaces_the_default_self_realization() {
    let selected = DefaultOrAdapter::define()
        .using(DefaultOrAdapterExternal::new())
        .expect("explicit Adapter replaces the default");
    let built = Fabric::new("fabric.test.component-declaration.default-or-adapter")
        .expect("fabric")
        .component(selected)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_on(
            "fabric.test.component-declaration.default-or-adapter.instance",
            &HostDescriptor::native(),
        )
        .expect("instance");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<DefaultOrAdapter>()
        .expect("selected Adapter realization");
    assert_eq!(
        futures::executor::block_on(
            components.invoke_external(&default_or_adapter::api::source(), (),)
        )
        .expect("invoke"),
        "adapter"
    );
    instance.stop().expect("stop");
}

#[test]
fn api_less_component_adapter_can_prepare_one_participation() {
    let prepared = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let selected = AdapterPreparedAutonomous::define()
        .using(AdapterPreparedAutonomousRuntime::new(
            AdapterPreparationConfig(std::sync::Arc::clone(&prepared)),
        ))
        .expect("select autonomous Adapter realization");
    let built = Fabric::new("fabric.test.component-declaration.adapter-prepared-autonomous")
        .expect("fabric")
        .component(selected)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_on(
            "fabric.test.component-declaration.adapter-prepared-autonomous.instance",
            &HostDescriptor::native(),
        )
        .expect("instance");
    instance.start().expect("start");
    instance
        .components()
        .expect("component host")
        .materialize::<AdapterPreparedAutonomous>()
        .expect("prepare autonomous participation");
    assert_eq!(prepared.load(std::sync::atomic::Ordering::SeqCst), 1);
    instance.stop().expect("stop");
}

#[test]
fn component_adapter_state_relations_and_cleanup_are_participation_owned() {
    let state_allocations = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let prepares = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let teardowns = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let selected = AdapterParticipationAudit::define(AdapterParticipationAuditConfig {
        prefix: "component".to_owned(),
    })
    .using(AdapterParticipationAuditRuntime::new(
        AdapterParticipationConfig {
            state_allocations: std::sync::Arc::clone(&state_allocations),
            prepares: std::sync::Arc::clone(&prepares),
            teardowns: std::sync::Arc::clone(&teardowns),
        },
    ))
    .expect("select adapter realization");
    let built = Fabric::new("fabric.test.component-declaration.adapter-participation-audit")
        .expect("fabric")
        .resource(CanonicalRuntimeStore::select("component_store").expect("store"))
        .system(CanonicalRuntimeClock::select().expect("clock"))
        .component(selected)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_on(
            "fabric.test.component-declaration.adapter-participation-audit.instance",
            &HostDescriptor::native(),
        )
        .expect("instance");
    instance.start().expect("start");
    assert_eq!(
        state_allocations.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "Adapter provider materialization must not allocate participation state"
    );
    let components = instance.components().expect("component host");
    components
        .materialize::<AdapterParticipationAudit>()
        .expect("first participation");
    assert_eq!(
        state_allocations.load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    assert_eq!(prepares.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &adapter_participation_audit::api::inspect(),
            "key".to_owned(),
        ))
        .expect("invoke"),
        "component:store:key:7:store:key:7:1"
    );
    components
        .dematerialize::<AdapterParticipationAudit>()
        .expect("dematerialize");
    assert_eq!(teardowns.load(std::sync::atomic::Ordering::SeqCst), 1);
    components
        .materialize::<AdapterParticipationAudit>()
        .expect("fresh participation");
    assert_eq!(
        state_allocations.load(std::sync::atomic::Ordering::SeqCst),
        2
    );
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &adapter_participation_audit::api::inspect(),
            "key".to_owned(),
        ))
        .expect("fresh invoke"),
        "component:store:key:7:store:key:7:1",
        "state is fresh for a new ComponentParticipation"
    );
    instance.stop().expect("host stop");
    assert_eq!(teardowns.load(std::sync::atomic::Ordering::SeqCst), 2);
}

#[test]
fn explicit_component_adapter_replaces_self_prepare_and_teardown() {
    let self_prepares = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let self_teardowns = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let adapter_prepares = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let adapter_teardowns = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let selected = ReplacementAudit::define(ReplacementComponentConfig {
        self_prepares: std::sync::Arc::clone(&self_prepares),
        self_teardowns: std::sync::Arc::clone(&self_teardowns),
    })
    .using(ReplacementAuditAdapter::new(ReplacementAdapterConfig {
        adapter_prepares: std::sync::Arc::clone(&adapter_prepares),
        adapter_teardowns: std::sync::Arc::clone(&adapter_teardowns),
    }))
    .expect("Adapter replaces default self realization");
    let built = Fabric::new("fabric.test.component-declaration.replacement-audit")
        .expect("fabric")
        .component(selected)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_on(
            "fabric.test.component-declaration.replacement-audit.instance",
            &HostDescriptor::native(),
        )
        .expect("instance");
    instance.start().expect("start");
    instance
        .components()
        .expect("component host")
        .materialize::<ReplacementAudit>()
        .expect("adapter participation");
    assert_eq!(self_prepares.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert_eq!(
        adapter_prepares.load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    instance.stop().expect("stop");
    assert_eq!(self_teardowns.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert_eq!(
        adapter_teardowns.load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

#[test]
fn component_adapter_target_resolution_uses_the_imported_type() {
    let selected = ImportedComponentTarget::define()
        .using(ImportedComponentTargetAdapter::new())
        .expect("imported target adapter");
    let built = Fabric::new("fabric.test.component-declaration.imported-target")
        .expect("fabric")
        .component(selected)
        .build()
        .expect("build");
    let mut instance = built
        .materialize_on(
            "fabric.test.component-declaration.imported-target.instance",
            &HostDescriptor::native(),
        )
        .expect("instance");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<ImportedComponentTarget>()
        .expect("adapter participation");
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &imported_component_target::imported_target::api::source(),
            (),
        ))
        .expect("invoke"),
        "imported"
    );
    instance.stop().expect("stop");
}

#[test]
fn component_adapter_rejects_schema_style_support_overrides() {
    let error = match AdapterPreparedAutonomous::define()
        .using(UnsupportedComponentAdapterSupport::new())
    {
        Ok(_) => panic!("ComponentInstanceBinding Adapter support must be target-derived"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        fabric::component::ComponentError::UnsupportedComponentAdapterSupportOverride
    ));
    assert!(error.to_string().contains("`supports:` is not valid"));
}
