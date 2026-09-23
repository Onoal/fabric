use std::fs;
use std::path::Path;

fn crate_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn sdk_manifest_depends_only_on_generic_fabric_layers() {
    let manifest = fs::read_to_string(crate_root().join("Cargo.toml")).expect("read manifest");
    for required in [
        "fabric-core",
        "fabric-host",
        "fabric-system",
        "fabric-resource",
        "fabric-component",
        "fabric-sdk-macros",
    ] {
        assert!(
            manifest.contains(required),
            "sdk manifest must depend on {required}"
        );
    }
    for experimental in [
        "fabric-binding",
        "fabric-projection",
        "fabric-resource-registry",
    ] {
        assert!(
            !manifest.contains(experimental),
            "sdk manifest must not depend on experimental {experimental}"
        );
    }
    for forbidden in [
        "fabric-resource-worker",
        "fabric-resource-server",
        "fabric-resource-process",
        "fabric-resource-database",
        "fabric-resource-kv",
        "fabric-resource-service",
        "fabric-adapter-",
        "fabric-component-gateway",
        "fabric-component-namespace",
        "fabric-component-publication",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "sdk manifest must stay free of concrete dependency {forbidden}"
        );
    }
}

#[test]
fn sdk_source_stays_generic_and_curated() {
    for file in [
        "src/lib.rs",
        "src/adapter.rs",
        "src/prelude.rs",
        "src/core.rs",
        "src/resource.rs",
        "src/component.rs",
        "src/host.rs",
        "src/contracts.rs",
        "src/ids.rs",
        "src/system.rs",
        "src/versions.rs",
        "src/experimental/mod.rs",
        "src/experimental/projection/contract.rs",
        "src/experimental/projection/error.rs",
        "src/experimental/projection/handler.rs",
        "src/experimental/projection/model/lease.rs",
        "src/experimental/projection/model/materialized.rs",
        "src/authoring/block_author.rs",
        "src/authoring/fabric_builder.rs",
        "src/authoring/composition_ext.rs",
        "src/authoring/fabric/mod.rs",
        "src/authoring/fabric/builder.rs",
        "src/authoring/fabric/manifest.rs",
        "src/authoring/fabric/resource.rs",
        "src/authoring/fabric/sealed.rs",
        "src/authoring/fabric/system.rs",
        "src/authoring/system/mod.rs",
        "src/authoring/definitions/mod.rs",
        "src/authoring/definitions/adapter_definition.rs",
        "src/authoring/definitions/adapter_provider_module.rs",
        "src/authoring/definitions/component_definition.rs",
        "src/authoring/definitions/contract_dependency.rs",
        "src/authoring/definitions/primary_resource_contract.rs",
        "src/authoring/definitions/requires.rs",
        "src/authoring/definitions/resource_definition.rs",
        "src/authoring/definitions/resource_realization.rs",
        "src/authoring/definitions/resource_selection.rs",
        "src/authoring/runtime_authoring.rs",
        "src/authoring/system/adaptable_system_definition.rs",
        "src/authoring/system/primary_system_contract.rs",
        "src/authoring/system/system_definition.rs",
        "src/authoring/system/system_realization.rs",
        "src/authoring/system/system_requires.rs",
        "src/authoring/system/system_selection.rs",
    ] {
        let source = fs::read_to_string(crate_root().join(file)).expect("read source");
        for forbidden in [
            "Deno",
            "Systemd",
            "Worker",
            "Server",
            "Database",
            "Gateway",
            "SQLite",
            "Fjall",
            "Pingora",
            "Cedar",
            "Clock",
            "Greeter",
            "third-party.test-clock",
            "macro_rules!",
            "proc_macro",
            "host!",
            "fabric-packages",
        ] {
            assert!(
                !source.contains(forbidden),
                "{file} must stay free of concrete implementation name {forbidden}"
            );
        }
    }
}

#[test]
fn runtime_authoring_stays_sdk_machinery_not_a_lifecycle_ontology() {
    let runtime = fs::read_to_string(crate_root().join("src/authoring/runtime_authoring.rs"))
        .expect("read runtime authoring");
    let root = fs::read_to_string(crate_root().join("src/lib.rs")).expect("read root exports");
    let prelude =
        fs::read_to_string(crate_root().join("src/prelude.rs")).expect("read prelude exports");

    assert!(
        runtime.contains("fresh runtime occurrence")
            && runtime.contains("not a Fabric semantic subject")
            && runtime.contains("StatefulAdapterDefinition"),
        "SDK should expose stateful runtime construction without creating a semantic lifecycle plane"
    );
    for public in ["RuntimeState", "RuntimeContext", "StatefulRuntimeAuthoring"] {
        assert!(
            !root.contains(&format!("pub use authoring::{public}")) && !prelude.contains(public),
            "normal root and prelude surfaces must not expose handwritten runtime machinery {public}"
        );
    }
}

