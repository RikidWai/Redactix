import pytest
import redactix
from redactix import Detection, RedactionResult


def test_redact_builtin_pii_placeholder_strategy():
    text = "Email alex@example.com"
    assert redactix.redact(text) == "Email {{EMAIL}}"


def test_redact_builtin_pii_fixed_strategy():
    text = "Email alex@example.com"
    assert redactix.redact(text, mask_strategy="fixed") == "Email ***"


def test_redact_builtin_pii_length_preserving_strategy():
    text = "Email alex@example.com"
    assert redactix.redact(text, mask_strategy="length_preserving") == "Email ****************"


def test_redact_builtin_examples():
    text = "Contact me at alex@example.com or +1 415-555-2671. Card: 4111 1111 1111 1111."
    assert redactix.redact(text) == "Contact me at {{EMAIL}} or {{PHONE}}. Card: {{CREDIT_CARD}}."
    assert (
        redactix.redact(text, mask_strategy="length_preserving")
        == "Contact me at **************** or ***************. Card: *******************."
    )


def test_redact_with_report():
    result = redactix.redact_with_report("Email alex@example.com")
    assert result == RedactionResult(
        text="Email {{EMAIL}}",
        detections=[
            Detection(
                start=6,
                end=22,
                value="alex@example.com",
                entity_type="EMAIL",
                detector_name="email",
                replacement="{{EMAIL}}",
            )
        ],
    )


def test_no_pii():
    assert redactix.detect("hello world") == []
    assert redactix.redact("hello world") == "hello world"


def test_invalid_mask_strategy():
    with pytest.raises(ValueError):
        redactix.redact("Email alex@example.com", mask_strategy="unknown")


def test_invalid_mask_char():
    with pytest.raises(ValueError):
        redactix.Redactor(mask_strategy="length_preserving", mask_char="xx")


def test_mask_length_uses_python_character_length():
    redactor = redactix.Redactor(mask_strategy="length_preserving")
    redactor.register_detector("name", "Jane 😄")
    assert redactor.redact("Jane 😄 emailed alex@example.com") == (
        "****** emailed ****************"
    )
