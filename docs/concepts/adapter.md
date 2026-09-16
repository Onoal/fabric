# Adapter

An **Adapter** is Fabric's explicit realization boundary between semantic
Component, Resource, or System contracts and one concrete implementation. It answers *how
are these semantics implemented here?* It does not redefine what the target
means or what a Component requires.

```text
Component/Resource/System semantics != Adapter realization
Adapter = one concrete realization of one Component, Resource, or System target
```

## Target, Config, and interface

Every `AdapterDefinition` has one explicit `Target`: the Component, Resource,
or System definition it realizes. Its public responsibilities are `Target`,
`Compatibility`, `compatibility()`, `host_requirement()`, declaration
construction, and optional provider-runtime materialization. `adapter!` is the
normal ergonomic authoring form for Resource and System targets; handwritten
`AdapterDefinition` remains the public path for Component targets.

```rust
fabric::adapter! {
    pub LocalStoreAdapter for resource Store implements StoreRealization {
        schema: provisional;
        realization: "1.0.0";
        config { directory: String; }
        runtime { /* realization methods */ }
    }
}
```

The target defines the typed `RealizationContract`; the Adapter implements it.
That realization interface differs from the consumer semantic contract:
Components consume the latter, while the target runtime uses the former to
realize its semantics.

Adapter Config configures the implementation—for example, a directory or
endpoint—not Resource/System semantic Config, Component Config, or a Host
description. For an Adapter-realized Component, Fabric retains the configured
Component occurrence and supplies its Component Config to the typed realization
contract separately from Adapter Config. `AdapterDefinition::materialize_provider()` may return `None`, so
an Adapter declaration is not necessarily a native provider runtime; macro
Adapters normally supply one.

## Compatibility and selection

Resource-targeting Adapters use `AdapterResourceSchemaSupport`; System targets
use `AdapterSystemSchemaSupport`. Their compatibility says which semantic schema versions
the implementation understands. `Provisional` matches only `Provisional`;
versioned support uses semantic-version requirements. Target identity must
also match: compatible-looking methods cannot apply an Adapter for one target
to another. Component-targeting Adapters use the Component's typed realization
boundary and target identity; Components do not gain Resource/System schema
machinery for this purpose.

```rust
let realized = MyResource::select("primary", resource_config)?
    .using(MyAdapter::new(adapter_config))?;
```

```text
target selection + compatible Adapter = declarative realization
```

`.using(adapter)` validates target/schema compatibility, creates a provider
selection, and records declaration truth. It does not connect, start a process,
discover a service, or change a live Instance.

Compatibility is not selection: compatibility means an Adapter *can* satisfy
the realization requirement; provider selection means this
[Composition](composition.md) chooses it. `ResourceRealization` retains its
`ResourceSelection`, `AdapterProviderModule`, and `ContractProviderSelection`.
`SystemRealization` retains the analogous System selection, provider carrier,
and selection. This keeps semantic identity separate from realization.

`AdapterProviderModule<A>` is Core-facing typed carrier machinery for the
Adapter declaration, Host requirement, and optional provider runtime. Its
derived `ModuleId` is not a universal Adapter identity. Fabric defines no
`AdapterId`, global Adapter namespace, `all_adapters()`, or Adapter registry.

Compatibility is also not replacement safety. It establishes that a selected
realization can bind or materialize in one Composition; it does not establish
safe live replacement, migration, state transfer, cutover, or rollback. A
realization change is represented by new declarative truth and a fresh Instance
generation, not by mutating a live Adapter provider.

## Host and dependencies

An Adapter may declare a `HostRequirement` for allowed operating systems,
architectures, or required facilities. `HostRequirement::new()` means no
additional restrictions in those current dimensions; it does not promise that
an implementation works everywhere forever.

Host requirements are declarative compatibility, not Host selection or
placement. At host-aware materialization, the supplied `HostDescriptor` is
checked before the provider is accepted.

```text
Target/schema compatibility: can Adapter A realize target T?
Host compatibility: can Adapter A run on Host H?
Provider selection: did this Composition choose Adapter A?
```

An Adapter provider may also require a System semantic contract. Core resolves
and binds that dependency before the Adapter runtime starts; it is not a global
lookup or a requirement for a concrete System Adapter.

```text
Component/Resource/System definition -> realization interface -> selection .using(Adapter)
-> provider selection -> Composition -> materialize on Host -> provider runtime
```

Provider runtimes participate in ordinary Core bind, initialize, start, stop,
and health phases. Adapter does not introduce a separate lifecycle, scheduler,
or control plane.

## Augmentation boundary

Adapter openness does not require Adapter augmentation. An Adapter is an
interchangeable realization relation, not a globally identified semantic
subject: Fabric defines no `AdapterAugmentation`, `AdapterId`, or Adapter
registry. An external semantic instead augments a selected Resource, System,
or Component target, and independently authored support can compose around
that target's chosen realization when appropriate. Semantic-target
augmentation is not Adapter-definition augmentation. See
[Augmentation](augmentation.md).

## Boundaries

Multiple Adapter types may realize one target, and the same Adapter type may
use different Config in different realizations. For example,
`Store("primary")` and `Store("analytics")` may have distinct realization
selections; they are Resource occurrences, not globally registered Adapters.
Likewise, separate Compositions can choose different Adapters for one System.

```text
Adapter != Component/Resource/System: semantics versus implementation
Adapter != Host: implementation versus environment compatibility
Adapter != external service or infrastructure account
Adapter != plugin, marketplace entry, dynamic ABI, or discovery registry
```

Do not put implementation-specific Adapter Config into semantic target Config.
Keeping those layers distinct lets Components require Resource/System semantics
without depending on a concrete realization.

| Concept | Authoring / Composition | Runtime |
| --- | --- | --- |
| Target | fixed by type | preserved |
| Adapter Config | implementation input | used by provider runtime |
| schema support | compatibility validation | not discovery |
| HostRequirement | declared | checked at materialization |
| provider selection | Composition truth | realized provider |
| realization contract | declared | live typed implementation |
| System requirement | declared | bound typed contract |

Return to the [Concept map](README.md) or [Architecture](../architecture.md).