#[test]
fn component_normal_surface_is_small_and_named_surfaces_keep_advanced_capability() {
    let root = fs::read_to_string(crate_root().join("src/lib.rs")).expect("read root exports");
    let prelude =
        fs::read_to_string(crate_root().join("src/prelude.rs")).expect("read prelude exports");
    let component =
        fs::read_to_string(crate_root().join("src/component.rs")).expect("read component API");
    let authoring =
        fs::read_to_string(crate_root().join("src/authoring/mod.rs")).expect("read authoring API");

    for leaked in [
        "ComponentDefinition",
        "ComponentSpec",
        "ComponentInstanceBinding",
        "ComponentParticipation",
        "ComponentHost",
        "ComponentHostService",
        "ComponentHostLifecycle",
        "ComponentHostStatus",
        "ComponentHostModule",
        "ComponentHostHandle",
        "ComponentParticipationRealization",
        "OperationId",
        "OperationTypeId",
        "OperationKey",
        "OperationRail",
        "OperationRegistrar",
        "ComponentParticipationContribution",
        "ComponentParticipationPreparation",
        "ComponentParticipationScope",
        "InvocationContext",
        "ComponentRegistry",
        "ComponentControl",
        "ComponentReadiness",
        "ComponentReconstruction",
    ] {
        assert!(
            !root.contains(leaked) && !prelude.contains(leaked),
            "normal root/prelude must not expose Component implementation vocabulary {leaked}"
        );
    }
    for family in [
        "pub mod declaration",
        "pub mod participation",
        "pub mod invocation",
        "pub mod operator",
        "pub mod advanced",
    ] {
        assert!(
            component.contains(family),
            "Component API must expose {family}"
        );
    }
    assert!(
        component.contains("#[doc(hidden)]") && authoring.contains("pub mod component"),
        "macro ABI must be hidden and handwritten Component APIs must have a named authoring module"
    );
}

#[test]
fn component_host_export_identity_stays_stable_during_surface_purification() {
    let builder = fs::read_to_string(crate_root().join("src/authoring/fabric/builder.rs"))
        .expect("read Fabric builder");
    assert!(
        builder.contains("fabric.sdk.export.component-runtime"),
        "renaming the Rust host vocabulary must not rewrite the Fabric manifest export identity"
    );
}

#[test]
fn requires_anchor_has_no_public_foreign_contract_escape() {
    let source = fs::read_to_string(crate_root().join("src/authoring/definitions/requires.rs"))
        .expect("read requires definition");
    let system_source =
        fs::read_to_string(crate_root().join("src/authoring/system/system_requires.rs"))
            .expect("read system requires definition");

    assert!(
        !source.contains("pub fn from_requirement"),
        "Requires<R>::from_requirement must not stay public"
    );
    assert!(
        source.contains("pub(crate) fn from_requirement"),
        "Requires<R>::from_requirement should stay crate-local for SDK internals"
    );
    assert!(
        !system_source.contains("pub fn from_requirement"),
        "SystemRequires<S>::from_requirement must not stay public"
    );
    assert!(
        system_source.contains("pub(crate) fn from_requirement"),
        "SystemRequires<S>::from_requirement should stay crate-local for SDK internals"
    );
}

