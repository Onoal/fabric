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
fn component_source_stays_headless_and_foundational() {
    for file in [
        "/src/component.rs",
        "/src/communication.rs",
        "/src/control.rs",
        "/src/contract.rs",
        "/src/error.rs",
        "/src/invocation.rs",
        "/src/lifecycle.rs",
        "/src/native/module.rs",
        "/src/operation/definition.rs",
        "/src/operation/id.rs",
        "/src/operation/key.rs",
        "/src/operation/mod.rs",
        "/src/operation/type_id.rs",
        "/src/operations.rs",
        "/src/registry.rs",
        "/src/surface.rs",
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
        "Component host lifecycle must not encode health"
    );
}

#[test]
fn component_resolution_api_stays_sealed_against_public_synthetic_bypasses() {
    let communication = fs::read_to_string(format!(
        "{}/src/communication.rs",
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
        .join("..")
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
    let module = fs::read_to_string(format!("{manifest_dir}/src/native/module.rs"))
        .expect("native module source");
    let state = fs::read_to_string(format!("{manifest_dir}/src/native/module/state.rs"))
        .expect("native state source");
    let operation_runtime = fs::read_to_string(format!(
        "{manifest_dir}/src/native/module/operation_runtime.rs"
    ))
    .expect("operation runtime source");
    let error = fs::read_to_string(format!("{manifest_dir}/src/error.rs")).expect("error source");
    let reconstruction = fs::read_to_string(format!("{manifest_dir}/src/reconstruction.rs"))
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
        "native host must not derive Component existence from runtime attachments"
    );
    assert!(
        module.contains("MissingComponentRuntimeAttachment"),
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
