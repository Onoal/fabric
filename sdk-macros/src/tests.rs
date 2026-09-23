use std::fs;
use std::path::Path;

fn crate_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn macro_manifest_is_proc_macro_and_runtime_free() {
    let manifest = fs::read_to_string(crate_root().join("Cargo.toml")).expect("read manifest");

    assert!(manifest.contains("proc-macro = true"));
    for forbidden in [
        "fabric-core",
        "fabric-host",
        "fabric-resource",
        "fabric-component",
        "fabric-system",
        "fabric-packages",
        "fabric-test-resource-clock",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "macro crate manifest must not depend on runtime crate {forbidden}"
        );
    }
}

#[test]
fn macro_source_stays_syntax_only_and_generic() {
    for file in [
        "src/lib.rs",
        "src/ast.rs",
        "src/parse.rs",
        "src/validate.rs",
        "src/codegen/adapter.rs",
        "src/codegen/common.rs",
        "src/codegen/component.rs",
        "src/codegen/resource.rs",
        "src/codegen/system.rs",
    ] {
        let source = fs::read_to_string(crate_root().join(file)).expect("read source");
        for forbidden in [
            "Clock",
            "MemoryClock",
            "DirectCounter",
            "Database",
            "Worker",
            "Gateway",
            "SQLite",
            "Deno",
            "host!",
            "materialize_named",
            "select_provider(",
            "HostDescriptor",
            "Instance::start",
            "ContractProviderSelection::new(",
        ] {
            assert!(
                !source.contains(forbidden),
                "{file} must stay free of concrete or runtime authority term {forbidden}"
            );
        }
    }
}

#[test]
fn macro_codegen_uses_resource_scoped_raw_namespaces_and_strong_module_ids() {
    let codegen =
        fs::read_to_string(crate_root().join("src/codegen/resource.rs")).expect("read codegen");

    assert!(
        codegen.contains("let resource_mod = format_ident!(\"{}\", to_snake_case(resource_name));"),
        "resource! should derive a resource-specific namespace from the Rust type name"
    );
    assert!(
        !codegen.contains("#visibility mod raw {"),
        "resource! must not emit one fixed public raw module per invocation"
    );
    assert!(
        codegen.contains("#visibility mod #resource_mod {") && codegen.contains("pub mod raw {"),
        "resource! should emit a resource-scoped public raw namespace"
    );
    assert!(
        codegen.contains("type Contract = #resource_mod::raw::#contract_name;"),
        "PrimaryResourceContract should point through the resource-scoped raw namespace"
    );
    assert!(
        codegen.contains("pub fn new(")
            && codegen.contains("module_id: #sdk::core::ModuleId")
            && codegen.contains("config: #config_ty"),
        "generated Runtime should retain a strong ModuleId and typed Config constructor"
    );
    assert!(
        codegen.contains("effective_realization_contract_key")
            && codegen.contains("DifferentialSemanticApi"),
        "resource! should expose an effective realization contract only for an explicit differential boundary"
    );
}

#[test]
fn generated_helpers_do_not_assume_a_fixed_caller_module_depth() {
    for file in ["src/codegen/resource.rs", "src/codegen/system.rs"] {
        let source = fs::read_to_string(crate_root().join(file)).expect("read codegen");
        assert!(
            !source.contains("use super::super::super::*;"),
            "{file} must not assume a fixed caller module depth"
        );
    }
}

#[test]
fn resource_macro_codegen_supports_dependencies_and_realizations() {
    let codegen =
        fs::read_to_string(crate_root().join("src/codegen/resource.rs")).expect("read codegen");
    let common =
        fs::read_to_string(crate_root().join("src/codegen/common.rs")).expect("read common");
    let validate = fs::read_to_string(crate_root().join("src/validate.rs")).expect("read validate");

    assert!(
        !codegen.contains(".filter_map(|arg|"),
        "method codegen must not silently discard unsupported parsed arguments"
    );
    assert!(
        codegen.contains("let materialize_self_runtime = if input.runtime_methods.is_some()")
            && codegen.contains("let self_runtime_definition = input.runtime_methods.is_some()"),
        "resource! must generate live runtime machinery only for an explicit self runtime"
    );
    assert!(
        common.contains(".skip(1)")
            && common.contains("macro validation guarantees simple argument bindings"),
        "argument forwarding should rely on prior validation rather than dropping inputs"
    );
    assert!(
        codegen.contains("ContractDependency")
            && codegen.contains("RelationTarget")
            && codegen.contains("relation_requirement_versioned"),
        "resource! should lower relations through the typed relation target anchor"
    );
    assert!(
        codegen.contains("impl #sdk::authoring::AdaptableResourceDefinition for #resource_name")
            && codegen.contains("effective_realization_contract_key"),
        "resource! should lower only explicit differential realization through the adaptable resource facet"
    );
    assert!(
        validate.contains("must use an `&self` receiver")
            && validate.contains("arguments must use simple `name: Type` bindings")
            && validate.contains("generic resource API methods are not supported"),
        "resource! validation should explicitly reject unsupported signatures"
    );
    assert!(
        validate.contains("invalid resource dependency version requirement")
            && validate.contains("duplicate resource dependency field")
            && validate.contains("async resource API methods are not supported"),
        "resource! validation should cover relation and API syntax integrity"
    );
}