#[test]
fn resource_selection_owns_total_module_id_derivation() {
    let resource_definition =
        fs::read_to_string(crate_root().join("src/authoring/definitions/resource_definition.rs"))
            .expect("read resource definition");
    let adapter_definition =
        fs::read_to_string(crate_root().join("src/authoring/definitions/adapter_definition.rs"))
            .expect("read adapter definition");
    let definitions = fs::read_to_string(crate_root().join("src/authoring/definitions/mod.rs"))
        .expect("read definitions mod");
    let resource_selection =
        fs::read_to_string(crate_root().join("src/authoring/definitions/resource_selection.rs"))
            .expect("read resource selection");
    let resource_realization =
        fs::read_to_string(crate_root().join("src/authoring/definitions/resource_realization.rs"))
            .expect("read resource realization");
    let base_resource_definition = resource_definition
        .split("pub trait AdaptableResourceDefinition")
        .next()
        .expect("base resource definition");

    assert!(
        !base_resource_definition.contains("fn module_id("),
        "ResourceDefinition must not own overridable module id derivation"
    );
    assert!(
        !base_resource_definition.contains("expect(\"resource definition module ids"),
        "ResourceDefinition must not panic on module id derivation"
    );
    assert!(
        !base_resource_definition.contains("type AdapterHandle"),
        "ResourceDefinition must not require a universal adapter handle"
    );
    assert!(
        base_resource_definition.contains("fn materialize(selection: &ResourceSelection<Self>)"),
        "ResourceDefinition must own one canonical resource materialization bridge"
    );
    assert!(
        base_resource_definition.contains("-> Option<Box<dyn ModuleRuntime>>"),
        "ResourceDefinition runtime realization must stay optional for declaration-only Resources"
    );
    assert!(
        base_resource_definition.contains("None"),
        "ResourceDefinition must default to no runtime materializer"
    );
    for forbidden in [
        "DeclarativeResourceDefinition",
        "RuntimeResourceDefinition",
        "ExecutableResourceDefinition",
        "MaterializableResourceDefinition",
    ] {
        assert!(
            !resource_definition.contains(forbidden),
            "SDK must not split Resources into parallel definition families, found {forbidden}"
        );
    }
    assert!(
        resource_definition.contains("pub trait AdaptableResourceDefinition: ResourceDefinition"),
        "SDK must expose an explicit adaptable resource facet"
    );
    assert!(
        !resource_definition.contains("DirectResourceDefinition"),
        "SDK must not preserve a separate direct resource category"
    );
    assert!(
        adapter_definition.contains("type Target: Send + Sync + 'static;"),
        "AdapterDefinition must own an explicit target type"
    );
    assert!(
        adapter_definition.contains("type Compatibility: Clone + Send + Sync + 'static;"),
        "AdapterDefinition must own an explicit target-scoped compatibility type"
    );
    assert!(
        !adapter_definition.contains("adapter_handle"),
        "AdapterDefinition must not retain adapter-handle injection"
    );
    assert!(
        adapter_definition.contains("fn materialize_provider(&self, provider_module_id: ModuleId)"),
        "AdapterDefinition should materialize an adapter provider module"
    );
    assert!(
        adapter_definition.contains("-> Option<Box<dyn ModuleRuntime>>"),
        "AdapterDefinition runtime participation must stay optional for declaration-only Adapters"
    );
    assert!(
        adapter_definition.contains("None"),
        "AdapterDefinition must default to no runtime provider materializer"
    );
    for forbidden in [
        "DeclarativeAdapterDefinition",
        "RuntimeAdapterDefinition",
        "ExecutableAdapterDefinition",
        "MaterializableAdapterDefinition",
        "AdapterId",
    ] {
        assert!(
            !adapter_definition.contains(forbidden),
            "SDK must not split Adapters into parallel definition families, found {forbidden}"
        );
    }
    assert!(
        resource_definition.contains("type RealizationContract: Send + Sync + 'static;"),
        "AdaptableResourceDefinition should own a realization contract type"
    );
    assert!(
        resource_definition.contains("fn realization_requirement()"),
        "AdaptableResourceDefinition should own its realization requirement"
    );
    assert!(
        resource_selection.contains(".selection."),
        "ResourceSelection should own deterministic selection module ids"
    );
    assert!(
        !resource_selection.contains("pub fn direct("),
        "ResourceSelection must not preserve the old direct-resource authoring path"
    );
    assert!(
        resource_selection.contains("pub fn using<A>(")
            && resource_selection
                .contains(") -> Result<ResourceRealization<R, A>, ResourceCompatibilityError>"),
        "ResourceSelection::using should produce the declarative adapted authoring value"
    );
    assert!(
        resource_selection.contains("ContractProviderSelection::new"),
        "using() should generate the canonical core provider selection primitive"
    );
    assert!(
        resource_selection.contains("{}.realization"),
        "adapter provider module ids should derive from the resource selection slot"
    );
    assert!(
        resource_realization.contains("pub fn into_raw_parts(")
            && resource_realization.contains("ContractProviderSelection,"),
        "adapted resource authoring should expose the explicit raw-parts bridge"
    );
    assert!(
        definitions.contains("pub use resource_realization::ResourceRealization;"),
        "definitions mod should export the new adapted authoring value"
    );
    assert!(
        definitions.contains("pub use contract_dependency::ContractDependency;"),
        "definitions mod should export the generic contract dependency helper"
    );
    assert!(
        resource_realization.contains("ContractProviderSelection"),
        "adapted resource authoring should preserve explicit provider-selection ownership"
    );
    assert!(
        resource_realization.contains("A: AdapterDefinition<Target = R>")
            && resource_realization.contains("ResourceAdapterCompatibility<R>"),
        "resource realizations must share target-aware compatibility between legacy and canonical adapters"
    );
}

