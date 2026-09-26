# Fabric manual

This is the current technical manual. Release notes preserve history; they are
not required to understand the current architecture.

## Start

1. [Getting Started](getting-started.md) — define an API, realize it, compose
   it, materialize an Instance, and consume a typed capability.
2. [Mental model and architecture](architecture.md) — the ownership boundaries
   that make the vocabulary coherent.

## Authoring semantic capabilities

- [Resource](concepts/resource.md)
- [System](concepts/system.md)
- [Component](concepts/component.md)
- [Adapter and realization](concepts/adapter.md)

## Cross-cutting concepts

- [Config](concepts/config.md)
- [Relations](concepts/relations.md)
- [API](concepts/api.md)
- [Realization](concepts/realization.md)
- [State](concepts/state.md)
- [Lifecycle](concepts/lifecycle.md)
- [Health](concepts/health.md)
- [Host](concepts/host.md)
- [Composition](concepts/composition.md)
- [Instance](concepts/instance.md)
- [Augmentation](concepts/augmentation.md)

## Advanced

- [Raw/Core API](advanced/raw-api.md)
- [Core architecture](../core/ARCHITECTURE.md)
- [Experimental crates](#experimental)

## Migration and history

- [0.4.x to 0.5 migration](migrations/0.5.md)
- [Canonical executable examples](examples.md)
- [Fabric v1 foundation readiness](releases/v1-foundation-readiness.md)
- [Fabric 0.6 release notes](releases/0.6.0.md)
- [Fabric 0.5 release notes](releases/0.5.0.md)

## Experimental

`experimental/binding` and `experimental/resource-registry` are repository
research crates. They are not part of the normal public authoring path or the
published package family.

## Repository topology

Fabric's repository folders communicate ownership; they are not the same thing
as Cargo package names, Rust module paths, or public API namespaces.

```text
core/          generic structural construction machinery
host/          environmental compatibility for materialization
participants/ Fabric's semantic participant crates
sdk/           the normal developer SDK and its implementation support
experimental/ maturity boundary for research crates
tests/         workspace-integrated regression fixtures
verification/ external/ecosystem verification workspaces
docs/          architecture and usage documentation
```

`participants/resource`, `participants/system`, and
`participants/component` remain the published packages `onoal-fabric-resource`,
`onoal-fabric-system`, and `onoal-fabric-component`. `sdk/macros` remains the
distinct proc-macro package `onoal-fabric-sdk-macros`; it lives under `sdk/`
because it supports the Rust SDK rather than defining a separate architectural
family.

Within those packages, source folders also communicate ownership without
defining public API namespaces. The SDK separates authoring input
(`sdk/src/authoring`), immutable built semantic truth (`sdk/src/composition`),
and live Instance projection (`sdk/src/instance`). The Component participant
crate separates declaration, participation, control, invocation, operation, and
runtime machinery under `participants/component/src`.

## Maintaining this manual

[Documentation maintenance](documentation.md) assigns source ownership and
defines the review rule for public changes.