#[test]
fn system_macro_codegen_supports_dependencies_and_realizations() {
    let codegen =
        fs::read_to_string(crate_root().join("src/codegen/system.rs")).expect("read codegen");
    let validate = fs::read_to_string(crate_root().join("src/validate.rs")).expect("read validate");
    let parse = fs::read_to_string(crate_root().join("src/parse.rs")).expect("read parse");

    assert!(
        codegen.contains("let system_mod = format_ident!(\"{}\", to_snake_case(system_name));"),
        "system! should derive a system-specific namespace from the Rust type name"
    );
    assert!(
        codegen.contains("let materialize_self_runtime = if input.runtime_methods.is_some()")
            && codegen.contains("let self_runtime_definition = input.runtime_methods.is_some()"),
        "system! must generate live runtime machinery only for an explicit self runtime"
    );
    assert!(
        codegen.contains("type Contract = #system_mod::raw::#contract_name;")
            && codegen.contains("impl #sdk::authoring::AdaptableSystemDefinition for #system_name"),
        "system! should lower its primary API and explicit differential realization through system-owned APIs"
    );
    assert!(
        codegen.contains("RelationTarget")
            && codegen.contains("relation_requirement_versioned")
            && codegen.contains("effective_realization_contract_key"),
        "system! should lower relations and explicit differential requirements through canonical typed helpers"
    );
    assert!(
        validate.contains("invalid system schema version")
            && validate.contains("invalid system dependency version requirement")
            && validate.contains("duplicate system dependency field")
            && validate.contains("async system API methods are not supported"),
        "system! validation should cover system schema, dependency, and method integrity"
    );
    assert!(
        parse.contains("system! supports only one `relations { ... }` section")
            && parse.contains("system! supports only one `realization { ... }` section"),
        "system! parsing should define relation and differential realization sections"
    );
}

#[test]
fn adapter_macro_codegen_exposes_one_type_driven_canonical_path() {
    let lib = fs::read_to_string(crate_root().join("src/lib.rs")).expect("read lib");
    let ast = fs::read_to_string(crate_root().join("src/ast.rs")).expect("read ast");
    let parse = fs::read_to_string(crate_root().join("src/parse.rs")).expect("read parse");
    let validate = fs::read_to_string(crate_root().join("src/validate.rs")).expect("read validate");
    let codegen =
        fs::read_to_string(crate_root().join("src/codegen/adapter.rs")).expect("read codegen");

    assert!(
        lib.contains("pub fn adapter(input: TokenStream) -> TokenStream"),
        "fabric-sdk-macros should expose adapter!"
    );
    assert!(
        ast.contains("pub struct AdapterInput")
            && !ast.contains("AdapterTargetKind")
            && !ast.contains("realization_interface"),
        "adapter! should retain only the type-resolved target in its canonical AST"
    );
    assert!(
        parse.contains("was removed in Fabric 0.5")
            && parse.contains("adapter! supports only one `relations { ... }` section")
            && parse.contains("target API is the realization contract"),
        "adapter! should reject removed target/interface ceremony with a direct migration"
    );
    assert!(
        validate.contains("async adapter runtime methods are not supported")
            && validate.contains("generic adapter runtime methods are not supported")
            && validate.contains("duplicate system dependency field")
            && validate.contains("must use an `&self` receiver"),
        "adapter! validation should reject unsupported runtime syntax clearly"
    );
    assert!(
        codegen.contains("type Target = #target;")
            && codegen.contains(
                "type Compatibility = #sdk::authoring::CanonicalAdapterSupport<#target>;"
            )
            && codegen.contains("CanonicalAdapterSupport<#target>")
            && codegen.contains("AdapterBridgeMode")
            && codegen.contains("__fabric_canonical_adapter_bridge_mode")
            && codegen.contains("relation_requirement_versioned"),
        "adapter! should lower into AdapterDefinition, target-owned typed bridge modes, and canonical relations"
    );
    assert!(
        !codegen.contains("target_api_raw_path")
            && !codegen.contains("to_snake_case(&final_segment.ident)")
            && codegen.contains("__fabric_canonical_adapter_builder")
            && codegen.contains("__fabric_canonical_adapter_contract_key")
            && codegen.contains("host_requirement(&self) -> #sdk::host::HostRequirement"),
        "adapter! should reach target API machinery only through the resolved target type"
    );
    for forbidden in [
        "fabric-adapter",
        "AdapterId",
        "CapabilityDefinition",
        "component!",
        "host!",
        "select_provider(",
        "HostDescriptor",
    ] {
        assert!(
            !codegen.contains(forbidden),
            "adapter! codegen must stay free of forbidden runtime or ontology term {forbidden}"
        );
    }
}

