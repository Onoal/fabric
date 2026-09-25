# Open Questions

These questions are intentionally unresolved. Do not answer them without
evidence.

## Core

- What is the minimum genuinely generic semantic grammar?
- Which current Core concepts are truly structural?
- Which are still contaminated by Fabric's first semantic world?
- Is Module still the right foundational unit?
- What does a non-Resource/non-Component semantic world need?

## Definitions

- What constitutes Definition identity?
- What is Definition revision vs semantic identity?
- How are Rust-native and Forge-provided definitions equivalent or different?
- Does Core know where definitions came from?
- What does compatibility between definition revisions mean?

## Composition

- Which realization constraints legitimately belong in Composition?
- Where does semantic resolution end?

## Profile

- Which intent belongs to Profile?
- What is policy versus simple config?
- Can Profiles compose or inherit?
- Is profile identity semantic or only operational?

## Materialization

- What exactly makes a Plan immutable?
- Does the Plan have stable identity or only occurrence-local identity?
- What is the relationship between Plan and InstanceGeneration?

## Adapters

- When is Adapter to Adapter dependency legitimate?
- How are realization requirements expressed?
- How are realization cycles rejected?
- How are Host-provided realizations represented?

## Instance

- Which operational facilities should be attachable dynamically?
- Which must be planned before start?
- What belongs to Instance vs InstanceGeneration?

## Forge

- What is the minimal definition-source seam Fabric v2 needs?
- Can Forge provide definitions without Fabric Core depending on Forge?
- What remains valid about Rust-native definitions?
- How are third-party definition sources evaluated without coupling Core to one
  repository or service?
