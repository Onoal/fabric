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
        codegen
            .contains("pub fn new(module_id: #sdk::core::ModuleId, config: #config_name) -> Self"),
        "generated Runtime::new should require a strong ModuleId"
    );
    assert!(
        codegen.contains("#visibility mod #resource_mod {")
            && codegen.contains("let realization_mod = format_ident!(\"realization\")")
            && codegen.contains("pub mod #realization_mod {"),
        "resource! should scope generated realization APIs under the resource namespace"
    );
}

#[test]
fn resource_macro_codegen_supports_dependencies_and_realizations() {
    let codegen =
        fs::read_to_string(crate_root().join("src/codegen/resource.rs")).expect("read codegen");
    let common =
        fs::read_to_string(crate_root().join("src/codegen/common.rs")).expect("read common");
    let validate = fs::read_to_string(crate_root().join("src/validate.rs")).expect("read validate");
    let parse = fs::read_to_string(crate_root().join("src/parse.rs")).expect("read parse");

    assert!(
        !codegen.contains(".filter_map(|arg|"),
        "method codegen must not silently discard unsupported parsed arguments"
    );
    assert!(
        common.contains(".skip(1)")
            && common.contains("macro validation guarantees simple argument bindings"),
        "argument forwarding should rely on prior validation rather than dropping inputs"
    );
    assert!(
        codegen.contains("ContractDependency")
            && codegen.contains("Requires::<#resource>::provisional()")
            && codegen.contains("Requires::<#resource>::versioned("),
        "resource! should lower requires {{}} through the typed resource dependency anchor"
    );
    assert!(
        codegen.contains("impl #sdk::authoring::AdaptableResourceDefinition for #resource_name")
            && codegen.contains("#resource_mod::realization::raw::requirement()"),
        "resource! should lower adapter realization sections through the adaptable resource facet"
    );
    assert!(
        validate.contains("must use an `&self` receiver")
            && validate.contains("arguments must use simple `name: Type` bindings")
            && validate.contains("generic resource contract methods are not supported"),
        "resource! validation should explicitly reject unsupported signatures"
    );
    assert!(
        validate.contains("invalid resource dependency version requirement")
            && validate.contains("duplicate resource dependency field")
            && parse.contains(
                "{subject} realization contract requires a `compatibility: ...;` declaration"
            )
            && validate.contains("async {subject} realization methods are not supported"),
        "resource! validation should cover dependency and realization syntax integrity"
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
        codegen.contains("type Contract = #system_mod::raw::#contract_name;")
            && codegen.contains("impl #sdk::authoring::AdaptableSystemDefinition for #system_name"),
        "system! should lower primary contracts and optional realization through system-owned APIs"
    );
    assert!(
        codegen.contains("SystemRequires::<#system>::provisional()")
            && codegen.contains("SystemRequires::<#system>::versioned(")
            && codegen.contains("#system_mod::realization::raw::requirement()"),
        "system! should lower system dependencies and realization requirements through canonical typed helpers"
    );
    assert!(
        validate.contains("invalid system schema version")
            && validate.contains("invalid system dependency version requirement")
            && validate.contains("duplicate system dependency field")
            && validate.contains("async system contract methods are not supported"),
        "system! validation should cover system schema, dependency, and method integrity"
    );
    assert!(
        parse.contains("system! supports only one `system { ... }` section")
            && parse.contains(
                "{subject} realization contract requires a `compatibility: ...;` declaration"
            ),
        "system! parsing should define the dedicated system dependency and realization sections"
    );
}

