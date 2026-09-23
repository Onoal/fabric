# Fabric

Fabric is a Rust framework for declaring a system's semantic capabilities,
selecting concrete realizations for them, and materializing those declarations
as typed live Instances. It makes the boundary between *what a capability
means* and *how it runs here* explicit.

Fabric is not a scheduler, deployment engine, service registry, recovery
manager, transport protocol, or cloud-only model. It works equally for an
embedded capability, a local process, a database-backed service, or a shared
system capability.

## The model in one view

```text
Resource / System             Adapter
semantic identity, Config,    concrete implementation, Config,
Relations, API                state, lifecycle, health
          \                       /
           \  selected in a      /
            +-- Composition ----+
                    |
                    | materialize
                    v
             Instance generation
```

- **Resource**: an occurrence-based semantic capability.
- **System**: an instance-wide shared semantic capability.
- **Adapter**: a concrete realization of a Component, Resource, or System.
- **Component**: a semantic behavioral participant that consumes capabilities
  and may expose a typed API.
- **Host**: environment facts that determine whether a realization can run.
- **Composition**: reusable declarative truth and provider selection.
- **Instance**: one generation-scoped live materialization of a Composition.

The cross-cutting terms have deliberately different owners:

| Term | Question it answers |
| --- | --- |
| Config | What may a creator/consumer choose for this occurrence? |
| Relations | Which semantic capabilities are required? |
| API | What can semantic consumers call? |
| realization | Which concrete machinery provides that API? |
| state | What mutable live data belongs to this occurrence? |
| lifecycle | When is that live machinery initialized, started, and stopped? |
| health | How able is that live owner to fulfill its responsibility? |

`Config != state`, `API != implementation`, `Resource != Adapter`,
`Composition != Instance`, and `lifecycle != health`.

## Start here

```toml
[dependencies]
fabric = { package = "onoal-fabric", version = "0.5.4" }
```

```rust
use fabric::*;

fabric::resource! {
    Store {
        id: "example.store";
        api { fn get(&self, key: String) -> Option<String>; }
    }
}

fabric::adapter! {
    MemoryStore for Store {
        runtime {
            fn get(&self, _key: String) -> Option<String> { None }
        }
    }
}
```

`Store` states the semantic API once. `MemoryStore` owns the concrete
implementation. A Composition selects that Adapter; consumers bind and call
the `Store` API, not an Adapter-specific interface.

## Read next

1. [Getting Started](docs/getting-started.md) carries one capability from
   definition through realization, Composition, Instance, and typed use.
2. [The manual index](docs/README.md) gives the intended learning order.
3. [Architecture](docs/architecture.md) explains the stable ownership model.
4. [Migration to 0.5](docs/migrations/0.5.md) maps legacy 0.4.x ceremony to
   canonical authoring.

Normal code uses `fabric::*`. Named modules such as `fabric::core` and
`fabric::authoring` are explicit advanced surfaces; generated `raw` machinery
is not normal author vocabulary.

## Packages

The umbrella [`onoal-fabric`](sdk/README.md) crate is the normal dependency.
Focused crates (`onoal-fabric-core`, `-resource`, `-system`, `-component`,
`-host`, and `-sdk-macros`) document their advanced/direct-use boundaries in
their own package READMEs. Each package links back to this manual when viewed
on crates.io or docs.rs.

## License

Fabric is licensed under the [MIT License](LICENSE).
