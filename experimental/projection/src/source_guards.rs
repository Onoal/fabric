use std::fs;

#[test]
fn shared_projection_source_stays_open_and_resource_neutral() {
    for file in [
        "/src/contract.rs",
        "/src/error.rs",
        "/src/handler.rs",
        "/src/model/lease.rs",
        "/src/model/materialized.rs",
    ] {
        let source =
            fs::read_to_string(format!("{}{}", env!("CARGO_MANIFEST_DIR"), file)).expect("source");
        for forbidden in [
            "DatabaseRef",
            "KvRef",
            "AuthorizedSecretRef",
            "SqliteDatabaseMaterialization",
            "KvAccess",
            "SecretMaterial",
            "SQLite",
            "Fjall",
            "Deno",
            "ActorRef",
        ] {
            assert!(
                !source.contains(forbidden),
                "{file} leaked resource-specific or runtime-specific term: {forbidden}"
            );
        }
    }
}
