use std::fs;

#[test]
fn resource_registry_source_stays_foundational_and_excludes_concrete_implementations() {
    let root = format!("{}/src", env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![std::path::PathBuf::from(&root)];
    while let Some(path) = files.pop() {
        let metadata = fs::metadata(&path).expect("resource registry metadata");
        if metadata.is_dir() {
            for entry in fs::read_dir(&path).expect("resource registry directory") {
                files.push(entry.expect("resource registry entry").path());
            }
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path).expect("resource registry source");
        let display_path = path
            .strip_prefix(env!("CARGO_MANIFEST_DIR"))
            .expect("resource registry relative path")
            .display()
            .to_string();
        if display_path.ends_with("source_guards.rs") {
            continue;
        }
        for forbidden in [
            "DatabaseConfig",
            "FjallKvConfig",
            "DenoWorkerConfig",
            "SystemdProcessConfig",
            "SQLite",
            "Postgres",
            "Fjall",
            "Deno",
            "Systemd",
            "Pingora",
            "Apps",
            "Authentication",
            "EngineResourceId",
            "EngineResourceDescriptor",
            "EngineResourceInterface",
            "EngineResourceFacetId",
            "ResourceKind",
            "SecretRef",
            "DatabaseRef",
            "KvRef",
            "ServiceId",
            "HomeId",
            "Platform",
            "WorkloadBinding",
            "BindingTarget",
            "ResolvedResourceBinding",
            "PreparedResource",
            "Clock",
            "Greeter",
            "third-party.test-clock",
        ] {
            assert!(
                !source.contains(forbidden),
                "{display_path} leaked concrete resource or future rail detail: {forbidden}"
            );
        }
    }
}

#[test]
fn shared_configuration_facet_does_not_offer_a_noop_constructor() {
    let path = format!("{}/src/model/configuration.rs", env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(path).expect("configuration facet source");
    assert!(
        !source.contains("pub fn typed<"),
        "shared configuration facet must require a concrete resource-owned consumer"
    );
    assert!(
        !source.contains("Self::typed_with(kind, |_value: &T| Ok(()))"),
        "shared configuration facet must not reintroduce implicit successful no-op consumption"
    );
}

#[test]
fn registry_resource_id_key_is_local_and_not_canonical_occurrence_identity() {
    let lib = fs::read_to_string(format!("{}/src/lib.rs", env!("CARGO_MANIFEST_DIR")))
        .expect("read resource registry lib");
    assert!(
        lib.contains("Experimental, extraction-era Resource Registry machinery")
            && lib.contains("ResourceId + ResourceName"),
        "registry documentation must state that its ResourceId key is local, not canonical occurrence identity"
    );
}
