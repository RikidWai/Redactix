# Contributing

Thank you for considering a contribution to Redactix.

## Development

Build and install the local extension with maturin:

```bash
uv run maturin develop
```

Run tests:

```bash
uv run pytest
```

Run Rust checks:

```bash
cargo fmt --check
cargo check
```

Build a wheel:

```bash
uv run maturin build --release
```

## Pull Requests

Before opening a pull request, make sure the tests and Rust checks pass locally.

Please keep changes focused, include tests for behavior changes, and update the README when user-facing APIs or examples change.
