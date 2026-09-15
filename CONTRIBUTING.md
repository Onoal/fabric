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
and update public documentation when a public API changes. Keep the normal SDK
path distinct from advanced Core APIs, and avoid product-specific vocabulary in
generic Fabric machinery.
