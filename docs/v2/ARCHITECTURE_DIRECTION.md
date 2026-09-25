# Fabric V2 Architecture Direction

This document records current future direction. It is not a v1 migration guide
and does not redefine current Fabric main.

The central direction is:

```text
generic semantic construction machinery
            ↓
Fabric semantic world
            ↓
Composition / materialization / Instance
```

V2 should make these boundaries substantially cleaner.

## Core Direction

Fabric Core != Fabric semantic vocabulary.

Core should become clean construction machinery with which semantic worlds can
be built. It must not intrinsically require every world to use:

- Resource
- System
- Component
- Adapter
- Host

Those are Fabric's first-party semantic world above Core.

The intended architecture is approximately:

```text
FABRIC CORE
    semantic definition machinery
    identity machinery
    relation machinery
    contracts / requirements
    composition machinery
    resolution machinery
    occurrence/materialization machinery
    lifecycle primitives where structurally general
    inspection machinery where structurally general

            ↓

FABRIC
    Resource
    System
    Component
    Adapter
    Host
    Composition
    Instance
    Fabric-specific authoring and runtime semantics
```

The precise Core vocabulary is not established yet. V2 should generalize
machinery, not Fabric vocabulary.

## Non-Fabric Semantic Worlds

A third party or another Onoal system should eventually be able to use the
construction machinery without pretending its world contains Resources,
Components, Systems, or Adapters.

Another semantic world may define different kinds of definitions, relations,
occurrences, realizations, lifecycle subjects, and runtime subjects. Core
should support that structurally without Fabric vocabulary leaking into the
world.

This does not claim that Origin, Oracle, Forge, or any other system must depend
on Fabric Core. Dependency must still be earned. The v2 goal is that Core is
sufficiently pure that reuse becomes structurally possible.

## Fabric Above Core

Fabric Core says how semantic worlds may be constructed.

Fabric is one semantic world built using that machinery.

Fabric's own world may continue to define Resource, System, Component, Adapter,
Host, Composition, and Instance. These are valid Fabric concepts. They simply
cease to define what every Core consumer must look like.

This prevents the opposite mistake: making Fabric itself so generic that its
own semantics disappear.

## Definition Direction

Reserve a v2 architectural question around definitions:

```text
definition
!= occurrence
!= realization
!= binding
!= runtime state
```

Definitions should become cleanly addressable and versionable semantic inputs
rather than being accidentally equivalent to Rust type declarations forever.

Forge is a future independent definition world or definition source that may
eventually provide, store, version, resolve, or distribute definitions. Examples
outside Fabric may include identity schemas, contracts, tickets, assets,
protocols, semantic definitions, and Fabric definitions.

Do not establish Fabric Core as depending on Forge, and do not make Forge part
of Fabric. The reserved v2 direction is:

- Core should have a clean definition seam.
- Forge may later provide definitions through that seam.
- Rust-native definitions remain valid.
- Definition source != semantic construction machinery.

Conceptually:

```text
Forge / Rust / other definition source
             ↓
        Definitions
             ↓
      semantic world
```

The exact Definition abstraction is deferred to v2 evidence.

## Materialization Direction

Record the separation:

```text
Composition
!= MaterializationProfile
!= MaterializationPlan
!= Instance
```

Conceptual model:

```text
DEFINITIONS
    ↓
COMPOSITION
    reusable semantic assembly truth
    │
    ├──────── Host
    │           environmental facts
    │
    ├──────── MaterializationProfile
    │           occurrence-specific intent / policy
    │
    └──────── available realizations
                ↓
         MATERIALIZATION PLANNER
                ↓
         MATERIALIZATION PLAN
                ↓
             INSTANCE
```

The full planner is v2, not current v1 implementation.

## Composition Law

Candidate/validated direction:

Composition identity follows semantic assembly, not one concrete realization
choice.

Composition should increasingly represent semantic occurrences, semantic
relations, requirements, semantic constraints, and realization constraints where
those constraints genuinely belong to system definition.

Composition should not inherently mean one Host, one Profile, one final Adapter
graph, or one runtime occurrence. One Composition may produce multiple valid
Instances.

