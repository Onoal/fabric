use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn canonical_distribution_packages_keep_the_fabric_rust_crate_names() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repository root");
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--manifest-path",
        ])
        .arg(repository.join("Cargo.toml"))
        .output()
        .expect("run cargo metadata");
    assert!(output.status.success(), "cargo metadata should succeed");
    let metadata = String::from_utf8(output.stdout).expect("metadata utf-8");
    let workspace_manifest =
        fs::read_to_string(repository.join("Cargo.toml")).expect("read workspace manifest");

    for (package, library) in [
        ("onoal-fabric-host", "fabric_host"),
        ("onoal-fabric-core", "fabric_core"),
        ("onoal-fabric-resource", "fabric_resource"),
        ("onoal-fabric-system", "fabric_system"),
        ("onoal-fabric-component", "fabric_component"),
        ("onoal-fabric-sdk-macros", "fabric_sdk_macros"),
        ("onoal-fabric", "fabric"),
    ] {
        assert!(metadata.contains(&format!("\"name\":\"{package}\"")));
        assert!(metadata.contains(&format!("\"name\":\"{library}\"")));
    }

    for manifest in [
        "host/Cargo.toml",
        "core/Cargo.toml",
        "resource/Cargo.toml",
        "system/Cargo.toml",
        "component/Cargo.toml",
        "sdk-macros/Cargo.toml",
        "sdk/Cargo.toml",
    ] {
        let source = fs::read_to_string(repository.join(manifest)).expect("read public manifest");
        assert!(
            source.contains("publish = [\"crates-io\"]"),
            "canonical package must explicitly allow crates.io publication: {manifest}"
        );
        assert!(
            source.contains("license.workspace = true")
                && source.contains("repository.workspace = true")
                && source.contains("description ="),
            "canonical package must inherit release metadata: {manifest}"
        );
    }

    for dependency in [
        "fabric-host = { package = \"onoal-fabric-host\", version = \"0.1.0\", path = \"host\" }",
        "fabric-core = { package = \"onoal-fabric-core\", version = \"0.1.0\", path = \"core\" }",
        "fabric-resource = { package = \"onoal-fabric-resource\", version = \"0.1.0\", path = \"resource\" }",
        "fabric-system = { package = \"onoal-fabric-system\", version = \"0.1.0\", path = \"system\" }",
        "fabric-component = { package = \"onoal-fabric-component\", version = \"0.1.0\", path = \"component\" }",
        "fabric-sdk-macros = { package = \"onoal-fabric-sdk-macros\", version = \"0.1.1\", path = \"sdk-macros\" }",
        "fabric = { package = \"onoal-fabric\", version = \"0.1.1\", path = \"sdk\" }",
    ] {
        assert!(
            workspace_manifest.contains(dependency),
            "canonical sibling dependency must retain package alias, version, and local path: {dependency}"
        );
    }

    assert!(
        !workspace_manifest.contains("fabric-sdk = {"),
        "the primary Fabric dependency key must remain fabric"
    );

    for document in [
        "README.md",
        "sdk/README.md",
        "docs/README.md",
        "docs/getting-started.md",
        "docs/concepts/README.md",
        "docs/concepts/composition.md",
        "docs/concepts/instance.md",
        "docs/concepts/component.md",
        "docs/concepts/resource.md",
        "docs/concepts/system.md",
        "docs/architecture.md",
        "docs/advanced/raw-api.md",
    ] {
        let source = fs::read_to_string(repository.join(document)).expect("read public document");
        assert!(
            !source.contains("fabric_sdk") && !source.contains("fabric-sdk"),
            "public primary-crate documentation must use fabric: {document}"
        );
    }

    for manifest in ["sdk-macros/Cargo.toml", "sdk/Cargo.toml"] {
        let source =
            fs::read_to_string(repository.join(manifest)).expect("read corrected manifest");
        assert!(
            source.contains("version = \"0.1.1\"") && !source.contains("version.workspace = true"),
            "only the corrected public package must carry its explicit 0.1.1 version: {manifest}"
        );
    }

    for private_package in [
        "fabric-binding",
        "fabric-projection",
        "fabric-resource-registry",
        "fabric-test-sdk-api",
        "fabric-test-raw-api",
    ] {
        assert!(metadata.contains(&format!("\"name\":\"{private_package}\"")));
    }
    for manifest in [
        "experimental/binding/Cargo.toml",
        "experimental/projection/Cargo.toml",
        "experimental/resource-registry/Cargo.toml",
        "tests/raw-api/Cargo.toml",
        "tests/sdk-api/Cargo.toml",
        "tests/external-extension/Cargo.toml",
        "tests/extensions/adapter-clock-memory/Cargo.toml",
        "tests/extensions/adapter-external/Cargo.toml",
        "tests/extensions/component-greeter/Cargo.toml",
        "tests/extensions/resource-clock/Cargo.toml",
        "tests/extensions/resource-counter/Cargo.toml",
        "tests/extensions/system-operations/Cargo.toml",
    ] {
        let source = fs::read_to_string(repository.join(manifest)).expect("read private manifest");
        assert!(
            source.contains("publish = false"),
            "experimental and test package must remain private: {manifest}"
        );
    }
}

