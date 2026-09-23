# Augmentation

An **augmentation** is externally owned semantic meaning attached to an
existing Fabric semantic subject. It lets a later author add a typed contract
without modifying, forking, inheriting from, or globally registering with the
base definition.

```text
base subject != semantic X != attachment != support implementation != realization
```

The base definition does not know its augmentations. An augmentation may know
its base; an independently authored support implementation may know both the
base and the augmentation. The semantic definition of `X` does not own one
particular support implementation.

```text
X ------augments------> selected Resource, System, or Component
Support --realizes X--> that attachment
```

This is typed declarative Composition truth. It is not inheritance, a feature
bag, free-form metadata, a plugin registry, or a new Core resolver.

## Attachment and support

An attachment says that `X` applies to a target. It does **not** claim that a
provider for `X::Contract` exists. A support implementation is optional,
independently owned implementation truth that provides that contract. A bare
attachment remains inspectable in the Manifest; a consumer that requires `X`
still needs a supported provider and otherwise fails through ordinary Core
provider resolution.

A support declaration may have its own required and optional contracts,
additional provided contracts, and Host requirement. Fabric preserves that
implementation truth while adding the target relationship and `X` provision.
Fabric owns the composition-local support-provider `ModuleId`, so its
declaration, runtime, and provider selection describe one occurrence.

Consumers require the typed semantic contract owned by `X`, never the support
implementation type. The shared Rust type for `X::Contract` belongs in the
semantic crate: support and consumers import it from there.

## Resource augmentation

Resource attachments are occurrence-specific. Their target is
`ResourceId + ResourceName`, so attaching `X` to `Store("primary")` does not
also attach it to `Store("secondary")`.

```rust
use fabric::*;

let store = Store::select("primary", StoreConfig {})?;
let attachment = ResourceAugmentation::<Store, Readback>::attach(&store, ())?;
let supported = attachment.using(ReadbackSupport);
let readback = supported.require_from(&store)?;

let composition = Fabric::new("example.resource-augmentation")?
    .resource(store)
    .resource_augmentation(supported)
    .build()?;
# let _ = (readback, composition);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`ResourceAugmentationDefinition`, `ResourceAugmentation`,
`ResourceAugmentationSupportDefinition`, `ResourceAugmentationRealization`,
and `ResourceAugmentationRequirement` retain this target relationship. The
requirement binds `X` to the selected Resource occurrence and its selected
support provider. See [Resource](resource.md) for the occurrence model.

## System augmentation

System augmentation follows the same ownership law but targets the selected
System occurrence identified by `SystemId`. Normal typed authoring has one
coherent System occurrence for an identity in a Composition; there is no
`SystemName`.

```rust
use fabric::*;

let clock = SharedClock::select(SharedClockConfig {})?;
let attachment = SystemAugmentation::<SharedClock, DriftObservation>::attach(&clock, ())?;
let supported = attachment.using(DriftSupport);
let drift = supported.require_from(&clock)?;

let composition = Fabric::new("example.system-augmentation")?
    .system(clock)
    .system_augmentation(supported)
    .build()?;
# let _ = (drift, composition);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`SystemAugmentation*` types preserve the selected System target and do not
turn a System into a named Resource. See [System](system.md).

## Component augmentation

Component augmentation has the same semantic law, but its runtime lowering is
Component-specific. `ComponentDefinition` continues to own base identity,
Config, and the base operation declaration. An augmentation cannot append an
operation to `ComponentDeclaration.operations`.

One configured Component has exactly one base runtime attachment and zero or
more augmentation preparation contributions. All those preparations use the
same generation-scoped Component participation; they do not create a second
base Component realization.

```rust
use fabric::*;

let audit = Gateway::define(GatewayConfig {})
    .augment::<Audit>(())?
    .using(AuditSupport);
let audit_requirement = audit.requirement();

let traced = audit.into_set()
    .augment::<Trace>(())?
    .using(TraceSupport);
let trace_requirement = traced.requirement();

let composition = Fabric::new("example.component-augmentation")?
    .component(traced)
    .build()?;
# let _ = (audit_requirement, trace_requirement, composition);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`ComponentAugmentationSet` carries one configured Component while additional
semantics are added. Each supported attachment retains its own typed
`ComponentAugmentationRequirement`; a bare set attachment uses
`without_support()` and has no provider-bound requirement. The same authoring
works when the base Component is self-realizing or uses its ordinary Adapter
realization. It does not add a primary Component contract or a
Component-to-Component dependency. See [Component](component.md).

The Component-specific public flow is represented by
`ComponentAugmentationDefinition`, `ComponentAugmentation`,
`ComponentAugmentationSupportDefinition`, `ComponentAugmentationRealization`,
and `ComponentAugmentationRequirement`; multi-augmentation authoring uses
`ComponentAugmentationSet`, `ComponentAugmentationSetAttachment`,
`ComponentAugmentationSetRealization`, and
`ComponentAugmentationSetAdapterRealization`.

## Multiple augmentations and inspection

Attachment identity is the target semantic occurrence plus the augmentation
Contract identity. Therefore one target may carry distinct `X` and `Y`, while
attaching the same `X` twice to that target is not a second meaningful semantic
occurrence and is rejected by Composition identity validation. There is no
global `AugmentationId`.

`FabricManifest` reports semantic attachments through the explicit
`ResourceAugmentationManifestEntry`, `SystemAugmentationManifestEntry`, and
`ComponentAugmentationManifestEntry` types. They identify the augmentation
contract and its semantic target:

| Target world | Semantic Manifest fields |
| --- | --- |
| Resource | Contract ID/identity, `ResourceId`, `ResourceName` |
| System | Contract ID/identity, `SystemId` |
| Component | Contract ID/identity, `ComponentId` |

They intentionally do not expose support implementation identity, provider
`ModuleId`, Host requirements, support dependencies, or runtime history.
`manifest.diagnostics()` remains the separate raw/Core diagnostic view of
module declarations and provider selections.

## What augmentation is not

Augmentation does not create an Adapter ontology: an Adapter remains
realization machinery, with no `AdapterAugmentation`, `AdapterId`, or Adapter
registry. Host remains environmental compatibility truth, not an augmentation
target. Core remains vocabulary-neutral and uses its existing `Module`,
`ModuleDeclaration`, Contract requirement, provider-selection, dependency
ordering, and materialization machinery.

For the complete ownership and inspection boundary, see
[Architecture](../architecture.md). The repository's
`verification/third-party-augmentation` workspace is the reproducible proof
that independent base, semantic, support, consumer, and composition crates can
use only the public surface.
