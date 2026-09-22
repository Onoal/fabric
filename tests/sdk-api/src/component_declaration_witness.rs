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
    let ping = api_only_declaration::operations::ping();
    let _: fabric::component::OperationKey<(), ()> = ping;
    let revision = api_only_declaration::operations::revision();
    let _: fabric::component::OperationKey<(), u64> = revision;
    let greet = api_only_declaration::operations::greet();
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
    let pair = api_only_declaration::operations::pair();
    let _: fabric::component::OperationKey<(String, String), String> = pair;
    let fallible = api_only_declaration::operations::fallible();
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
        .materialize_named("fabric.test.component-declaration.runtime-free.instance")
        .expect("host-known declaration materializes");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    assert!(matches!(
        components.materialize::<ApiOnlyDeclaration>(),
        Err(fabric::component::ComponentError::MissingComponentRuntimeAttachment(_))
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
        .materialize_named("fabric.test.component-declaration.canonical-runtime.instance")
        .expect("instance");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<CanonicalRuntimeDeclaration>()
        .expect("canonical self realization");
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &canonical_runtime_declaration::operations::greet(),
            "Ada".to_owned(),
        ))
        .expect("invoke"),
        RuntimeResult("hello:Ada".to_owned())
    );
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &canonical_runtime_declaration::operations::join(),
            ("a".to_owned(), "b".to_owned()),
        ))
        .expect("invoke"),
        "ab"
    );
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &canonical_runtime_declaration::operations::domain(),
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
        .materialize_named("fabric.test.component-declaration.related-runtime.instance")
        .expect("instance");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<CanonicalRelatedRuntime>()
        .expect("materialize canonical relation runtime");
    assert_eq!(
        futures::executor::block_on(components.invoke_external(
            &canonical_related_runtime::operations::inspect(),
            "key".to_owned(),
        ))
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
        .materialize_named("fabric.test.component-declaration.lifecycle-runtime.instance")
        .expect("instance");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<CanonicalLifecycleRuntime>()
        .expect("first participation");
    assert_eq!(
        futures::executor::block_on(
            components.invoke_external(&canonical_lifecycle_runtime::operations::next(), (),)
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
            components.invoke_external(&canonical_lifecycle_runtime::operations::next(), (),)
        )
        .expect("fresh next"),
        1,
        "state is fresh for a new participation"
    );
    instance.stop().expect("host stop");
    assert_eq!(teardowns.load(std::sync::atomic::Ordering::SeqCst), 2);
}
