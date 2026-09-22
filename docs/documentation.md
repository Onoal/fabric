# Documentation maintenance

Fabric documentation is part of the public contract. A public change is not
complete until the affected documentation surfaces have been reviewed.

## Sources of truth

| Truth | Owner |
| --- | --- |
| Architecture-wide laws and ownership boundaries | [Architecture](architecture.md) |
| Semantic concept meaning | `docs/concepts/<concept>.md` |
| User onboarding | [Getting Started](getting-started.md) |
| Package role and direct-dependency guidance | each crate README and crate-level rustdoc |
| Advanced/Core machinery | `docs/advanced/` and the Core crate docs |
| Public transition | `docs/migrations/` |
| Historical release record | `docs/releases/` |

Release notes record what changed. They are not the only explanation of the
current architecture.

## Change checklist

When a canonical public concept changes, review the affected concept guide,
crate README, crate root rustdoc, onboarding/example, architecture guide, and
migration or release note. The list is intentionally scoped: an internal
refactor does not require editing every document.

Examples presented as executable are maintained in workspace test fixtures.
Conceptual snippets say so explicitly. Before a release, run the documentation
fixtures, rustdoc, link audit, and `cargo package` checks for every published
crate.

## Documentation inventory

The 0.5 documentation foundation classified the existing graph as follows.

| Surface | Classification | Current role |
| --- | --- | --- |
| Repository `README.md` | Canonical current | orientation and manual map |
| `docs/README.md` | Canonical current | manual index |
| `docs/getting-started.md` | Canonical current | first end-to-end path |
| `docs/architecture.md` | Canonical current | architecture-wide laws |
| `docs/concepts/README.md`, `resource.md`, `system.md`, `component.md`, `adapter.md`, `config.md`, `relations.md`, `api.md`, `realization.md`, `state.md`, `lifecycle.md`, `health.md`, `host.md`, `composition.md`, `instance.md`, `augmentation.md` | Canonical current | concept ownership and authoring |
| `docs/advanced/raw-api.md` | Advanced | intentionally explicit Core surface |
| `docs/migrations/0.5.md` | Migration | 0.4.x to canonical 0.5 path |
| `docs/releases/0.5.0.md` | Release / migration | the canonical 0.5 release boundary and user-facing transition summary |
| Earlier release notes | Historical | immutable release context, not current truth |
| `core/ARCHITECTURE.md`, `host/ARCHITECTURE.md` | Advanced | crate-specific architecture detail |
| `experimental/*/README.md` | Experimental | non-published research crates |
| seven published crate READMEs | Canonical current | package landing pages |
| `CONTRIBUTING.md` | Canonical current | contribution and maintenance rules |

No current document is removed or merged by this foundation. Earlier release
notes remain historical records; legacy syntax in tests is compatibility
coverage rather than canonical documentation.
