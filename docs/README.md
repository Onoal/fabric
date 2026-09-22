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
- [Fabric 0.5 release notes](releases/0.5.0.md)

## Experimental

`experimental/binding` and `experimental/resource-registry` are repository
research crates. They are not part of the normal public authoring path or the
published package family.

## Maintaining this manual

[Documentation maintenance](documentation.md) assigns source ownership and
defines the review rule for public changes.
