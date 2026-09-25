use std::fs;

#[test]
fn component_manifest_depends_only_on_fabric_core() {
    let manifest =
        fs::read_to_string(format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"))).expect("manifest");
    let dependency_keys: Vec<&str> = manifest
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(name, _)| name.trim().trim_end_matches(".workspace"))
        .collect();

    assert!(dependency_keys.contains(&"fabric-core"));
    assert!(!dependency_keys.contains(&"fabric-host"));
    for forbidden in [
        "fabric-auth",
        "fabric-apps",
        "fabric-home",
        "fabric-local-host",
        "steld",
        "fabric-client-contracts",
        "fabric-adapter-worker-deno",
        "fabric-adapter-ingress-pingora",
    ] {
        assert!(
            !dependency_keys.contains(&forbidden),
            "component manifest must not depend on {forbidden}"
        );
    }
}

#[test]
fn component_public_families_distinguish_declaration_participation_and_operator_ownership() {
    let root = fs::read_to_string(format!("{}/src/lib.rs", env!("CARGO_MANIFEST_DIR")))
        .expect("component root");
    for family in [
        "pub mod declaration",
        "pub mod participation",
        "pub mod invocation",
        "pub mod operator",
        "pub mod advanced",
    ] {
        assert!(
            root.contains(family),
            "component crate must expose {family}"
        );
    }
    let binding = fs::read_to_string(format!(
        "{}/src/participation/binding.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("component binding");
    assert!(
        binding.contains("pub struct ComponentInstanceBinding")
            && !binding.contains("pub struct Component\n"),
        "the Instance-bound identity must not be named as the semantic Component"
    );
}

#[test]
fn component_host_renames_preserve_established_graph_identity_strings() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let sources = [
        fs::read_to_string(format!("{manifest_dir}/src/runtime/contract.rs"))
            .expect("host contract"),
        fs::read_to_string(format!("{manifest_dir}/src/operational.rs"))
            .expect("host handle contract"),
        fs::read_to_string(format!("{manifest_dir}/src/runtime/mod.rs")).expect("materializer"),
        fs::read_to_string(format!("{manifest_dir}/src/registry.rs")).expect("registry"),
        fs::read_to_string(format!("{manifest_dir}/src/invocation/operations.rs"))
            .expect("operations"),
        fs::read_to_string(format!("{manifest_dir}/src/invocation/mod.rs")).expect("invocation"),
        fs::read_to_string(format!("{manifest_dir}/src/readiness.rs")).expect("readiness"),
        fs::read_to_string(format!("{manifest_dir}/src/control/mod.rs")).expect("control"),
        fs::read_to_string(format!("{manifest_dir}/src/control/reconstruction.rs"))
            .expect("reconstruction"),
        fs::read_to_string(format!("{manifest_dir}/src/runtime/native/module.rs"))
            .expect("native host module"),
    ]
    .join("\n");

    for stable_id in [
        "fabric.component.runtime",
        "fabric.component.runtime-handle",
        "fabric.component.materializer",
        "fabric.component.registry",
        "fabric.component.operation",
        "fabric.component.operation.registrar",
        "fabric.component.invocation",
        "fabric.component.readiness",
        "fabric.component.control",
        "fabric.component.reconstruction",
        "fabric.component.runtime.unbound",
        "fabric.component.runtime.module",
    ] {
        assert!(
            sources.contains(stable_id),
            "Component host vocabulary changes must not rewrite stable graph identity {stable_id}"
        );
    }
}

#[test]
fn component_source_stays_headless_and_foundational() {
    for file in [
        "/src/declaration/component.rs",
        "/src/invocation/communication.rs",
        "/src/control/mod.rs",
        "/src/runtime/contract.rs",
        "/src/error.rs",
        "/src/invocation/mod.rs",
        "/src/lifecycle.rs",
        "/src/runtime/native/module.rs",
        "/src/operation/definition.rs",
        "/src/operation/id.rs",
        "/src/operation/key.rs",
        "/src/operation/mod.rs",
        "/src/operation/type_id.rs",
        "/src/invocation/operations.rs",
        "/src/registry.rs",
        "/src/invocation/surface.rs",
    ] {
        let source =
            fs::read_to_string(format!("{}{}", env!("CARGO_MANIFEST_DIR"), file)).expect("source");
        for forbidden in [
            "Auth",
            "Apps",
            "Clock",
            "Greeter",
            "HostDescriptor",
            "HostRequirement",
            "HostFacilityId",
            "Home",
            "LocalHost",
            "steld",
            "Deno",
            "Pingora",
            "Client",
            "PathBuf",
            "rusqlite",
            "Ory",
            "BetterAuth",
            "Hono",
            "Axum",
            "Pingora",
            "SocketAddr",
            "serde_json",
            "Http",
            "Ingress",
            "third-party.test-clock",
        ] {
            assert!(
                !source.contains(forbidden),
                "{file} leaked non-component term {forbidden}"
            );
        }
    }
}

#[test]
fn component_host_lifecycle_keeps_health_out_of_its_state_vocabulary() {
    let lifecycle = fs::read_to_string(format!("{}/src/lifecycle.rs", env!("CARGO_MANIFEST_DIR")))
        .expect("lifecycle source");
    assert!(
        lifecycle.contains("Starting,")
            && lifecycle.contains("Ready,")
            && lifecycle.contains("Stopping,")
            && lifecycle.contains("Stopped,")
    );
    assert!(
        !lifecycle.contains("Degraded,"),
        "ComponentInstanceBinding host lifecycle must not encode health"
    );
}

#[test]
fn component_resolution_api_stays_sealed_against_public_synthetic_bypasses() {
    let communication = fs::read_to_string(format!(
        "{}/src/invocation/communication.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("communication source");
    assert!(
        !communication.contains("pub fn bind_synthetic"),
        "production component API must not expose bind_synthetic"
    );
    assert!(
        communication.contains("pub fn bind_resolved"),
        "production component API must keep bind_resolved"
    );

    let requirement =
        fs::read_to_string(format!("{}/src/requirement.rs", env!("CARGO_MANIFEST_DIR")))
            .expect("requirement source");
    assert!(
        !requirement.contains("pub fn synthetic("),
        "production component API must not expose a public synthetic resolved requirement"
    );
    assert!(
        requirement.contains("pub(crate) fn synthetic("),
        "crate-private synthetic requirement support must stay test-only"
    );
    assert!(
        requirement.contains("pub fn register_resolved"),
        "production component API must keep register_resolved"
    );
}

#[test]
fn core_and_component_keep_the_canonical_resolved_contract_path() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root");
    let bindings = fs::read_to_string(repo_root.join("core/src/composition/bindings.rs"))
        .expect("bindings source");
    assert!(
        bindings.contains("pub fn resolve_with_provider"),
        "fabric core must keep resolve_with_provider as the provenance-preserving seam"
    );
}

#[test]
fn native_host_catalog_stays_declaration_driven() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let module = fs::read_to_string(format!("{manifest_dir}/src/runtime/native/module.rs"))
        .expect("native module source");
    let state = fs::read_to_string(format!("{manifest_dir}/src/runtime/native/module/state.rs"))
        .expect("native state source");
    let operation_runtime = fs::read_to_string(format!(
        "{manifest_dir}/src/runtime/native/module/operation_runtime.rs"
    ))
    .expect("operation runtime source");
    let error = fs::read_to_string(format!("{manifest_dir}/src/error.rs")).expect("error source");
    let reconstruction =
        fs::read_to_string(format!("{manifest_dir}/src/control/reconstruction.rs"))
            .expect("reconstruction source");

    assert!(
        module.contains("component_declarations")
            && module.contains("runtime_attachments")
            && state.contains("fn validated_catalog"),
        "native host must store declaration truth independently from attachments"
    );
    assert!(
        !module.contains("runtime_definitions: BTreeMap")
            && !module.contains("with_runtime_definitions"),
        "native host must not derive ComponentInstanceBinding existence from runtime attachments"
    );
    assert!(
        module.contains("MissingComponentParticipationRealization"),
        "materialization without attachment must report the missing attachment, not unknown identity"
    );
    assert!(
        !error.contains("UnknownComponentRuntimeDefinition")
            && !module.contains("UnknownComponentRuntimeDefinition")
            && !reconstruction.contains("MissingRuntimeDefinition"),
        "stale runtime-definition existence vocabulary must not survive the catalog split"
    );
    assert!(
        operation_runtime.contains("UndeclaredComponentOperation"),
        "handler registration must be validated against the owner's declaration"
    );
    assert!(
        reconstruction.contains("MissingRuntimeAttachment")
            && reconstruction.contains("UndeclaredComponent"),
        "reconstruction must distinguish missing attachment from undeclared identity"
    );
}
