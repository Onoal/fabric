use std::fs;
use std::path::Path;

#[test]
fn resource_schema_types_stay_distinct_from_contract_versions() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root");
    let core_contract =
        fs::read_to_string(repo_root.join("core/src/contract.rs")).expect("core contract source");
    let resource_schema =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/model/schema.rs"))
            .expect("resource schema source");

    assert!(
        !resource_schema.contains("type ResourceSchemaVersion = ContractVersion"),
        "resource schema version must not alias contract version"
    );
    assert!(
        !core_contract.contains("type ContractVersion ="),
        "contract version must not alias package version or another version axis"
    );
}

#[test]
fn resource_schema_compatibility_stays_adapter_neutral() {
    let requirement =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/model/requirement.rs"))
            .expect("resource requirement source");

    for forbidden in [
        "sqlite", "fjall", "pingora", "deno", "process", "clock", "greeter",
    ] {
        assert!(
            !requirement.contains(forbidden),
            "shared resource compatibility grammar must not depend on concrete adapter {forbidden}"
        );
    }
}

#[test]
fn resource_identity_does_not_encode_schema_versions() {
    let resource =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/model/resource.rs"))
            .expect("resource id source");
    let requirement =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/model/requirement.rs"))
            .expect("resource requirement source");

    for source in [resource, requirement] {
        assert!(
            !source.contains(".v1")
                && !source.contains(".v2")
                && !source.contains("@1")
                && !source.contains("@2"),
            "resource identity grammar must not encode schema versions into resource ids"
        );
    }
}

#[test]
fn core_stays_free_of_concrete_candidate_resource_knowledge() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root");
    let module_runtime = fs::read_to_string(repo_root.join("core/src/module_runtime.rs"))
        .expect("module runtime source");
    let composition = fs::read_to_string(repo_root.join("core/src/composition/mod.rs"))
        .expect("composition source");

    for source in [module_runtime, composition] {
        for forbidden in [
            "database",
            "worker",
            "gateway",
            "namespace",
            "publication",
            "sqlite",
            "pingora",
        ] {
            assert!(
                !source.contains(forbidden),
                "core compatibility grammar must stay candidate-primitive neutral and found {forbidden}"
            );
        }
    }
}

#[test]
fn resource_api_stays_free_of_host_compatibility_types() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("manifest");
    assert!(
        !manifest.contains("fabric-host"),
        "fabric resource API must not depend on fabric-host"
    );

    for file in [
        "src/lib.rs",
        "src/model/mod.rs",
        "src/model/requirement.rs",
        "src/model/resource.rs",
        "src/model/schema.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(file)).expect("source");
        for forbidden in ["HostDescriptor", "HostRequirement", "HostFacilityId"] {
            assert!(
                !source.contains(forbidden),
                "{file} must not know host compatibility type {forbidden}"
            );
        }
    }
}