#[test]
fn component_definition_owns_canonical_component_identity() {
    let source =
        fs::read_to_string(crate_root().join("src/authoring/definitions/component_definition.rs"))
            .expect("read component definition");

    assert!(
        source.contains("fn component_id() -> ComponentId;"),
        "ComponentDefinition must own canonical component identity"
    );
    assert!(
        source.contains("fn declaration() -> ComponentDeclaration;"),
        "ComponentDefinition must own declarative endpoint metadata"
    );
    assert!(
        source.contains("pub fn requires_resource<R>"),
        "ComponentSpec must own typed Resource requirement authoring"
    );
    assert!(
        source.contains("pub struct ComponentRealizationContract<C>"),
        "external realization preparation must remain a separate typed contract"
    );
    assert!(
        source.contains("fn define(config: Self::Config) -> ComponentSpec<Self>"),
        "ComponentDefinition must expose the configured specification bridge"
    );
    assert!(
        !source.contains("runtime_attachment"),
        "ComponentDefinition must not own a local realization hook"
    );
    assert!(
        source.contains("pub trait SelfRealizingComponentDefinition: ComponentDefinition"),
        "native realization must be explicit and separate from ComponentDefinition"
    );
    assert!(
        source.contains("pub fn new(config: C::Config) -> Self"),
        "ComponentSpec must own canonical SDK construction"
    );
    assert!(
        source.contains("pub fn declaration(&self) -> &ComponentDeclaration"),
        "ComponentSpec must preserve declarative truth independently of local realization"
    );
    assert!(
        source.contains(
            "pub fn into_self_realization(self) -> Option<ComponentParticipationRealization>"
        ),
        "ComponentSpec must expose explicit self realization separately from declaration"
    );
    assert!(
        !source.contains("from_runtime_definition"),
        "ComponentSpec must not expose arbitrary runtime-definition construction"
    );
    assert!(
        !source.contains("impl<C> From<ComponentSpec<C>> for ComponentParticipationRealization"),
        "ComponentSpec must not convert infallibly into runtime-only state"
    );
    for forbidden in [
        "DeclarativeComponentDefinition",
        "RuntimeComponentDefinition",
        "ExecutableComponentDefinition",
        "MaterializableComponentDefinition",
    ] {
        assert!(
            !source.contains(forbidden),
            "SDK must not split Components into parallel definition families, found {forbidden}"
        );
    }
}

#[test]
fn sdk_provider_selection_stays_a_thin_core_wrapper() {
    let builder = fs::read_to_string(crate_root().join("src/authoring/fabric_builder.rs"))
        .expect("read fabric builder");
    let core = fs::read_to_string(crate_root().join("src/core.rs")).expect("read sdk core");
    let prelude = fs::read_to_string(crate_root().join("src/prelude.rs")).expect("read prelude");

    assert!(
        builder.contains(
            "pub fn select_provider(mut self, selection: ContractProviderSelection) -> Self"
        ),
        "FabricBuilder should expose the generic provider-selection seam"
    );
    assert!(
        builder.contains("self.inner = self.inner.select_provider(selection);"),
        "FabricBuilder provider selection should delegate directly to core"
    );
    assert!(
        core.contains("ContractProviderSelection"),
        "sdk core facade should re-export ContractProviderSelection"
    );
    assert!(
        !prelude.contains("ContractProviderSelection"),
        "sdk prelude must keep raw provider selection in the named core module"
    );
}

#[test]
fn sdk_host_materialization_surface_stays_thin_and_generic() {
    let adapter_definition =
        fs::read_to_string(crate_root().join("src/authoring/definitions/adapter_definition.rs"))
            .expect("read adapter definition");
    let provider_module = fs::read_to_string(
        crate_root().join("src/authoring/definitions/adapter_provider_module.rs"),
    )
    .expect("read adapter provider module");
    let composition_ext = fs::read_to_string(crate_root().join("src/authoring/composition_ext.rs"))
        .expect("read composition ext");
    let core = fs::read_to_string(crate_root().join("src/core.rs")).expect("read sdk core");
    let prelude = fs::read_to_string(crate_root().join("src/prelude.rs")).expect("read prelude");

    assert!(
        adapter_definition.contains("fn host_requirement(&self) -> HostRequirement"),
        "AdapterDefinition should expose a target-neutral host compatibility declaration"
    );
    assert!(
        provider_module.contains("fn declaration(&self) -> ModuleDeclaration")
            && provider_module.contains(".with_host_requirement(")
            && provider_module.contains("HostMaterializationRequirement::new"),
        "AdapterProviderModule should lower adapter host compatibility into the raw module seam"
    );
    assert!(
        composition_ext.contains("fn materialize_named_on<I>(")
            && composition_ext
                .contains("self.materialize_on(instance_id.into_instance_id()?, host)"),
        "CompositionExt should expose only a thin named-host wrapper over core materialization"
    );
    assert!(
        core.contains("HostMaterializationRequirement")
            && !prelude.contains("HostMaterializationRequirement"),
        "sdk prelude must keep raw host materialization declarations in the named core module"
    );
    for source in [adapter_definition, provider_module, composition_ext] {
        for forbidden in [
            "HostDescriptor::native",
            "FabricHost",
            "HostRegistry",
            "HostServiceLocator",
            "host!",
            "fabric-packages",
        ] {
            assert!(
                !source.contains(forbidden),
                "sdk host materialization surface must stay thin and free of {forbidden}"
            );
        }
    }
}

