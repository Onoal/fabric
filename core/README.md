# onoal-fabric-core

`onoal-fabric-core` is Fabric's generic structural and materialization layer.
It validates contracts and requirements, selects/binds providers, and
materializes generation-scoped Instances with deterministic lifecycle cleanup.

Core is deliberately free of Resource, System, Component, Adapter, transport,
deployment, and product vocabulary. Normal users should depend on
[`onoal-fabric`](https://crates.io/crates/onoal-fabric), not Core directly.
Direct Core use is an advanced integration path.

Core can derive a typed provided capability from typed required capabilities;
this is general provider composition, not a method-level provider registry.
See [Core architecture](ARCHITECTURE.md) and the
[advanced manual](https://github.com/Onoal/fabric/blob/main/docs/advanced/raw-api.md).
