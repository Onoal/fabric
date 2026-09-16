use std::fs;
use std::path::Path;

#[test]
fn projection_stays_experimental_and_resource_neutral() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/experimental/projection");
    for file in [
        "contract.rs",
        "error.rs",
        "handler.rs",
        "model/lease.rs",
        "model/materialized.rs",
    ] {
        let source = fs::read_to_string(root.join(file)).expect("read source");
        for forbidden in ["Database", "SQLite", "Fjall", "Deno", "ResourceRegistry"] {
            assert!(!source.contains(forbidden), "{file} leaked {forbidden}");
        }
    }

    let lib = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"))
        .expect("read SDK root");
    let prelude = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/prelude.rs"))
        .expect("read prelude");
    assert!(lib.contains("pub mod experimental;"));
    assert!(!lib.contains("pub use experimental::projection"));
    assert!(!prelude.contains("Projector"));
}
