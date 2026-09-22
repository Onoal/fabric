# Canonical executable examples

Fabric keeps its canonical examples as compiled workspace fixtures so the
manual does not depend on untested pseudo-Rust. They are small examples, not a
Cargo-topology prescription: a Resource, System, Adapter, or Component does
not imply a separate crate.

| Example | What it proves | Executable source |
| --- | --- | --- |
| Getting Started | semantic Resource, direct Adapter, Component consumption, Composition, Instance | [`tests/docs-getting-started`](../tests/docs-getting-started/src/lib.rs) |
| Resource | named Resource occurrences and realization selection | [`tests/docs-resource`](../tests/docs-resource/src/lib.rs) |
| System | shared System selection and typed consumer dependency | [`tests/docs-system`](../tests/docs-system/src/lib.rs) |
| Adapter | selection and Host compatibility | [`tests/docs-adapter`](../tests/docs-adapter/src/lib.rs) |
| Composition | independent Resource roles and declarative reuse | [`tests/docs-composition`](../tests/docs-composition/src/lib.rs) |
| Lifecycle and realization | API-only, self, direct Adapter, differential Adapter, state, health, cleanup | [`tests/sdk-api`](../tests/sdk-api/src/runtime_lifecycle_witness.rs) |
| Third-party target resolution | imported, aliased, re-exported Adapter targets | [`tests/macro-context-consumer`](../tests/macro-context-consumer/src/lib.rs) |

Run a fixture through Cargo, for example:

```bash
cargo test --manifest-path tests/docs-getting-started/Cargo.toml --locked
```

The examples use normal public authoring unless their entry explicitly says it
proves an advanced/Core boundary.