#[test]
fn adapter_macro_codegen_supports_explicit_realizations() {
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
            && ast.contains("pub enum AdapterTargetKind")
            && ast.contains("Resource,")
            && ast.contains("System,"),
        "adapter! should parse an explicit target-plane model"
    );
    assert!(
        parse.contains("adapter! requires `for resource ...` or `for system ...`")
            && parse.contains("adapter! supports only one `system { ... }` section")
            && parse.contains("legacy `realization: ...;` declaration")
            && parse.contains("legacy `schema: ...;` declaration"),
        "adapter! parsing should retain explicit compatibility forms without making them mandatory"
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
            && codegen.contains("type Compatibility = #schema_support_ty;")
            && codegen.contains("impl #target_service for Runtime")
            && codegen.contains("<#raw_impl_mod::Runtime as #interface>::realization_contract")
            && codegen.contains("SystemRequires::<#system>::versioned("),
        "adapter! should lower into AdapterDefinition, explicit typed realization interfaces, and canonical system dependencies"
    );
    assert!(
        !codegen.contains("target_namespace_path")
            && !codegen.contains("to_snake_case(&last.into_value().ident)")
            && codegen.contains("let interface = &input.realization_interface;")
            && codegen.contains("realization_target")
            && codegen.contains("host_requirement(&self) -> #sdk::host::HostRequirement"),
        "adapter! should use an explicit public realization interface rather than derive another crate's module layout"
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
fn normal_authoring_defaults_version_support_and_empty_config_without_removing_explicit_forms() {
    let ast = fs::read_to_string(crate_root().join("src/ast.rs")).expect("read ast");
    let parse = fs::read_to_string(crate_root().join("src/parse.rs")).expect("read parse");
    let adapter =
        fs::read_to_string(crate_root().join("src/codegen/adapter.rs")).expect("read adapter");

    assert!(
        ast.contains("pub schema: Option<RequirementLiteral>")
            && parse.contains("schema.unwrap_or(VersionLiteral::Provisional)")
            && parse.contains("config_fields: config_fields.unwrap_or_default()")
            && parse.contains("resolve_contract_versions"),
        "resource and system normal authoring should derive provisional version, empty config, and contract version"
    );
    assert!(
        parse.contains("supports: ...;")
            && parse.contains("version: ...;` or legacy `realization: ...;")
            && parse.contains("schema: ...;` declaration"),
        "adapter! should prefer version/support syntax while retaining legacy explicit forms"
    );
    assert!(
        adapter.contains("ResourceSchemaIdentity::Versioned")
            && adapter.contains("SystemSchemaIdentity::Versioned")
            && adapter.contains("exact target resource schema requirement")
            && adapter.contains("exact target system schema requirement"),
        "an adapter without an explicit support override must derive exact target compatibility"
    );
}

#[test]
fn lifecycle_authoring_is_shared_by_resource_system_and_adapter_and_component_uses_teardown() {
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
        !component.contains("RuntimeState") && !component.contains("RuntimeContext"),
        "component teardown must not inherit the Resource/System runtime lifecycle vocabulary"
    );
    assert!(
        component.contains("new_with_teardown")
            && parse.contains("component! supports only one `teardown { ... }` section"),
        "component! should expose participation-scoped teardown without lifecycle DSL"
    );
}

#[test]
fn component_macro_codegen_supports_typed_operations() {
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
            && ast.contains("pub struct ComponentOperationDefinition")
            && ast.contains("pub enum ComponentOperationContext"),
        "component! should parse dedicated component and operation inputs"
    );
    assert!(
        parse.contains("component! requires an `id: ...;` declaration")
            && parse.contains("component! supports only one `operations { ... }` section")
            && parse.contains("component operation requires a `handler ...` declaration")
            && parse.contains("component operation supports only `context: invocation;`"),
        "component! parsing should define component sections and required operation fields"
    );
    assert!(
        validate.contains("duplicate component operation")
            && validate.contains(
                "component operation handlers must use a closure expression like `|input: Type| async move { ... }`"
            )
            && validate.contains("component operation handlers must declare exactly one typed input parameter")
            && validate.contains("component operation handlers must use a simple `name: Type` input binding")
            && validate.contains("context-aware component operation handlers must declare `|context, input: Type|`")
            && validate.contains("context-aware component operation handlers with dependencies must declare `|context, dependencies, input: Type|`"),
        "component! validation should reject duplicate operations and unsupported handler forms clearly"
    );
    assert!(
        codegen
            .contains("let component_mod = format_ident!(\"{}\", to_snake_case(component_name));")
            && codegen.contains("pub mod operations {")
            && codegen.contains("impl #sdk::authoring::ComponentDefinition for #component_name")
            && codegen.contains("#sdk::authoring::ComponentSpec::<Self>::self_realizing(config)")
            && codegen.contains("scope.operation(")
            && codegen.contains("scope.operation_with_context(")
            && codegen.contains("#sdk::component::InvocationContext")
            && !codegen.contains("InvocationContext::new")
            && !codegen.contains("InvocationRail"),
        "component! should lower into semantic ComponentDefinition, explicit self realization, and ComponentRuntimeScope registration"
    );
    assert!(
        codegen.contains("fn declaration() -> #sdk::component::ComponentDeclaration")
            && codegen.contains("#sdk::component::ComponentDeclaration::new("),
        "component! should populate declarative endpoint metadata from its static operation table"
    );
    assert!(
        codegen.contains("impl #sdk::authoring::SelfRealizingComponentDefinition")
            && codegen.contains("ComponentRuntimeDefinition::new("),
        "component! should attach handlers through the optional native runtime bridge"
    );
    assert!(
        !codegen.contains("fn prepare("),
        "component! must not generate definition-time runtime preparation"
    );
    assert!(
        !codegen.contains("#visibility mod raw {")
            && !codegen.contains("ComponentRuntimeDefinition::new(C::component_id()")
            && !codegen.contains("AdapterDefinition")
            && !codegen.contains("HostRequirement")
            && !codegen.contains("ResourceDefinition")
            && !codegen.contains("SystemDefinition"),
        "component! codegen must stay component-scoped and avoid reopening sealed bridges or other semantic planes"
    );
}
