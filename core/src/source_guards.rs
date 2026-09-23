use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn contract_identity_does_not_encode_versions_into_ids() {
    let contract = fs::read_to_string(crate_root().join("src/contract.rs"))
        .expect("read core contract source");
    let composition = fs::read_to_string(crate_root().join("src/composition/mod.rs"))
        .expect("read core composition source");

    for source in [contract, composition] {
        assert!(
            !source.contains(".v1")
                && !source.contains(".v2")
                && !source.contains("@1")
                && !source.contains("@2"),
            "contract identity grammar must not encode versions into contract ids"
        );
    }
}

#[test]
fn core_keeps_canonical_resolution_seams() {
    let manifest = fs::read_to_string(crate_root().join("Cargo.toml")).expect("read core manifest");
    let bindings = fs::read_to_string(crate_root().join("src/composition/bindings.rs"))
        .expect("read binding source");
    let contract = fs::read_to_string(crate_root().join("src/contract.rs"))
        .expect("read core contract source");
    let composition = fs::read_to_string(crate_root().join("src/composition/mod.rs"))
        .expect("read core composition source");
    let module =
        fs::read_to_string(crate_root().join("src/module.rs")).expect("read core module source");
    let host_materialization = fs::read_to_string(crate_root().join("src/host_materialization.rs"))
        .expect("read host materialization source");
    let runtime = fs::read_to_string(crate_root().join("src/module_runtime.rs"))
        .expect("read module runtime source");

    assert!(bindings.contains("pub fn resolve_with_provider"));
    assert!(bindings.contains("pub fn identity(&self) -> &ContractIdentity"));
    assert!(
        manifest.contains("fabric-host"),
        "fabric core should depend narrowly on fabric-host for explicit materialization truth"
    );
    assert!(
        !contract.contains("pub fn new(key: ContractKey")
            && !contract.contains("pub fn from_key(key: ContractKey")
            && !contract.contains("pub fn new(id: ContractId) -> Self"),
        "public contract APIs must not recreate provider-key or implicit provisional constructors"
    );
    assert!(
        composition.contains("pub fn materialize_on("),
        "core composition should expose explicit host-aware materialization"
    );
    assert!(
        composition.contains("resolution.start_order.iter()"),
        "runtime binding must follow the resolved dependency order, not authoring order"
    );
    let built_composition = composition
        .split("pub struct Composition {")
        .nth(1)
        .and_then(|source| source.split("impl std::fmt::Debug for Composition").next())
        .expect("Composition definition");
    assert!(
        composition.contains("struct CompositionResolution")
            && built_composition.contains("resolution: CompositionResolution")
            && !built_composition.contains("provider_selections"),
        "a built Composition must retain only frozen declarative resolution, not raw selections"
    );
    assert!(
        !composition.contains("validate_declarations(&self.blocks")
            && !composition.contains("resolve_runtime_exports("),
        "materialization must consume frozen declaration and export bindings rather than resolve again"
    );
    assert!(
        !composition.contains("HostDescriptor::native"),
        "core must not silently materialize against an ambient native host"
    );
    assert!(
        module.contains("pub struct ModuleDeclaration")
            && module
                .contains("fn host_requirement(&self) -> Option<&HostMaterializationRequirement>"),
        "raw Module declarations should carry an optional host materialization declaration seam"
    );
    assert!(
        host_materialization.contains("pub struct HostMaterializationRequirement"),
        "core should own a structured host materialization declaration type"
    );
    assert!(
        !runtime.contains("HostDescriptor")
            && !runtime.contains("HostRequirement")
            && !runtime.contains("HostMaterializationRequirement"),
        "module runtime must stay free of ambient host compatibility state"
    );
    for source in [composition, module, host_materialization] {
        for forbidden in [
            "AdapterDefinition",
            "ResourceDefinition",
            "SystemDefinition",
            "AdapterResourceSchemaSupport",
            "AdapterSystemSchemaSupport",
            "HostRegistry",
            "HostServiceLocator",
        ] {
            assert!(
                !source.contains(forbidden),
                "fabric core host materialization seam must stay generic and free of {forbidden}"
            );
        }
    }
}

