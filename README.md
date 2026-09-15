# Fabric

Fabric is a Rust framework for describing, composing, realizing, and operating
technical systems through explicit semantic boundaries. It separates what a
system needs from how that need is implemented, while keeping declarations and
live runtime state distinct.

## Why Fabric?

Fabric helps applications and extension packages:

- express semantic capabilities independently from concrete implementations;
- make dependencies and provider choices explicit;
- compose systems declaratively;
- replace compatible realizations without changing Component behavior; and
- use the same public machinery for first-party and third-party extensions.

## Core model

```text
Fabric
├── Resource       an occurrence-based technical capability
├── System         an instance-wide shared capability
├── Adapter         a concrete Resource or System realization
├── Component       semantic behavior expressed as typed operations
├── Host            environmental compatibility information
├── Composition     a declarative assembly
└── Instance        one live materialization of a Composition
```

Resources, Systems, Adapters, and Components are declared in a Composition.
Materializing that Composition creates an Instance with its own lifecycle and
runtime state.

## Getting started

Add the normal Rust SDK package to your application:

```toml
[dependencies]
fabric-sdk = { package = "onoal-fabric", version = "0.1.0" }
```

Then import its stable Rust crate name:

```rust
use fabric_sdk::prelude::*;
```

To develop Fabric from source:

```bash
git clone https://github.com/Onoal/fabric.git
cd fabric
cargo test --workspace
```

## Documentation

- [Rust SDK guide](sdk/README.md)
- [Architecture](docs/architecture.md)
- [Advanced raw API](docs/raw-api.md)
- [Contributing](CONTRIBUTING.md)

## Repository structure

```text
core/              structural composition and runtime machinery
resource/          occurrence-based capabilities
system/            instance-wide capabilities
component/         semantic behavior and typed operations
host/              environmental compatibility
sdk/               high-level Rust authoring
sdk-macros/        SDK procedural macros
experimental/      non-standard experimental machinery
tests/             integration and extension tests
docs/              architecture and advanced API documentation
```

The modules under `experimental/` are retained research and experimental
machinery. They are outside the normal Fabric SDK and Fabric 0.1 public model.

## Status

Fabric is in early development. The current public API targets the 0.1 line.

## License

Fabric is licensed under the [MIT License](LICENSE).
