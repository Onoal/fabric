# Fabric

Fabric is a Rust framework for describing, composing, realizing, and operating
technical systems through explicit semantic boundaries.

It separates a system's semantic meaning from the concrete implementations
that realize it, and separates both from the live state of a running system.
That makes dependencies explicit, lets compatible implementations be
selected interchangeably across declarative Compositions, and gives first-party
and third-party extensions the same public authoring surface.

## Why Fabric?

Fabric helps a system describe behavior, the capabilities that behavior needs,
shared Systems, implementations that realize Resources or Systems, Host
compatibility, a declarative Composition, and one or more live Instances.

The central distinction is:

```text
Composition = what the system declares
Instance    = one live materialization of that declaration
```

A Composition can therefore be inspected, reused, and materialized more than
once without becoming runtime state itself.

```text
Composition -> materialize -> Instance generation
new declarative truth -> materialize -> fresh Instance generation
```

An `InstanceGeneration` is a process-local runtime incarnation, not a version
or a replacement record. Fabric does not infer migration, state transfer,
cutover, rollback, or which concurrent generation is authoritative. Read the
[Instance concept](docs/concepts/instance.md) for the temporal contract.

## How Fabric fits together

```text
                         Composition
                              |
          +-------------------+-------------------+
          |                   |                   |
      Component            Resource             System
          |                   |                   |
          |              realized by         realized by
          +-------------------+-------------------+
                              |
                           Adapter
                              |
                     compatible with
                              |
                             Host

                 Composition materializes an Instance
```

- **Component** expresses typed behavior and can require Resources and Systems.
- **Resource** is an occurrence-based technical capability.
- **System** is an instance-wide shared capability.
- **Adapter** realizes a Component, Resource, or System for a concrete
  environment.
- **Host** describes environmental compatibility for a realization.
- **Manifest** inspects the semantic declarations in a Composition.

## Install

```toml
[dependencies]
fabric = { package = "onoal-fabric", version = "0.1.1" }
```

```rust
use fabric::prelude::*;
```

## A first look

```rust
use fabric::prelude::*;

let built = Fabric::new("example")
    .expect("valid composition")
    .build()
    .expect("build");

let manifest = built.manifest();
let mut instance = built
    .materialize_named("example.local")
    .expect("materialize");

instance.start().expect("start");
// Operate declared Components here when the Composition contains them.
instance.stop();
```

The [Getting Started guide](docs/getting-started.md) builds on this with a
complete Component operation and explains each transition from declaration to
live runtime.

## Where to go next

- New to Fabric? Start with [Getting Started](docs/getting-started.md).
- Want the conceptual map? Read [Concepts](docs/concepts/README.md).
- Need the precise model? Read the [Architecture](docs/architecture.md).
- Building directly on Core? See the [Advanced Raw API](docs/advanced/raw-api.md).
- Contributing from source? See [CONTRIBUTING.md](CONTRIBUTING.md).

## Project status

The latest crates.io release is Fabric 0.1.1. The `main` branch has completed
the Fabric 0.3 Time / Lifecycle / Change milestone and is preparing
`onoal-fabric` 0.1.2 for publication. It is not yet published to crates.io.

## License

Fabric is licensed under the [MIT License](LICENSE).
