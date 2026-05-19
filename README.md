# Redactix

<p align="center">
  <img src="assets/redactix-logo.png" alt="Redactix logo" width="320">
</p>

Redactix is a lightweight Rust-backed Python library for detecting and redacting common PII in text.

> Redactix is currently an alpha release. The detector set is intentionally small, and public APIs may change before a stable `1.0` release.

- Rust-backed speed for high-throughput text redaction.
- Simple Python API for detection, redaction, and reporting.
- Lightweight design focused on common structured PII and custom regex detectors.

## Default Detectors

| Name | Detection method | Validator |
| --- | --- | --- |
| `email` | Regex | `NULL` |
| `phone` | Regex | Boundary checks; 10-15 digits; rejects repeated single-digit numbers |
| `credit_card` | Regex | Luhn checksum; 13-19 digits |

## Comparison

Redactix is focused on lightweight, deterministic text redaction for common structured PII. It is intentionally smaller than full PII detection frameworks.

| Library | Difference |
| --- | --- |
| **Redactix** | Lightweight Rust-backed Python library with a simple API, built-in regex/validator detectors, custom regex detectors, and deterministic overlap resolution. |
| **Microsoft Presidio** | Much broader framework: NLP, regex, checksums, context-aware recognizers, multilingual support, text/images/structured data, Docker/PySpark/K8s options. More powerful but heavier. Source: [Presidio README](https://github.com/microsoft/presidio). |
| **scrubadub** | Python free-text scrubber with more built-in PII types: names, addresses, DOBs, URLs, credentials, SSNs, tax IDs, etc., plus optional/external detectors. Source: [scrubadub docs](https://scrubadub.readthedocs.io/en/stable/). |

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

## Roadmap

- Add more common PII detectors and localized PII patterns.
- Add optional AI-powered NER detection for advanced use cases while keeping the core library lightweight.

## Contributing

Development setup, test commands, and contribution guidance are documented in [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Redactix is distributed under the MIT License. See [LICENSE](LICENSE) for details.
