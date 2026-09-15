use std::fs;
use std::path::Path;

#[test]
fn host_manifest_stays_foundational_and_semantic_free() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("manifest");
    for forbidden in [
        "fabric-core",
        "fabric-resource",
        "fabric-component",
        "fabric-adapter",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "fabric-host must not depend on semantic package {forbidden}"
        );
    }
}

#[test]
fn host_source_does_not_become_a_service_locator() {
    for file in [
        "src/lib.rs",
        "src/descriptor.rs",
        "src/error.rs",
        "src/identifier.rs",
        "src/requirement.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(file)).expect("source");
        for forbidden in [
            "HostRegistry",
            "HostContext::get",
            "dyn Any",
            "service locator",
        ] {
            assert!(
                !source.contains(forbidden),
                "{file} must not introduce ambient host access via {forbidden}"
            );
        }
    }
}
