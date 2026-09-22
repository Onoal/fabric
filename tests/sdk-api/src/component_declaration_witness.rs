use fabric::*;

#[derive(Clone)]
struct ExternalComponentConfig {
    label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Greeting(String);

#[derive(Clone, Debug, PartialEq, Eq)]
struct GreetingReply(String);

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

component! {
    EmptyDeclaration {
        id: "fabric.test.component-declaration.empty";
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
            fn greet(&self, input: Greeting) -> GreetingReply;
            fn pair(&self, first: String, second: String) -> String;
        }
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
}

#[test]
fn canonical_component_api_lowers_to_deterministic_operation_metadata_without_handlers() {
    let declaration = ApiOnlyDeclaration::define().declaration().clone();
    assert_eq!(declaration.operations().len(), 2);
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
