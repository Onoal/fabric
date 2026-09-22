# Fabric Core

`fabric-core` is the Fabric Core crate. It owns the structural grammar
and the generic Instance runtime boundary used to materialize that grammar.

## Core vocabulary

- `ModuleDeclaration`: one structural participant's identity, contract, and host metadata
- `Module`: a declaration plus an optional runtime materializer
- `Contract`: the typed boundary between Modules
- `Composition`: one assembled declarative graph of Blocks with authoritative
  global contract resolution and lifecycle ordering
- `Instance`: one concrete materialization of a Composition
- `ModuleRuntime`: one materialized graph participant that exports, binds, and
  may own concrete live machinery
- derived provider: a provider which exports one typed Contract assembled from
  ordinary typed required Contracts
- `Block`: current structural grouping machinery inside a Composition
- `InstanceId`: stable semantic identity for that materialized runtime
- `InstanceGeneration`: fresh ephemeral runtime-generation fence for one
  materialization

The canonical Core law is:

- `Composition != Instance`
- `InstanceId != InstanceGeneration`
- `Module != ModuleRuntime`
- Module structural truth != materialized runtime truth
- Block != generic runtime owner

Composition owns reusable structure, resolution, and dependency order. A
`ModuleDeclaration` owns graph truth and can exist without any runtime
implementation. A `Module` may optionally materialize a `ModuleRuntime` when
an Instance is requested. Instance owns the generic runtime lifecycle, health
projection, startup cleanup, and reverse shutdown for the materialized graph.
`ModuleRuntime` owns live exports, resolved bindings, instance context, and
lifecycle hooks. Blocks remain visible as structural report groupings inside an
Instance, but they are no longer a peer runtime primitive.

```text
Composition
|- Fabric Block A
|  `- Fabric Module
|- Fabric Block B
|  `- Fabric Module
`- higher consumer / integration module

Composition
`- materialize(InstanceId)
   `- Instance { InstanceId, InstanceGeneration, runtime participants }
```

Composition validates globally before lifecycle side effects: a required
contract has zero providers (missing), one provider (bound), or multiple
providers (ambiguous). Contract dependencies form one cross-Block module graph.
Materialized Instance initialization and startup follow that graph; failures
stop initialized modules in reverse order. Instance, Block-report, and
Module-report health aggregate as Unavailable, then Degraded, then Healthy.

Composition validation reads declarations only. It never constructs a runtime
participant. `Composition::materialize` and `Composition::materialize_on`
create runtime participants later and fail explicitly when a declaration has no
runtime materializer; declarations are never silently omitted from an Instance.

## Contract compatibility

Core treats contract identity and compatibility as separate semantic axes:

- `ContractId`: stable typed contract name
- `ContractIdentity`: provisional or exact `ContractVersion`
- `ContractCompatibilityRequirement`: provisional or versioned requirement

Current donor Modules remain explicitly provisional. Core can also resolve
versioned synthetic/neutral contracts with semantic-version compatibility, but
it does not silently pick a highest version. Resolution stays authoritative:

1. same `ContractId`
2. compatibility requirement satisfied
3. typed value matches
4. exactly one compatible provider

If no provider exists, the contract is missing. If providers exist but none
satisfy compatibility, Core reports an incompatible-provider failure. If
multiple compatible providers exist, the result remains ambiguous.

## Derived providers

Core can also compose typed capability providers. A derived provider declares
one exported Contract and ordinary typed Contract requirements, receives those
requirements through normal `ModuleBindings`, and constructs its exported typed
value from them. Resolution still selects exactly one provider for each
Contract; Core does not select providers method by method.

A provider need not own independently live machinery. The generic
`DerivedContractProvider` participates in the existing graph for binding and
export but has no stateful lifecycle work of its own; a provider with live state
uses ordinary `ModuleRuntime` hooks. Composition still orders, binds, starts,
and cleans the complete provider graph uniformly.

## Non-ownership

Core does not own:

- Resource API semantics
- Component/product semantics
- transport protocols
- plugins or registries
- dynamic loading
- environment policy
- adapter-specific machinery

Core must remain free of upward dependencies on Resources, Components, or concrete
Composition crates.