#[test]
fn normal_authoring_defaults_version_support_and_universal_config_without_removing_explicit_forms()
{
    let ast = fs::read_to_string(crate_root().join("src/ast.rs")).expect("read ast");
    let parse = fs::read_to_string(crate_root().join("src/parse.rs")).expect("read parse");
    let adapter =
        fs::read_to_string(crate_root().join("src/codegen/adapter.rs")).expect("read adapter");

    assert!(
        ast.contains("pub schema: Option<RequirementLiteral>")
            && parse.contains("schema.unwrap_or(VersionLiteral::Provisional)")
            && ast.contains("pub enum ConfigDefinition")
            && ast.contains("None,")
            && ast.contains("Inline(Vec<ConfigField>)")
            && ast.contains("Type(Type)")
            && parse.contains("config: config.unwrap_or(ConfigDefinition::None)")
            && parse.contains("expected `config { ... }` or `config: RustConfigType;`")
            && parse.contains("resolve_api"),
        "resource and system normal authoring should derive provisional version, model no/inline/type Config explicitly, and derive the API version from its owner"
    );
    assert!(
        parse.contains("supports: ...;")
            && parse.contains("was removed from adapter! in Fabric 0.5")
            && parse.contains("not an Adapter compatibility declaration"),
        "adapter! should retain support overrides while rejecting removed compatibility ceremony"
    );
    assert!(
        adapter.contains("CanonicalAdapterSupport::<#target>::inferred()")
            && adapter.contains("CanonicalAdapterSupport::<#target>::versioned")
            && !adapter.contains("AdapterTargetKind")
            && !adapter.contains("schema_support_expr"),
        "an adapter without an explicit support override must use its target-owned canonical compatibility bridge"
    );
}

#[test]
fn api_parser_reports_normal_authoring_errors_at_the_api_layer() {
    fn resource_error(source: &str) -> String {
        match syn::parse_str::<crate::ast::ResourceInput>(source) {
            Ok(_) => panic!("resource input should fail to parse"),
            Err(error) => error.to_string(),
        }
    }

    assert!(
        resource_error(r#"Thing { id: "example.thing"; runtime {} }"#)
            .contains("resource! requires an `api { ... }` section")
    );
    assert!(
        resource_error(r#"Thing { id: "example.thing"; api {} api {} runtime {} }"#)
            .contains("resource! supports only one `api { ... }` section")
    );
    assert!(resource_error(
        r#"Thing { id: "example.thing"; api {} contracts { primary Api { id: "example.thing.api"; } } runtime {} }"#
    )
    .contains("`contracts { primary ... }` was removed from resource! in Fabric 0.5"));
    assert!(resource_error(
        r#"Thing { id: "example.thing"; api { fn read(&self) {} } runtime { fn read(&self) {} } }"#
    )
    .contains("api methods are signatures and must not include bodies"));
}

#[test]
fn self_realization_sections_require_an_explicit_runtime_owner() {
    let resource = syn::parse_str::<crate::ast::ResourceInput>(
        r#"Thing { id: "example.thing"; api {} state { State = State; } }"#,
    )
    .expect("stateful semantic definition parses before ownership validation");
    let resource_error = crate::validate::validate_resource(&resource)
        .expect_err("state cannot exist without a self-realizing runtime")
        .to_string();
    assert!(resource_error.contains("state { ... }` requires `runtime { ... }"));

    let system = syn::parse_str::<crate::ast::SystemInput>(
        r#"Thing { id: "example.thing"; api {} lifecycle { start { Ok(()) } } }"#,
    )
    .expect("lifecycle semantic definition parses before ownership validation");
    let system_error = crate::validate::validate_system(&system)
        .expect_err("lifecycle hooks cannot exist without a self-realizing runtime")
        .to_string();
    assert!(system_error.contains("lifecycle { ... }` requires `runtime { ... }"));
}

#[test]
fn differential_realization_reports_semantic_authoring_errors() {
    fn resource_error(source: &str) -> String {
        match syn::parse_str::<crate::ast::ResourceInput>(source) {
            Ok(_) => panic!("differential resource input should fail to parse"),
            Err(error) => error.to_string(),
        }
    }

    assert!(
        resource_error(
            r#"Thing {
            id: "example.thing";
            api { fn read(&self) -> u64; }
            realization { mediate missing; }
        }"#
        )
        .contains("realization mediates `missing`, but that method is not in the semantic API")
    );
    assert!(
        resource_error(
            r#"Thing {
            id: "example.thing";
            api { fn read(&self) -> u64; }
            realization { mediate read; }
        }"#
        )
        .contains(
            "semantic method `read` is mediated and requires a matching `runtime` implementation"
        )
    );
    assert!(
        resource_error(
            r#"Thing {
            id: "example.thing";
            api { fn read(&self) -> u64; }
            realization { mediate read; }
            runtime { fn read(&self) -> String { String::new() } }
        }"#
        )
        .contains("mediation implementation `read` must match its semantic API signature")
    );
}

