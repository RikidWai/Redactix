import pytest
import redactix


def test_redact_builtin_pii_placeholder_mode():
    text = "Email alex@example.com"
    assert redactix.redact(text) == "Email {{EMAIL}}"


def test_redact_builtin_pii_mask_mode():
    text = "Email alex@example.com"
    assert redactix.redact(text, mode="mask") == "Email ****************"


def test_redact_builtin_examples():
    text = "Contact me at alex@example.com or +1 415-555-2671. Card: 4111 1111 1111 1111."
    assert redactix.redact(text) == "Contact me at {{EMAIL}} or {{PHONE}}. Card: {{CREDIT_CARD}}."
    assert (
        redactix.redact(text, mode="mask")
        == "Contact me at **************** or ***************. Card: *******************."
    )


def test_no_pii():
    assert redactix.detect("hello world") == []
    assert redactix.redact("hello world") == "hello world"


def test_invalid_mode_module_function():
    with pytest.raises(ValueError):
        redactix.redact("Email alex@example.com", mode="unknown")


def test_mask_length_uses_python_character_length():
    redactor = redactix.Redactor(custom_detectors={"name": "Jane 😄"}, default_detectors=True)
    assert redactor.redact("Jane 😄 emailed alex@example.com", mode="mask") == "****** emailed ****************"
