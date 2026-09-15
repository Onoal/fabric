use std::path::Path;

fn read(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

#[test]
fn shared_binding_rail_does_not_import_resource_specific_or_projection_terms() {
    let source = [
        read("src/lib.rs"),
        read("src/error.rs"),
        read("src/model/consumer.rs"),
        read("src/model/id.rs"),
        read("src/model/name.rs"),
        read("src/model/target.rs"),
    ]
    .join("\n");

    for forbidden in [
        "DatabaseRef",
        "KvRef",
        "AuthorizedSecretRef",
        "WorkloadId",
        "Deno",
        "SQLite",
        "Fjall",
        "Environment",
        "DATABASE_URL",
        "Apps",
        "Platform",
    ] {
        assert!(
            !source.contains(forbidden),
            "shared binding rail must not know `{forbidden}`"
        );
    }
}

#[test]
fn shared_binding_api_exports_only_canonical_binding_vocabulary() {
    let source = read("src/lib.rs");

    for forbidden in ["EngineBinding", "engine_binding", "ENGINE_BINDING"] {
        assert!(
            !source.contains(forbidden),
            "shared binding api must not export `{forbidden}`"
        );
    }

    for required in [
        "BindingConsumer",
        "BindingConsumerId",
        "BindingConsumerKind",
        "BindingId",
        "BindingName",
        "BindingTargetId",
        "BindingTargetKind",
    ] {
        assert!(
            source.contains(required),
            "shared binding api must export `{required}`"
        );
    }
}

#[test]
fn representative_consumers_use_canonical_binding_types() {
    let synthetic = read("src/tests.rs");

    assert!(
        !synthetic.contains("EngineBinding"),
        "binding consumers must not depend on EngineBinding vocabulary"
    );
    assert!(
        synthetic.contains("BindingName"),
        "binding consumers must use canonical BindingName"
    );
    assert!(
        synthetic.contains("BindingId"),
        "binding consumers must use canonical BindingId"
    );
}
