# Fabric v1 foundation readiness

This document freezes the current Fabric v1 foundation architecture for the
technical `1.0.0` release task. It is a release-readiness anchor, not the
release itself: source crates remain `0.7.0` until the release task performs
the coordinated version bump, tag, publication, and GitHub release.

## Foundation model

Fabric v1 is the construction and live-materialization foundation:

```text
Definitions and authoring
    -> Fabric
    -> Composition
    + MaterializationProfile
    + Host
    -> MaterializationPlan
    -> Instance
    -> semantic live world
    + InstanceFacility
```

- **Core** is generic construction, compatibility, provider resolution, and
  runtime materialization machinery.
- **Fabric** is the first-party semantic world built from Resource, System,
  Component, Adapter, Relations, Realizations, and Augmentations.
- **Composition** is immutable declared semantic truth.
- **MaterializationProfile** is occurrence intent and provenance.
- **Host** is environmental compatibility truth.
- **MaterializationPlan** is frozen, non-live effective materialization truth.
- **Instance** is one generation-scoped live materialization.
- **InstanceFacility** is optional live operational tooling attached to one
  Instance generation.

## Frozen v1 distinctions

The foundation release depends on these separations:

```text
Definition != Contribution != Composition != MaterializationPlan != Instance
Resource != System != Component != Adapter
Config != runtime State
Lifecycle != Health
Component != ComponentParticipation
semantic Augmentation != InstanceFacility
declared truth != planned truth != live semantic truth != live operational truth
```

Reusable contributions organize authoring only. They have no runtime,
identity, lifecycle, semantic occurrence, or inspection presence.

Facilities organize live operational behavior only. They are not semantic
participants, do not alter Composition or Plan truth, do not define health,
and do not intercept Resource, System, Component, or Adapter operations.

## Deliberately outside v1

The following remain outside the technical Fabric v1 foundation:

- a general `MaterializationPlanner`;
- Profile-driven realization policy;
- a generic Forge/definition-source seam;
- a split generic semantic-resolution and realization-resolution model;
- distributed occurrence truth;
- durable event/history records;
- plan migration, replanning, cutover, rollback, or replacement authority;
- Host offers, capacity planning, scheduling, orchestration, and deployment;
- package marketplace, publication flow, or wider ecosystem launch semantics.

Those topics are reserved for future architecture work. They are not required
for the v1 foundation to build, inspect, materialize, observe, and operate one
complete Fabric semantic system locally.

## Release handoff

The next release task should perform only mechanical release actions if the
audit remains green:

1. bump all seven public crates from `0.7.0` to `1.0.0`;
2. update all workspace internal dependency versions to `1.0.0`;
3. regenerate `Cargo.lock`;
4. run the full validation and packaging matrix;
5. publish crates in dependency order;
6. create the `v1.0.0` tag on the release commit;
7. create the GitHub release;
8. update ecosystem repositories from git/SHA dependencies to released
   `1.0.0` crates.

No ecosystem package breadth, examples repository, or marketing launch is
required for this technical foundation release.
