# Fabric Documentation

Fabric documentation has three levels.

1. **Discover:** the repository [README](../README.md) explains what Fabric is
   and how to install it.
2. **Learn:** [Getting Started](getting-started.md) and the
   [Concepts overview](concepts/README.md) establish the normal model.
3. **Understand and extend:** [Architecture](architecture.md) defines the
   complete semantic model, while the [Advanced Raw API](advanced/raw-api.md)
   documents direct Core authoring.

## Learning order

1. [Getting Started](getting-started.md)
2. [Concepts](concepts/README.md)
   - [Composition](concepts/composition.md)
   - [Instance](concepts/instance.md)
   - [Component](concepts/component.md)
   - [Resource](concepts/resource.md)
   - [System](concepts/system.md)
   - [Adapter](concepts/adapter.md)
   - [Augmentation](concepts/augmentation.md)
   - Host
   - Manifest
3. [Architecture](architecture.md)
4. [Advanced Raw API](advanced/raw-api.md), when direct Core authoring is
   needed

Normal examples use `fabric::*`; `fabric::prelude::*` is an optional
compatibility convenience. Explicit experiments, when needed, live under
`fabric::experimental` and are not part of the normal learning path.

Release notes record their contract changes in [release notes](releases/0.4.1.md).

The concept sequence is intentional. Composition comes first because it
declares the assembled system; Instance then distinguishes that declaration
from its live materialization. Component introduces behavior, while Resource
and System explain two kinds of capability. Adapter explains realization, Host
explains environmental compatibility, and Manifest closes the normal path with
semantic Composition inspection.

Dedicated concept pages will expand this sequence. Until then, the Concepts
overview and Architecture provide the relevant model.
