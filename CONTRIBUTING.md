# Contributing to Fabric

## Development

Use a Rust toolchain compatible with the workspace's current edition. Clone
the repository and run the workspace checks locally before opening a pull
request.

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

Useful additional checks are:

```bash
cargo test --workspace --no-run --locked
cargo metadata --format-version 1 --locked
git diff --check
```

## Contributions

Keep semantic boundaries explicit, add or update tests for behavior changes,
and keep the normal SDK path distinct from advanced Core APIs. Avoid
product-specific vocabulary in generic Fabric machinery.

## Documentation is part of the change

When a canonical public concept changes, review the affected concept guide,
crate README, crate-level rustdoc, Getting Started/example, architecture guide,
and migration or release note. Do not update every page mechanically: update
the surfaces whose stated truth changed. Release notes are historical records,
not the sole explanation of the current architecture.

The documentation source-of-truth map and release documentation checklist live
in [docs/documentation.md](docs/documentation.md).
