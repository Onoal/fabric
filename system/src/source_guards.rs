use std::fs;
use std::path::Path;

fn crate_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn system_manifest_stays_generic_and_has_no_kernel_cycles() {
    let manifest = fs::read_to_string(crate_root().join("Cargo.toml")).expect("read manifest");
    let dependencies = manifest
        .split("[dev-dependencies]")
        .next()
        .expect("dependencies section");

    for required in ["semver", "thiserror"] {
        assert!(
            dependencies.contains(required),
            "fabric-system manifest must depend on {required}"
        );
    }
    for forbidden in [
        "fabric-core",
        "fabric-host",
        "fabric-resource-registry",
        "fabric =",
        "fabric-test",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "fabric-system manifest must stay free of {forbidden}"
        );
    }
}

#[test]
fn system_source_stays_generic_and_semantically_distinct() {
    for file in [
        "src/lib.rs",
        "src/error.rs",
        "src/model/mod.rs",
        "src/model/system.rs",
        "src/model/schema.rs",
    ] {
        let source = fs::read_to_string(crate_root().join(file)).expect("read source");
        for forbidden in [
            "SystemName",
            "SystemInstanceId",
            "CapabilityDefinition",
            "Clock",
            "Logging",
            "Scheduler",
            "Audit",
            "ResourceName",
        ] {
            assert!(
                !source.contains(forbidden),
                "{file} must stay free of {forbidden}"
            );
        }
    }
}
