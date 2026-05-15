# Redactix

Redactix is a lightweight Rust-backed Python library for detecting and redacting common PII in text.

The default detector set covers:

- `email`
- `phone`
- `credit_card`

Credit card candidates are validated with Luhn checks. Phone candidates use regex matching plus basic digit sanity checks.

## Installation

Build and install the local extension with maturin:

```bash
uv run maturin develop
```

## Quick Start

```python
import redactix

redactor = redactix.Redactor(
    detectors=["email", "credit_card"],
    mask_strategy="placeholder",
)

redactor.register_detector(
    name="employee_id",
    pattern=r"\bEMP\d{6}\b",
    placeholder="{{EMPLOYEE_ID}}",
    enabled=True,
)

cleaned = redactor.redact(text)
detections = redactor.detect(text)
report = redactor.redact_with_report(text)
```

`redact()` returns only redacted text:

```python
redactix.redact("Email alex@example.com")
# "Email {{EMAIL}}"
```

`detect()` returns structured `Detection` dataclasses:

```python
[
    redactix.Detection(
        start=6,
        end=22,
        value="alex@example.com",
        entity_type="EMAIL",
        detector_name="email",
        replacement="{{EMAIL}}",
    )
]
```

`redact_with_report()` returns a `RedactionResult` dataclass with both the cleaned text and detections used for redaction.

## Redactor API

```python
redactix.Redactor(
    detectors: Optional[Sequence[str]] = None,
    mask_strategy: "placeholder" | "fixed" | "length_preserving" = "placeholder",
    placeholder_format: str = "{{{entity_type}}}",
    mask_char: str = "*",
    fixed_mask: str = "***",
)
```

`detectors=None` enables all built-in detectors. Pass a subset such as `["email"]` to enable only those detectors, or `[]` to start with no built-ins and register only custom detectors.

## Masking Strategies

- `placeholder` replaces matches with stable placeholders such as `{{EMAIL}}`.
- `fixed` replaces every match with `fixed_mask`, which defaults to `***`.
- `length_preserving` replaces each Python character in the match with `mask_char`.

Examples:

```python
redactix.redact("Email alex@example.com", mask_strategy="fixed")
# "Email ***"

redactix.redact("Email alex@example.com", mask_strategy="length_preserving")
# "Email ****************"
```

## Custom Detectors

Register custom regex-based detectors on a `Redactor`:

```python
redactor = redactix.Redactor(detectors=[])
redactor.register_detector(
    name="employee_id",
    pattern=r"\bEMP\d{6}\b",
    placeholder="{{EMPLOYEE_ID}}",
    enabled=True,
    priority=100,
)
```

Detector names are normalized to lowercase identifiers for `detector_name`; placeholders and entity types use uppercase forms such as `EMPLOYEE_ID`.

Overlapping matches are resolved deterministically:

- Higher `priority` wins.
- If priority is equal, the longer match wins.
- Remaining ties are resolved by source position and registration order.

## Contributing

Development setup, test commands, and contribution guidance are documented in [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Redactix is distributed under the MIT License. See [LICENSE](LICENSE) for details.