#[test]
fn owner_derived_api_identity_depends_only_on_owner_kind_and_id() {
    let common =
        fs::read_to_string(crate_root().join("src/codegen/common.rs")).expect("read common");

    assert!(
        common.contains("concat!(\"fabric.resource.api.\", #owner_id)")
            && common.contains("concat!(\"fabric.system.api.\", #owner_id)"),
        "normal API identities must derive from the owner kind and stable owner id"
    );
    let identity_lowering = common
        .split("pub fn api_contract_id_expr")
        .nth(1)
        .expect("API identity lowering")
        .split("pub fn service_method_tokens")
        .next()
        .expect("API identity lowering end");
    assert!(
        !identity_lowering.contains("config")
            && !identity_lowering.contains("relations")
            && !identity_lowering.contains("runtime"),
        "API identity lowering must not depend on Config, Relations, or runtime implementation"
    );
}

#[test]
fn config_codegen_is_shared_and_keeps_no_config_out_of_normal_apis() {
    let common =
        fs::read_to_string(crate_root().join("src/codegen/common.rs")).expect("read common");
    for file in [
        "src/codegen/resource.rs",
        "src/codegen/system.rs",
        "src/codegen/adapter.rs",
    ] {
        let codegen = fs::read_to_string(crate_root().join(file)).expect("read codegen");
        assert!(
            codegen.contains("config_type_tokens")
                && codegen.contains("inline_config_definition_tokens")
                && codegen.contains("has_config"),
            "{file} must lower Config through the shared authoring model"
        );
        assert!(
            codegen.contains("config: #config_ty")
                && codegen.contains("pub fn config(&self) -> &#config_ty")
                && codegen.contains("&self.config"),
            "{file} must give authored runtime code typed read-only Config access"
        );
    }
    assert!(
        common.contains("ConfigDefinition::None => quote!(())")
            && common.contains("ConfigDefinition::Type(ty) => quote!(#ty)"),
        "no-config and creator-owned Rust Config types must not require generated wrappers"
    );
}

#[test]
fn lifecycle_authoring_is_shared_by_resource_system_and_adapter_and_component_uses_participation_runtime()
 {
    let ast = fs::read_to_string(crate_root().join("src/ast.rs")).expect("read ast");
    let parse = fs::read_to_string(crate_root().join("src/parse.rs")).expect("read parse");
    let resource =
        fs::read_to_string(crate_root().join("src/codegen/resource.rs")).expect("read resource");
    let system =
        fs::read_to_string(crate_root().join("src/codegen/system.rs")).expect("read system");
    let adapter =
        fs::read_to_string(crate_root().join("src/codegen/adapter.rs")).expect("read adapter");
    let component =
        fs::read_to_string(crate_root().join("src/codegen/component.rs")).expect("read component");

    assert!(
        ast.contains("pub struct RuntimeStateDefinition")
            && ast.contains("pub struct RuntimeLifecycleDefinition"),
        "state and lifecycle syntax should project through shared macro AST machinery"
    );
    for subject in ["resource", "system", "adapter"] {
        assert!(
            parse.contains(&format!(
                "{subject}! supports only one `state {{ ... }}` section"
            )) && parse.contains(&format!(
                "{subject}! supports only one `lifecycle {{ ... }}` section"
            )),
            "{subject}! should reject duplicate state and lifecycle sections"
        );
    }
    for codegen in [&resource, &system, &adapter] {
        assert!(
            codegen.contains("RuntimeState")
                && codegen.contains("RuntimeContext")
                && codegen.contains("fn bind_instance_context")
                && codegen.contains("#initialize_hook")
                && codegen.contains("#stop_hook"),
            "all supported authoring worlds should lower through the shared Core runtime spine"
        );
    }
    assert!(
        component.contains("RuntimeState")
            && !component.contains("RuntimeContext")
            && parse.contains("component runtime supports only one `state { ... }` section")
            && parse.contains("component runtime supports only one `prepare { ... }` hook"),
        "Component state and preparation belong to a participation realization, not the Resource/System lifecycle vocabulary"
    );
    assert!(
        component.contains("new_with_teardown")
            && parse.contains("component runtime supports only one `teardown { ... }` hook"),
        "component! should expose participation-scoped teardown inside canonical runtime authoring"
    );
}