#[test]
fn fabric_owns_normal_typed_authoring_without_resolution_machinery() {
    let authoring =
        fs::read_to_string(crate_root().join("src/authoring/mod.rs")).expect("read authoring mod");
    let lib = fs::read_to_string(crate_root().join("src/lib.rs")).expect("read sdk lib");
    let prelude = fs::read_to_string(crate_root().join("src/prelude.rs")).expect("read prelude");
    let builder = fs::read_to_string(crate_root().join("src/authoring/fabric/builder.rs"))
        .expect("read fabric builder");
    let manifest = fs::read_to_string(crate_root().join("src/authoring/fabric/manifest.rs"))
        .expect("read fabric manifest");
    let resource = fs::read_to_string(crate_root().join("src/authoring/fabric/resource.rs"))
        .expect("read fabric resource contribution");
    let system = fs::read_to_string(crate_root().join("src/authoring/fabric/system.rs"))
        .expect("read fabric system contribution");
    let composition_ext = fs::read_to_string(crate_root().join("src/authoring/composition_ext.rs"))
        .expect("read composition ext");

    assert!(
        authoring.contains("pub use fabric::{")
            && authoring.contains("Composition")
            && authoring.contains("Fabric,")
            && authoring.contains("FabricBuildError")
            && authoring.contains("FabricManifest"),
        "sdk authoring surface should export the normal typed authoring owner and built Composition"
    );
    assert!(
        lib.contains("Composition, Fabric")
            && prelude.contains("Composition, CompositionError")
            && !lib.contains("BuiltFabric")
            && !prelude.contains("BuiltFabric"),
        "sdk root and prelude must expose only the canonical SDK Composition"
    );
    assert!(
        lib.contains("Fabric,") && prelude.contains("Fabric,"),
        "sdk lib and prelude should expose Fabric as the obvious normal path"
    );
    assert!(
        builder.contains("pub fn resource(")
            && builder.contains("pub fn system(")
            && builder.contains("pub fn component(")
            && builder.contains("pub fn select_provider(")
            && builder.contains("pub fn with_block(")
            && builder.contains("pub fn block")
            && builder.contains("pub fn build("),
        "Fabric should own the full normal typed authoring recipe"
    );
    assert!(
        !builder.contains("pub fn adapter("),
        "Fabric must not offer a standalone Adapter contribution"
    );
    assert!(
        builder.contains("fabric.sdk.default"),
        "normal typed contributions should lower into one deterministic grouping-only Block"
    );
    assert!(
        builder.contains("pub struct Composition")
            && builder.contains("core: CoreComposition")
            && builder.contains("pub fn id(&self) -> &CompositionId")
            && builder.contains("pub fn core(&self) -> &CoreComposition")
            && builder.contains("pub fn manifest(&self) -> &FabricManifest")
            && builder.contains("pub fn into_core(self) -> CoreComposition")
            && !builder.contains("BuiltFabric")
            && !builder.contains("pub fn into_composition")
            && !builder.contains("pub fn into_parts(self) -> (CoreComposition, FabricManifest)"),
        "SDK Composition must own private Core and Manifest views without legacy aliases"
    );
    assert!(
        resource.contains("into_bridge_parts()") && system.contains("into_bridge_parts()"),
        "typed realization contributions must perform the provider decomposition internally"
    );
    assert!(
        builder.contains("component.into_parts()"),
        "component contributions must lower declaration, attachment, and dependency carriers together"
    );
    assert!(
        manifest.contains("pub struct ResourceManifestEntry")
            && manifest.contains("pub struct SystemManifestEntry")
            && manifest.contains("pub struct FabricManifestDiagnostics")
            && manifest.contains("pub fn component_resource_bindings(")
            && manifest.contains("pub fn diagnostics("),
        "FabricManifest should separate semantic facts from raw diagnostics"
    );
    assert!(
        !manifest.contains("pub fn ") || !manifest.contains("&mut self"),
        "FabricManifest must expose no mutating APIs"
    );
    let fabric_instance = fs::read_to_string(crate_root().join("src/authoring/fabric_instance.rs"))
        .expect("read FabricInstance source");
    assert!(
        composition_ext.contains("impl CompositionExt for CoreComposition")
            && !composition_ext.contains("impl CompositionExt for crate::Composition")
            && fabric_instance.contains("impl Composition")
            && fabric_instance.contains("pub struct FabricInstance")
            && fabric_instance.contains("pub fn materialize_named")
            && fabric_instance.contains("pub fn components"),
        "SDK Composition must materialize the bounded high-level FabricInstance while CompositionExt remains raw"
    );
    for source in [&builder, &manifest, &resource, &system] {
        for forbidden in [
            "ResolutionRequirements",
            "ResourceSpec",
            "ResourceRef",
            "ResourceMatch",
            "TargetSpec",
            "Properties",
            "downcast",
            "NoMatchingResource",
            "AmbiguousResource",
            "MissingSystem",
            "AdapterDiscovery",
            "serde_json",
            "dyn Any",
            "resolve_provider",
        ] {
            assert!(
                !source.contains(forbidden),
                "normal authoring must not leak donor resolution machinery, found {forbidden}"
            );
        }
    }
}