#[test]
fn declarations_are_structural_and_runtimes_remain_live_only() {
    let module =
        fs::read_to_string(crate_root().join("src/module.rs")).expect("read core module source");
    let runtime = fs::read_to_string(crate_root().join("src/module_runtime.rs"))
        .expect("read module runtime source");

    for forbidden in [
        "fn provided_contracts(",
        "fn required_contracts(",
        "fn optional_contracts(",
    ] {
        assert!(
            !runtime.contains(forbidden),
            "legacy contract-id module metadata seam must stay removed: {forbidden}"
        );
    }

    for required in [
        "pub struct ModuleDeclaration",
        "fn declaration(&self) -> ModuleDeclaration",
        "fn materialize(&self) -> Option<Box<dyn ModuleRuntime>>",
    ] {
        assert!(
            module.contains(required),
            "canonical declaration seam must stay present: {required}"
        );
    }
    assert!(
        runtime.contains("fn export_contracts(")
            && runtime.contains("fn bind(")
            && runtime.contains("fn initialize(")
            && runtime.contains("fn health(&self) -> Health"),
        "ModuleRuntime must remain the live export, binding, lifecycle, and health boundary"
    );
}

#[test]
fn core_public_api_no_longer_exports_block_error() {
    let lib = fs::read_to_string(crate_root().join("src/lib.rs")).expect("read core lib source");
    let block = fs::read_to_string(crate_root().join("src/block.rs")).expect("read block source");
    let error = fs::read_to_string(crate_root().join("src/error.rs")).expect("read error source");

    assert!(
        !lib.contains("BlockError"),
        "core crate root must not export BlockError"
    );
    assert!(
        !error.contains("pub enum BlockError"),
        "BlockError must not remain a public core error type"
    );
    assert!(
        block.contains("pub fn build(self) -> Block"),
        "BlockBuilder::build must stay infallible while block construction has no error path"
    );
}

#[test]
fn composition_and_instance_public_api_use_closed_error_vocabulary() {
    let composition = fs::read_to_string(crate_root().join("src/composition/mod.rs"))
        .expect("read composition source");
    let instance =
        fs::read_to_string(crate_root().join("src/instance.rs")).expect("read instance source");

    assert!(
        composition.contains("pub fn build(self) -> Result<Composition, CompositionError>"),
        "CompositionBuilder::build must expose CompositionError"
    );
    assert!(
        composition.contains("pub fn materialize(&self, instance_id: InstanceId) -> Result<Instance, CompositionError>"),
        "Composition::materialize must expose CompositionError"
    );
    assert!(
        instance.contains("pub fn start(&mut self) -> Result<(), InstanceError>"),
        "Instance::start must expose InstanceError"
    );
    assert!(
        instance.contains("pub fn stop(&mut self) -> Result<(), RuntimeCleanupError>"),
        "Instance::stop must expose runtime cleanup failures"
    );
    assert!(
        !composition.contains("BlockError") && !instance.contains("BlockError"),
        "Composition and Instance public APIs must not leak BlockError"
    );
}

#[test]
fn provider_selection_stays_a_generic_contract_resolution_primitive() {
    let composition = fs::read_to_string(crate_root().join("src/composition/mod.rs"))
        .expect("read composition source");
    let selection = fs::read_to_string(crate_root().join("src/composition/selection.rs"))
        .expect("read selection source");

    assert!(
        composition.contains(
            "pub fn select_provider(mut self, selection: ContractProviderSelection) -> Self"
        ),
        "CompositionBuilder should expose explicit provider selection"
    );
    assert!(
        composition.contains("let selected_provider = provider_selections"),
        "provider selection should participate in resolution"
    );
    assert!(
        composition.contains("validate_selected_providers"),
        "composition validation should structurally validate explicit provider selections"
    );
    assert!(
        !composition.contains("highest")
            && !composition.contains("max_by")
            && !composition.contains("last_write_wins"),
        "provider selection must not invent highest-version or last-write provider choice"
    );
    for source in [composition, selection] {
        for forbidden in [
            "Resource",
            "Adapter",
            "Component",
            "Clock",
            "MemoryClock",
            "Database",
            "SQLite",
            "BlockScope",
        ] {
            assert!(
                !source.contains(forbidden),
                "provider selection code must stay generic and free of {forbidden}"
            );
        }
    }
}

#[test]
fn repository_boundary_contains_only_kernel_production_roots() {
    let repo_root = crate_root().join("..").canonicalize().expect("repo root");
    assert!(
        !repo_root.join("resources").exists(),
        "kernel repository must not retain a top-level resources directory"
    );
    assert!(
        !repo_root.join("adapters").exists(),
        "kernel repository must not retain a top-level adapters directory"
    );
    assert!(
        !repo_root.join("components").exists(),
        "kernel repository must not retain a top-level components directory"
    );

    let manifest = fs::read_to_string(repo_root.join("Cargo.toml")).expect("read root manifest");
    for forbidden in [
        "fabric-adapter-",
        "fabric-resource-worker",
        "fabric-resource-server",
        "fabric-resource-process",
        "fabric-component-gateway",
        "fabric-component-namespace",
        "fabric-component-publication",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "root manifest must stay free of removed official package {forbidden}"
        );
    }
}
