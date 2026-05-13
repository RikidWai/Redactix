# Redactix

Redactix is a lightweight Rust-backed Python library for detecting and redacting common PII in text. The MVP focuses on a small, predictable detector set: email addresses, phone numbers, and Luhn-valid credit card numbers.

## Features

- Detects email addresses, phone numbers, and credit card numbers that pass Luhn validation.
- Redacts with double-curly placeholders by default: `{{EMAIL}}`, `{{PHONE}}`, `{{CREDIT_CARD}}`.
- Supports mask redaction with one `*` per detected Python character.
- Provides a configurable `Redactor` for choosing built-in detectors, adding custom regex patterns, overriding placeholders, and setting a default redaction mode.
- Returns Python character indexes in detection results, not UTF-8 byte offsets.

## Installation

Build and install the local extension with maturin:

```bash
uv run maturin develop
```

After installation:

```bash
uv run python -c "import redactix; print(redactix.redact('Email alex@example.com'))"
```

## Quick Start

```python
import redactix

text = "Contact me at alex@example.com or +1 415-555-2671. Card: 4111 1111 1111 1111."

matches = redactix.detect(text)
redacted = redactix.redact(text)
masked = redactix.redact(text, mode="mask")
```

`matches` contains dictionaries with the PII type, character span, original text, and replacement:

```python
[
    {
        "type": "email",
        "start": 14,
        "end": 30,
        "text": "alex@example.com",
        "replacement": "{{EMAIL}}",
    }
]
```

Placeholder redaction:

```python
redactix.redact("Email alex@example.com")
# "Email {{EMAIL}}"
```

Mask redaction:

```python
redactix.redact("Email alex@example.com", mode="mask")
# "Email ****************"
```

## Custom Redactors

Use `Redactor` when you need to choose which built-in PII types to detect, add custom text patterns, set custom placeholders, or change the default mode.

Supported built-in patterns are:

- `email`
- `phone`
- `credit_card`

Choose a subset of built-ins with `patterns`:

```python
redactor = redactix.Redactor(patterns=["email", "phone"])

redactor.redact("Email alex@example.com. Card: 4111 1111 1111 1111.")
# "Email {{EMAIL}}. Card: 4111 1111 1111 1111."
```

Use `patterns=[]` for a custom-only redactor:

```python
redactor = redactix.Redactor(
    patterns=[],
    custom_patterns={"name": r"\bJane Doe\b"},
)

redactor.redact("Jane Doe emailed alex@example.com")
# "{{NAME}} emailed alex@example.com"
```

```python
import redactix

redactor = redactix.Redactor(
    custom_patterns={"name": r"\bJane Doe\b"},
    placeholders={"name": "{{PERSON}}", "email": "{{HIDDEN_EMAIL}}"},
)

redactor.detect("Jane Doe emailed alex@example.com")
redactor.redact("Jane Doe emailed alex@example.com")
# "{{PERSON}} emailed {{HIDDEN_EMAIL}}"
```

Set mask mode as the default:

```python
redactor = redactix.Redactor(
    custom_patterns={"name": r"\bJane Doe\b"},
    mode="mask",
)

redactor.redact("Jane Doe emailed alex@example.com")
# "******** emailed ****************"
```

Override the mode per call:

```python
redactor.redact("Jane Doe emailed alex@example.com", mode="placeholder")
# "{{NAME}} emailed {{EMAIL}}"
```

## API

```python
redactix.detect(text: str) -> list[dict]
redactix.redact(text: str, mode: str = "placeholder") -> str
```

```python
redactix.Redactor(
    custom_patterns: dict[str, str] | None = None,
    placeholders: dict[str, str] | None = None,
    mode: str = "placeholder",
    patterns: list[str] | None = None,
)
```

`patterns=None` enables all built-in patterns. `patterns=[]` disables built-ins. Unsupported pattern names raise `ValueError`.

Supported redaction modes are `placeholder` and `mask`.

## Benchmark

Redactix includes a local benchmark script that compares Redactix redaction with Scrubadub redaction on the same repeated text payload.

Install Redactix first:

```bash
uv run maturin develop --release
```

Install Scrubadub only if you want the comparison row:

```bash
uv pip install scrubadub
```

Run the benchmark:

```bash
uv run python benchmarks/compare_scrubadub.py --iterations 100 --repetitions 100
```

The output reports mean latency, median latency, and approximate characters per second:

```text
Payload: 24,599 characters
Iterations: 100
Library   | Mean ms | Median ms | Chars/sec
--------- | ------- | --------- | ---------
Redactix  | ...
Scrubadub | ...
```

This benchmark measures runtime only. The libraries do not have identical detector sets, matching rules, or replacement formats, so use the numbers as a rough throughput comparison rather than a claim of identical behavior.

## Development

Run tests:

```bash
pytest
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