#[test]
fn sdk_reexports_the_current_package_macro_frontends_only() {
    let lib = fs::read_to_string(crate_root().join("src/lib.rs")).expect("read sdk lib");
    let prelude = fs::read_to_string(crate_root().join("src/prelude.rs")).expect("read prelude");

    assert!(
        lib.contains("pub use fabric_sdk_macros::{adapter, component, resource, system};"),
        "fabric should re-export adapter!, component!, resource!, and system!"
    );
    assert!(
        ["adapter", "component", "resource", "system"]
            .iter()
            .all(|macro_name| prelude.contains(macro_name)),
        "sdk prelude should expose the supported package macro frontends"
    );
    let forbidden = "fabric_sdk_macros::host";
    assert!(
        !lib.contains(forbidden),
        "fabric must not re-export unsupported macro frontend {forbidden}"
    );
}

#[test]
fn normal_prelude_quarantines_legacy_and_raw_machinery() {
    let prelude = fs::read_to_string(crate_root().join("src/prelude.rs")).expect("read prelude");
    let lib = fs::read_to_string(crate_root().join("src/lib.rs")).expect("read sdk lib");
    for forbidden in [
        "BindingId",
        "BindingConsumer",
        "MaterializedProjection",
        "Projector",
        "ModuleBindings",
        "ModuleRuntime",
        "InvocationRail",
        "OperationRail",
        "ComponentHostModule",
        "BlockAuthor",
        "FabricBuilder",
        "CompositionExt",
    ] {
        assert!(
            !prelude.contains(forbidden),
            "normal prelude must not expose {forbidden}"
        );
    }
    assert!(lib.contains("pub mod experimental;"));
    assert!(!lib.contains("pub use experimental::projection"));
    assert!(!lib.contains("pub mod binding;") && !lib.contains("pub mod resource_registry;"));
    for raw in [
        "ModuleRuntime",
        "ModuleDeclaration",
        "ModuleBindings",
        "ContractProviderSelection",
        "ComponentParticipationRealization",
        "ComponentParticipationScope",
    ] {
        let root_export = format!("pub use core::{{{raw}");
        assert!(
            !lib.contains(&root_export),
            "root surface must keep {raw} under its named module"
        );
    }
}