#[test]
fn sdk_api_fixture_has_no_official_concrete_package_dependency() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read sdk-api manifest");

    for forbidden in [
        "fabric-resource-worker",
        "fabric-resource-server",
        "fabric-resource-process",
        "fabric-adapter-worker-deno",
        "fabric-adapter-server-deno",
        "fabric-adapter-process-systemd",
        "fabric-component-gateway",
        "fabric-component-namespace",
        "fabric-component-publication",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "sdk-api fixture must stay free of official concrete package dependency {forbidden}"
        );
    }
}

#[test]
fn sdk_api_fixture_depends_on_the_synthetic_system_extension_not_official_system_packages() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read sdk-api manifest");

    assert!(
        manifest.contains("fabric-test-system-operations"),
        "sdk-api fixture should exercise the synthetic system extension"
    );
    assert!(
        !manifest.contains("fabric-adapter")
            && !manifest.contains("system!")
            && !manifest.contains("fabric-packages"),
        "sdk-api fixture must not depend on unrelated system machinery"
    );
}

#[test]
fn sdk_api_trybuild_suite_covers_resource_and_system_adapter_target_mismatches() {
    let compile_fail =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/compile_fail.rs"))
            .expect("read compile_fail suite");

    for case in [
        "resource_adapter_cannot_target_system.rs",
        "system_adapter_cannot_target_resource.rs",
        "resource_target_adapter_with_system_schema_support.rs",
        "system_target_adapter_with_resource_schema_support.rs",
    ] {
        assert!(
            compile_fail.contains(case),
            "sdk-api compile-fail coverage should include {case}"
        );
    }
}

#[test]
fn canonical_sdk_and_extension_fixtures_do_not_depend_on_the_experimental_registry() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repository root");
    for manifest in [
        repository.join("sdk/Cargo.toml"),
        repository.join("tests/external-extension/Cargo.toml"),
        repository.join("tests/extensions/resource-clock/Cargo.toml"),
    ] {
        let source = fs::read_to_string(&manifest).expect("read canonical witness manifest");
        assert!(
            !source.contains("fabric-resource-registry"),
            "canonical SDK/extension manifest must not depend on the experimental registry: {}",
            manifest.display()
        );
    }

    for entry in fs::read_dir(repository.join("tests/extensions")).expect("read extensions") {
        let manifest = entry.expect("extension entry").path().join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        let source = fs::read_to_string(&manifest).expect("read extension manifest");
        assert!(
            !source.contains("fabric-resource-registry"),
            "canonical extension manifest must not depend on the experimental registry: {}",
            manifest.display()
        );
    }
}

#[test]
fn public_tree_uses_neutral_documentation_and_macro_diagnostics() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repository root");

    assert!(repository.join("docs/architecture.md").exists());
    assert!(repository.join("docs/advanced/raw-api.md").exists());
    assert!(!repository.join("PROVENANCE.md").exists());

    for document in [
        repository.join("README.md"),
        repository.join("docs/README.md"),
        repository.join("docs/getting-started.md"),
        repository.join("docs/concepts/README.md"),
        repository.join("docs/concepts/composition.md"),
        repository.join("docs/concepts/instance.md"),
        repository.join("docs/concepts/component.md"),
        repository.join("docs/concepts/resource.md"),
        repository.join("docs/concepts/system.md"),
        repository.join("docs/architecture.md"),
        repository.join("docs/advanced/raw-api.md"),
        repository.join("sdk/README.md"),
    ] {
        let source = fs::read_to_string(&document).expect("read public document");
        for forbidden in ["DX2", "DX3", "DX4", "DX5", "PROVENANCE.md"] {
            assert!(
                !source.contains(forbidden),
                "public document must not expose internal development label {forbidden}: {}",
                document.display()
            );
        }
    }

    let validation = fs::read_to_string(repository.join("sdk-macros/src/validate.rs"))
        .expect("read macro validation");
    for forbidden in ["DX2", "DX3", "DX4", "DX5"] {
        assert!(
            !validation.contains(forbidden),
            "macro diagnostics must not expose internal development label {forbidden}"
        );
    }
}
