# Third-party augmentation equality witness

This independent Cargo workspace proves Fabric's public augmentation surface at
the pinned freeze-candidate revision `a4607d842a1d8fbf727667fa3a0333da4ac81c84`.
It is deliberately not a member of Fabric's workspace and obtains Fabric only
through its public Git dependency.

Dependency direction:

```text
base-semantics          -> Fabric
augmentation-semantics  -> Fabric + base-semantics
augmentation-support    -> Fabric + base-semantics + augmentation-semantics
consumers               -> Fabric + base-semantics + augmentation-semantics
app                     -> all witness crates
```
The base crate neither imports nor depends on augmentation crates; consumers do
not depend on support. The app proves Resource, System, Component X, and
Component X+Y contract binding at runtime.

Run from this directory:

```text
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo run -p third-party-augmentation-app --locked
cargo metadata --format-version 1 --locked
```
