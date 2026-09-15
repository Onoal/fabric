use std::fs;
use std::path::Path;

#[test]
fn raw_api_witness_has_no_official_concrete_package_dependency() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read raw-api manifest");

    for forbidden in [
        "fabric-resource-worker",
        "fabric-resource-server",
        "fabric-resource-process",
        "fabric-adapter-",
        "fabric-component-gateway",
        "fabric-component-namespace",
        "fabric-component-publication",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "raw-api witness must stay free of official concrete package dependency {forbidden}"
        );
    }
}
