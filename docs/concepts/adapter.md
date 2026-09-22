# Adapter

An **Adapter** is Fabric's explicit realization boundary between semantic
Component, Resource, or System contracts and one concrete implementation. It answers *how
are these semantics implemented here?* It does not redefine what the target
means or what a Component requires.

```text
Component/Resource/System semantics != Adapter realization
Adapter = one concrete realization of one Component, Resource, or System target
```

## Target, Config, and semantic API bridge

Every `AdapterDefinition` has one explicit `Target`: the Component, Resource,
or System definition it realizes. Its public responsibilities are `Target`,
`Compatibility`, `compatibility()`, `host_requirement()`, declaration
construction, and optional provider-runtime materialization. `adapter!` is the
normal ergonomic authoring form for Resource and System targets; handwritten
`AdapterDefinition` remains the public advanced path for Component targets.

```rust
fabric::adapter! {
    pub LocalStoreAdapter for Store {
        config { directory: String; }
        runtime {
            fn get(&self, key: String) -> Option<String> { let _ = key; None }
        }
    }
}
```

For a normal Resource or System target, its declared `api { ... }` is the
effective realization contract. The target alone therefore supplies the kind,
semantic API identity, and default exact schema support. The Adapter implements
that API directly; Fabric registers the Adapter-owned live provider behind the
typed semantic API. There is no Resource/System forwarding runtime in this
path, and consumers still call the semantic methods they declared. Target
expansion follows ordinary Rust type resolution: a local type, an imported
type, an alias, a re-export, and a fully qualified target are equivalent.

Target support is inferred exactly from `Store` by default. Use
`supports: "^0.4";` only for deliberate broader target support. A no-Config Adapter is constructed as
`Adapter::new()`; inline `config { ... }` generates a public Config type, while
`config: MyConfig;` uses a creator-owned Rust type directly.

An existing explicit realization boundary remains the advanced compatibility
form: `for resource Store implements StoreRealization` (or `for system ...`)
continues to lower through its declared realization contract. Use it when a
semantic owner deliberately mediates between its public API and a different
lower-level implementation interface. 0.4.7 does not add a new differential
realization language.

Adapter Config configures the implementation—for example, a directory or
endpoint—not Resource/System semantic Config, Component Config, or a Host
description. For an Adapter-realized Component, Fabric retains the configured
Component occurrence and supplies its Component Config to the typed realization
contract separately from Adapter Config. `AdapterDefinition::materialize_provider()` may return `None`, so
an Adapter declaration is not necessarily a native provider runtime; macro
Adapters normally supply one.

## Compatibility and selection

Fabric derives Resource versus System compatibility from the target in the
canonical form. Internally the existing schema support types still enforce
which semantic schema versions the implementation understands. `Provisional` matches only `Provisional`;
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

`.using(adapter)` validates target/schema compatibility and records declarative
realization truth. In the canonical path, the selected Adapter itself occupies
the semantic occurrence's provider slot; it does not add a fake target runtime
or per-method proxy. It does not connect, start a process, discover a service,
or change a live Instance.

Compatibility is not selection: compatibility means an Adapter *can* satisfy
the realization requirement; selection means this [Composition](composition.md)
chooses it. The legacy explicit form retains its provider selection machinery;
the canonical form directly exports the semantic API from the Adapter-owned
provider. Both keep semantic identity separate from concrete machinery.

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
Resource/System semantic API -> selection .using(Adapter)
-> Adapter-owned provider -> Composition -> materialize on Host -> semantic API handle
```

Provider runtimes participate in ordinary Core bind, initialize, start, stop,
and health phases. Adapter does not introduce a separate lifecycle, scheduler,
or control plane.

### Stateful realization runtime

Adapter Config is immutable materialization input. A realization that owns
live machinery may instead declare a `state` section: Fabric constructs that
state freshly for each provider runtime occurrence, while contract-service
clones for the same occurrence share it. This keeps runtime state out of
definition Config and prevents rematerialized generations from inheriting live
state accidentally.

```rust
fabric::adapter! {
    LocalStoreAdapter for Store {
        config { directory: String; }
        state { LocalStoreState = LocalStoreState::new(config.directory.clone()); }
        runtime { /* realization methods can use self.state.get() */ }
        lifecycle {
            initialize { Ok(()) }
            stop { self.state.get().close(); Ok(()) }
            health: Health::Healthy;
        }
    }
}
```

Every hook is optional. Defaults remain successful `initialize`, `start`, and
`stop`, with Healthy health. `RuntimeContext` is available after normal
instance-context binding, but cleanup must tolerate it being absent because
Core may abandon a materialized provider before binding completes. `stop` is
fallible and its cleanup failure is returned by `Instance::stop()`; it is not
a restart, recovery, or replacement policy.

The public SDK machinery behind this syntax is `RuntimeState`,
`RuntimeContext`, `StatefulRuntimeAuthoring`, and `StatefulAdapterDefinition`.
It is available for direct SDK authoring as well; these are runtime-construction
tools, not a new semantic Fabric participant kind.

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
| semantic API (normal) | target-derived | Adapter live typed implementation |
| System requirement | declared | bound typed contract |

Return to the [Concept map](README.md) or [Architecture](../architecture.md).
