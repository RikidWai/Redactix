# Redactix

Redactix is a lightweight Rust-backed Python library for detecting and redacting common PII in text.

## Features

- Detects email addresses, phone numbers, and Luhn-valid credit card numbers.
- Redacts with placeholders by default: `{{EMAIL}}`, `{{PHONE}}`, `{{CREDIT_CARD}}`.
- Supports mask redaction with one `*` per detected Python character.
- Provides a configurable `Redactor` for custom regex patterns, placeholder overrides, and default redaction mode.

## Usage

```python
import redactix

text = "Contact me at alex@example.com or +1 415-555-2671."

matches = redactix.detect(text)
redacted = redactix.redact(text)
masked = redactix.redact(text, mode="mask")
```

```python
redactor = redactix.Redactor(
    custom_patterns={"name": r"\bJane Doe\b"},
    placeholders={"name": "{{PERSON}}"},
)

redactor.redact("Jane Doe emailed alex@example.com")
# "{{PERSON}} emailed {{EMAIL}}"
```

## Development

Build and install the Rust extension into the local environment:

```bash
uv run maturin develop
```

Run tests:

```bash
pytest
```

Build a wheel:

```bash
uv run maturin build --release
```
