# Releasing

Redactix is a Rust-backed Python package. Public releases should publish prebuilt wheels so users do not need a local Rust toolchain for normal installation.

## Package Name

The normalized PyPI name `redactix` is already occupied by an existing `Redactix` project. Before publishing to PyPI, either:

- gain ownership or transfer of the existing PyPI project, or
- choose a different distribution name and update `pyproject.toml`, `Cargo.toml`, README install commands, and Trusted Publisher configuration.

Do not publish GitHub release artifacts or documentation that promises `pip install redactix` until the package name is actually controlled.

## Local Checks

Run these before publishing:

```bash
cargo fmt --check
cargo check
uv run maturin develop
uv run pytest
uv run maturin build --release --sdist
```

## Trusted Publishing Setup

The release workflow uses PyPI Trusted Publishing through GitHub OIDC. It does not use `PYPI_TOKEN`, `MATURIN_PYPI_TOKEN`, or any saved PyPI API token.

Configure these before running a publish job:

- On GitHub, create environments named `testpypi` and `pypi`.
- On TestPyPI, add a trusted publisher for owner `RikidWai`, repository `Redactix`, workflow `release.yml`, environment `testpypi`.
- On PyPI, add a trusted publisher for owner `RikidWai`, repository `Redactix`, workflow `release.yml`, environment `pypi`.
- Add required reviewers to the `pypi` GitHub environment so production uploads need manual approval.

## Release Flow

1. Run the `Build and Publish` workflow with `publish=none` to build wheels and the source distribution only.
2. Run the same workflow with `publish=testpypi`.
3. Install from TestPyPI in a clean environment and smoke-test imports and basic redaction.
4. If TestPyPI is correct, run the workflow with `publish=pypi`.

Example TestPyPI smoke test:

```bash
python -m venv /tmp/redactix-test
source /tmp/redactix-test/bin/activate
python -m pip install --upgrade pip
python -m pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ <distribution-name>
python -c "import redactix; print(redactix.redact('Email alex@example.com'))"
```
