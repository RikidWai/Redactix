# Redactix

Rust-based Python library built with `uv`, `maturin`, and PyO3.

## Requirements

- Python managed by `uv`
- Rust toolchain with `cargo` and `rustc`

Check the tools:

```bash
uv --version
cargo --version
rustc --version
```

## Setup

Create or sync the project environment:

```bash
uv sync
```

Activate the virtual environment:

```bash
source .venv/bin/activate
```

On Windows PowerShell:

```powershell
.venv\Scripts\Activate.ps1
```

## Run

Import the package through `uv`:

```bash
uv run python -c "import redactix; print(redactix)"
```

Or, after activating the environment:

```bash
python -c "import redactix; print(redactix)"
```

## Develop With Maturin

Build and install the Rust extension into the active virtual environment:

```bash
uv run maturin develop
```

For a release-style local install:

```bash
uv run maturin develop --release
```

## Build

Build Python package artifacts:

```bash
uv build
```

Or build directly with maturin:

```bash
uv run maturin build --release
```

Artifacts are written to `dist/` or `target/wheels/` depending on the command.

## Publish

Publish with maturin:

```bash
uv run maturin publish
```

Publish to TestPyPI first:

```bash
uv run maturin publish --repository testpypi
```

If you prefer using a token explicitly:

```bash
MATURIN_PYPI_TOKEN=pypi-... uv run maturin publish
```

For TestPyPI:

```bash
MATURIN_PYPI_TOKEN=pypi-... uv run maturin publish --repository testpypi
```