#[test]
fn sdk_exposes_a_curated_generic_system_surface() {
    let lib = fs::read_to_string(crate_root().join("src/lib.rs")).expect("read sdk lib");
    let adapter = fs::read_to_string(crate_root().join("src/adapter.rs")).expect("read adapter");
    let system = fs::read_to_string(crate_root().join("src/system.rs")).expect("read system");
    let prelude = fs::read_to_string(crate_root().join("src/prelude.rs")).expect("read prelude");
    let authoring =
        fs::read_to_string(crate_root().join("src/authoring/mod.rs")).expect("read authoring mod");
    let system_authoring = fs::read_to_string(crate_root().join("src/authoring/system/mod.rs"))
        .expect("read system authoring mod");
    let adaptable = fs::read_to_string(
        crate_root().join("src/authoring/system/adaptable_system_definition.rs"),
    )
    .expect("read adaptable system definition");
    let realization =
        fs::read_to_string(crate_root().join("src/authoring/system/system_realization.rs"))
            .expect("read system realization");
    let selection =
        fs::read_to_string(crate_root().join("src/authoring/system/system_selection.rs"))
            .expect("read system selection");
    let definition =
        fs::read_to_string(crate_root().join("src/authoring/system/system_definition.rs"))
            .expect("read system definition");

    assert!(
        lib.contains("pub mod system;"),
        "fabric should expose a dedicated generic system facade"
    );
    assert!(
        lib.contains("pub mod adapter;"),
        "fabric should expose a dedicated generic adapter facade"
    );
    assert!(
        adapter.contains("pub use crate::authoring::{AdapterDefinition, AdapterProviderModule};"),
        "sdk adapter facade should re-export the target-neutral adapter authoring surface"
    );
    assert!(
        system.contains("SystemId")
            && system.contains("SystemSchemaDescriptor")
            && system.contains("SystemCompatibilityError")
            && system.contains("AdapterSystemSchemaSupport"),
        "sdk system facade should re-export curated raw system types"
    );
    assert!(
        !prelude.contains("PrimarySystemContract")
            && !prelude.contains("SystemDefinition")
            && !prelude.contains("SystemRequires")
            && !prelude.contains("AdaptableSystemDefinition")
            && !prelude.contains("SystemRealization"),
        "normal prelude must not expose handwritten system/provider machinery"
    );
    assert!(
        authoring.contains("AdaptableSystemDefinition")
            && authoring.contains("SystemRealization")
            && authoring.contains("PrimarySystemContract"),
        "sdk authoring surface should own handwritten system definitions and realizations"
    );
    assert!(
        system_authoring.contains("mod adaptable_system_definition;")
            && system_authoring.contains("mod system_realization;"),
        "system authoring module should own the adaptable-system and realization bridge"
    );
    assert!(
        adaptable.contains("type RealizationContract: Send + Sync + 'static;")
            && adaptable.contains("fn realization_requirement() -> ContractRequirement"),
        "AdaptableSystemDefinition should own the system realization contract anchor"
    );
    assert!(
        realization.contains("A: AdapterDefinition<Target = S>")
            && realization.contains("SystemAdapterCompatibility<S>")
            && realization.contains("ContractProviderSelection"),
        "SystemRealization should preserve target-aware adapter typing and raw provider selection"
    );
    assert!(
        !selection.contains("name:")
            && !selection.contains("IntoResourceName")
            && selection.contains("fabric.system."),
        "SystemSelection must stay occurrence-free and derive module ids only from SystemId"
    );
    assert!(
        selection.contains("pub fn using<A>(self, adapter: A)")
            && selection.contains("A: AdapterDefinition<Target = S>")
            && selection.contains("SystemAdapterCompatibility<S>"),
        "SystemSelection::using should be available only for system-target adapters with target-aware support"
    );
    assert!(
        selection.contains("ContractProviderSelection::new")
            && selection.contains("S::realization_requirement().id().clone()"),
        "system realization should lower into the same core provider-selection primitive"
    );
    assert!(
        !selection.contains("AdapterResourceSchemaSupport"),
        "system realization authoring must not reuse resource schema support"
    );
    assert!(
        definition.contains("fn materialize(selection: &SystemSelection<Self>)"),
        "SystemDefinition must own one canonical system materialization bridge"
    );
    assert!(
        definition.contains("-> Option<Box<dyn ModuleRuntime>>"),
        "SystemDefinition runtime realization must stay optional for declaration-only Systems"
    );
    assert!(
        definition.contains("None"),
        "SystemDefinition must default to no runtime materializer"
    );
    for forbidden in [
        "DeclarativeSystemDefinition",
        "RuntimeSystemDefinition",
        "ExecutableSystemDefinition",
        "MaterializableSystemDefinition",
        "SingletonSystem",
        "SharedSystem",
        "ScopedSystem",
    ] {
        assert!(
            !definition.contains(forbidden),
            "SDK must not split Systems into parallel definition families, found {forbidden}"
        );
    }
}

#[test]
fn sdk_trybuild_covers_hostile_component_identity_override() {
    let compile_fail =
        fs::read_to_string(crate_root().join("../tests/sdk-api/tests/compile_fail.rs"))
            .expect("read sdk-api compile_fail test");

    assert!(
        compile_fail.contains("hostile_component_runtime_definition_override.rs"),
        "sdk-api trybuild suite should guard against hostile component runtime overrides"
    );
}

