# Fabric Raw API

Fabric's raw API is the advanced Core authoring surface beneath `fabric`.
It is public and supported for explicit integration work; it is not the normal
high-level authoring path.

Use named Fabric modules such as `fabric::core`, `fabric::component`, and
`fabric::authoring` when intentionally working below the high-level
frontend. Begin with [Getting Started](../getting-started.md) for normal
authoring, and use the [Architecture](../architecture.md) for the complete
semantic model.

## Raw flow

1. Implement `Module` and create `ModuleDeclaration` values.
2. Declare provided and required Contracts with compatibility metadata.
3. Add consumer-scoped `ContractProviderSelection` only when Composition must
   choose one of several compatible providers.
4. Optionally group Modules in `Block`s for composition-local reporting.
5. Build a `Composition`.
6. Materialize an `Instance` when all declarations have runtime materializers;
   use host-aware materialization when declarations require Host compatibility.
7. Start and stop the Instance according to its lifecycle.

`CompositionBuilder::build()` validates declaration graph truth but creates no
runtime. A declaration-only Module is valid; materializing a graph that lacks a
runtime materializer returns `MissingRuntimeMaterializer`.

## Raw laws

- `Module` is the smallest Core structural participant. It is not a normal
  high-level semantic world.
- `ModuleRuntime` is the live boundary for exports, bindings, Instance context,
  lifecycle, and health.
- `Block` is composition-local grouping/reporting machinery, not semantic
  topology or lifecycle ordering.
- Contract compatibility determines viability; provider selection determines
  which compatible provider a Composition chooses for one consumer.
- A raw `Composition` remains reusable declaration and materializes distinct,
  generation-scoped Instances.
- High-level Resource, System, and Component augmentation lowers to these
  ordinary Core Contracts, requirements, Modules, and provider selections.
  Core has no augmentation object or resolver; normal users should use the
  typed [Augmentation](../concepts/augmentation.md) API rather than hand-write
  its lowering.

Raw Core consumers can declare bounded `CompositionExport`s intentionally.
This does not make Instance a general service locator: only declared exports
are retained and retrievable for a specific materialization.

For normal semantic inspection use `FabricManifest`; raw Module declarations,
provider selections, Blocks, and Composition exports are exposed through
`manifest.diagnostics()`.