## Profile Law

MaterializationProfile = occurrence-specific materialization intent / policy.

Potential future concerns include preferred realizations, operational mode,
diagnostics, durability intent, performance intent, local vs remote preference,
realization preferences, and instance facilities.

Do not freeze a giant Profile schema now.

Profile policy != Host truth.

Profile may alter how a Composition is realized without rewriting what the
Composition semantically is.

## Host Law

Host = environmental truth / possibilities.

Examples include operating system, architecture, local capabilities, attached
hardware, filesystem, network, and compute.

Host should not become hidden policy.

Hard distinction:

```text
Host facts
!= realization preference
!= planning policy
```

## Semantic Resolution vs Realization Planning

Reserve this major v2 split:

```text
SemanticResolution
!= RealizationResolution
```

Semantic resolution may answer which semantic occurrence satisfies which
semantic relation and what the Composition means.

Realization planning may answer how a particular occurrence will actually be
realized and which realization is selected for this Profile + Host.

Conceptually:

```text
Composition
    ↓
SemanticResolution
    frozen semantic graph

SemanticResolution
+ Profile
+ Host
+ realization possibilities
    ↓
MaterializationPlan
```

## Adapter Direction

Record the deeper realization model:

```text
AdapterDefinition
!= realization preference
!= AdapterBinding
!= AdapterRuntime
```

Reserve support for real realization graphs where evidence requires:

```text
Semantic Resource
        ↓
Adapter A
        ↓
Adapter B
        ↓
Host capability
```

If A semantically requires B, model a semantic relation. Use Adapter to Adapter
dependency only when the dependency is genuinely a realization dependency.

Do not move semantic dependencies into realization machinery.

## Resource / System / Component Continuity

Do not remove these concepts from Fabric. Clarify their multi-stage truth:

```text
Resource definition
    ↓
Composition occurrence
    ↓
effective realization/binding
    ↓
live Resource state

System definition
    ↓
Composition System truth
    ↓
effective realization
    ↓
instance-wide live System

Component definition
    ↓
Composition Component truth
    ↓
participation intent
    ↓
ComponentParticipation
```

This continues the existing law:

```text
definition
!= occurrence
!= live state
```

## Instance Direction

Instance = live operational envelope of one MaterializationPlan.

An Instance may contain live Resource state, live Systems,
ComponentParticipations, Adapter runtimes, lifecycle, health, control,
observations, and operational facilities.

Instance is not merely Composition.start().

Instance != Composition.

## Instance Facility Direction

Reserve an explicit future seam for instance-scoped operational machinery that
does not redefine Composition semantic truth.

Working terminology: Instance Facility.

Do not freeze the name if evidence later finds a better one.

Potential examples include diagnostic logging, tracing, profiling, runtime
inspection, and operator instrumentation.

Hard distinction:

```text
semantic logging capability
!= instance operational logging facility
```

Do not build the facility framework here. This branch records the direction.

## Origin and Oracle Lessons

Use structural lessons without importing vocabulary.

Origin-like lesson:

```text
definition
!= occurrence
!= transition/history
!= evidence
```

Oracle-like lesson:

```text
need/definition
!= possibility
!= plan
!= binding
!= execution
!= observation
```

Fabric v2 applies the same structural discipline to its own world.

Do not import Oracle nouns such as Offer, Allocation, or Placement unless
Fabric independently earns them. Do not import Origin identity vocabulary.

## V1 Preservation Law

V2 should preserve the strongest lessons from Fabric 0.4 through 0.7 unless
stronger evidence contradicts them:

- relation-first construction
- definition != occurrence != live state
- Resource semantics
- System instance-wide semantics
- Component != ComponentParticipation
- Adapter realization separation
- Config != runtime State
- Composition != Instance
- generation-scoped runtime state
- typed invocation
- declarative Composition inspection
- live Instance observation
- contributions disappear into ordinary semantic truth
- Host is distinct from semantic capability
- generalize machinery, not vocabulary

V2 should purify boundaries around these laws, not restart Fabric from zero.