#[test]
fn sdk_docs_describe_the_current_public_contract() {
    let readme = fs::read_to_string(crate_root().join("README.md")).expect("read readme");

    assert!(
        readme.contains("ExampleAdapter for ExampleResource")
            && readme.contains("target determines whether it is a Resource or System")
            && readme.contains("names no generated\ninterface"),
        "SDK README must document target-derived canonical Adapter authoring for both targets"
    );
    assert!(
        readme.contains("Canonical `component!` declaration")
            && readme.contains("Relation fields are Component-local roles")
            && readme.contains("were removed in 0.5.4"),
        "SDK README must document canonical declaration roles and the hard-cut migration"
    );
    assert!(
        readme.contains("advanced invocation API")
            && readme.contains("Result<Result<Document, DocumentError>, ComponentError>"),
        "SDK README must separate advanced invocation provenance from nested domain results"
    );
    assert!(
        readme.contains("`FabricManifest` is immutable semantic Composition inspection")
            && readme.contains("composition.manifest().diagnostics()"),
        "SDK README must separate semantic Manifest inspection from raw diagnostics"
    );
    assert!(
        readme.contains("use fabric::authoring::FabricBuilder;"),
        "SDK README must keep FabricBuilder on the explicit advanced path"
    );
    let internal_labels = (1..=5)
        .map(|number| format!("D{}{}", "X", number))
        .chain(std::iter::once("future SDK".to_owned()));
    for forbidden in internal_labels {
        assert!(
            !readme.contains(&forbidden),
            "SDK README must not present internal development labels as current API: {forbidden}"
        );
    }
}

#[test]
fn current_component_docs_do_not_reteach_pre_purification_vocabulary() {
    for file in [
        "../docs/architecture.md",
        "../docs/concepts/component.md",
        "../docs/concepts/adapter.md",
        "../docs/concepts/realization.md",
        "../docs/concepts/state.md",
        "README.md",
    ] {
        let source =
            fs::read_to_string(crate_root().join(file)).expect("read current documentation");
        for stale in [
            "Components declare behavior as operations",
            "ComponentRuntimeDefinition",
            "Canonical Component runtime authoring is intentionally deferred",
            "handwritten\n`AdapterDefinition` remains the public advanced path for Component targets",
        ] {
            assert!(
                !source.contains(stale),
                "{file} must not teach stale Component vocabulary: {stale}"
            );
        }
    }
}

#[test]
fn sdk_trybuild_covers_component_macro_frontend_integrity() {
    let compile_fail = fs::read_to_string(
        crate_root().join("../tests/extensions/component-greeter/tests/compile_fail.rs"),
    )
    .expect("read component greeter compile_fail test");
    let ui_dir = crate_root().join("../tests/extensions/component-greeter/tests/ui");

    assert!(
        compile_fail.contains("removed_operations.rs"),
        "component trybuild suite should execute canonical and removal ui fixtures"
    );
    for case in [
        "duplicate_component_api_method.rs",
        "removed_operations.rs",
        "removed_requires.rs",
        "removed_system.rs",
        "removed_teardown.rs",
    ] {
        assert!(
            ui_dir.join(case).exists(),
            "component trybuild suite should guard {case} with a concrete ui fixture"
        );
    }
}

#[test]
fn sdk_trybuild_covers_direct_only_resources_without_adapter_paths() {
    let compile_fail =
        fs::read_to_string(crate_root().join("../tests/sdk-api/tests/compile_fail.rs"))
            .expect("read sdk-api compile_fail test");

    assert!(
        compile_fail.contains("direct_only_resource_using_adapter.rs"),
        "sdk-api trybuild suite should guard direct-only resources from adapter authoring"
    );
}

#[test]
fn generic_fabric_crates_do_not_depend_back_on_sdk() {
    let repo_root = crate_root().join("..").canonicalize().expect("repo root");
    for manifest in [
        repo_root.join("core/Cargo.toml"),
        repo_root.join("host/Cargo.toml"),
        repo_root.join("resource/Cargo.toml"),
        repo_root.join("experimental/binding/Cargo.toml"),
        repo_root.join("component/Cargo.toml"),
    ] {
        let source = fs::read_to_string(&manifest).expect("read manifest");
        assert!(
            !source.contains("\nfabric =") && !source.contains("\nfabric="),
            "{} must not declare a dependency on the primary fabric crate",
            manifest.display()
        );
    }
}

#[test]
fn sdk_trybuild_covers_adapter_target_integrity_across_resource_and_system_planes() {
    let compile_fail =
        fs::read_to_string(crate_root().join("../tests/sdk-api/tests/compile_fail.rs"))
            .expect("read sdk-api compile_fail test");

    for case in [
        "resource_adapter_cannot_target_system.rs",
        "system_adapter_cannot_target_resource.rs",
        "resource_target_adapter_with_system_schema_support.rs",
        "system_target_adapter_with_resource_schema_support.rs",
    ] {
        assert!(
            compile_fail.contains(case),
            "sdk-api trybuild suite should guard {case}"
        );
    }
}