#[test]
fn component_macro_codegen_supports_typed_api_lowering() {
    let lib = fs::read_to_string(crate_root().join("src/lib.rs")).expect("read lib");
    let ast = fs::read_to_string(crate_root().join("src/ast.rs")).expect("read ast");
    let parse = fs::read_to_string(crate_root().join("src/parse.rs")).expect("read parse");
    let validate = fs::read_to_string(crate_root().join("src/validate.rs")).expect("read validate");
    let codegen =
        fs::read_to_string(crate_root().join("src/codegen/component.rs")).expect("read codegen");

    assert!(
        lib.contains("pub fn component(input: TokenStream) -> TokenStream"),
        "fabric-sdk-macros should expose component!"
    );
    assert!(
        ast.contains("pub struct ComponentInput")
            && ast.contains("pub struct ApiDefinition")
            && !ast.contains("ComponentOperationDefinition")
            && !ast.contains("ComponentOperationContext")
            && !ast.contains("legacy_"),
        "component! should use the shared API declaration AST without a legacy operation frontend"
    );
    assert!(
        parse.contains("component! requires an `id: ...;` declaration")
            && parse.contains("`operations { ... }` was removed")
            && parse.contains("`requires { ... }` was removed")
            && parse.contains("top-level `teardown { ... }` was removed")
            && !parse.contains("parse_component_operations")
            && !parse.contains("parse_requires")
            && !parse.contains("parse_system_dependencies"),
        "component! parsing should reject the removed frontend with migration diagnostics"
    );
    assert!(
        validate.contains("duplicate API method")
            && !validate.contains("component operation handlers"),
        "component! validation should retain shared API checks without handler validation"
    );
    assert!(
        codegen
            .contains("let component_mod = format_ident!(\"{}\", to_snake_case(component_name));")
            && codegen.contains("format_ident!(\"api\")")
            && codegen.contains("pub mod #endpoint_module {")
            && codegen.contains("impl #sdk::authoring::ComponentDefinition for #component_name")
            && codegen.contains("#sdk::authoring::ComponentSpec::<Self>::self_realizing(config)")
            && codegen.contains("scope.operation(")
            && !codegen.contains("scope.operation_with_context(")
            && !codegen.contains("#sdk::component::InvocationContext")
            && !codegen.contains("InvocationContext::new")
            && !codegen.contains("InvocationRail")
            && !codegen.contains("legacy_")
            && !codegen.contains("requirements_module"),
        "component! should lower canonical APIs through an api module and register participation realization handlers"
    );
    assert!(
        codegen.contains("fn declaration() -> #sdk::component::ComponentDeclaration")
            && codegen.contains("#sdk::component::ComponentDeclaration::new("),
        "component! should populate declarative endpoint metadata from its static operation table"
    );
    assert!(
        codegen.contains("impl #sdk::authoring::SelfRealizingComponentDefinition")
            && codegen.contains("ComponentParticipationRealization::new_with_teardown("),
        "component! should attach handlers through the optional native runtime bridge"
    );
    assert!(
        !codegen.contains("fn prepare("),
        "component! must not generate definition-time runtime preparation"
    );
    assert!(
        !codegen.contains("#visibility mod raw {")
            && !codegen.contains("ComponentParticipationRealization::new(C::component_id()")
            && !codegen.contains("AdapterDefinition")
            && !codegen.contains("HostRequirement")
            && !codegen.contains("ResourceDefinition")
            && !codegen.contains("SystemDefinition"),
        "component! codegen must stay component-scoped and avoid reopening sealed bridges or other semantic planes"
    );
}
