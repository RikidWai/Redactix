import pytest
import redactix
from redactix import Detection


def test_expected_usage():
    text = "Employee EMP123456 used alex@example.com and 4111 1111 1111 1111."
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

    assert redactor.redact(text) == (
        "Employee {{EMPLOYEE_ID}} used {{EMAIL}} and {{CREDIT_CARD}}."
    )
    assert redactor.detect(text) == [
        Detection(
            start=9,
            end=18,
            value="EMP123456",
            entity_type="EMPLOYEE_ID",
            detector_name="employee_id",
            replacement="{{EMPLOYEE_ID}}",
        ),
        Detection(
            start=24,
            end=40,
            value="alex@example.com",
            entity_type="EMAIL",
            detector_name="email",
            replacement="{{EMAIL}}",
        ),
        Detection(
            start=45,
            end=64,
            value="4111 1111 1111 1111",
            entity_type="CREDIT_CARD",
            detector_name="credit_card",
            replacement="{{CREDIT_CARD}}",
        ),
    ]
    assert redactor.redact_with_report(text).text == redactor.redact(text)


def test_redactor_defaults_to_common_builtins():
    redactor = redactix.Redactor()
    assert redactor.redact("Jane emailed alex@example.com") == (
        "Jane emailed {{EMAIL}}"
    )


def test_empty_detectors_disables_builtins():
    redactor = redactix.Redactor(detectors=[])
    assert redactor.redact("Email alex@example.com") == "Email alex@example.com"


def test_select_builtin_detectors():
    redactor = redactix.Redactor(detectors=["email"])
    text = "Email alex@example.com or call +1 415-555-2671. Card: 4111 1111 1111 1111."
    assert redactor.detect(text) == [
        Detection(
            start=6,
            end=22,
            value="alex@example.com",
            entity_type="EMAIL",
            detector_name="email",
            replacement="{{EMAIL}}",
        )
    ]


def test_custom_placeholder_override():
    redactor = redactix.Redactor(detectors=[])
    redactor.register_detector("name", r"\bJane Doe\b", placeholder="{{PERSON}}")
    assert redactor.redact("Jane Doe") == "{{PERSON}}"


def test_placeholder_format():
    redactor = redactix.Redactor(placeholder_format="[[{entity_type}]]")
    assert redactor.redact("Email alex@example.com") == "Email [[EMAIL]]"


def test_fixed_mask_configuration():
    redactor = redactix.Redactor(mask_strategy="fixed", fixed_mask="[redacted]")
    assert redactor.redact("Email alex@example.com") == "Email [redacted]"


def test_length_preserving_mask_char_configuration():
    redactor = redactix.Redactor(mask_strategy="length_preserving", mask_char="x")
    assert redactor.redact("Email alex@example.com") == "Email xxxxxxxxxxxxxxxx"


def test_disabled_custom_detector_is_registered_but_inactive():
    redactor = redactix.Redactor(detectors=[])
    redactor.register_detector("name", r"\bJane Doe\b", enabled=False)
    assert redactor.redact("Jane Doe") == "Jane Doe"


def test_custom_detector_default_placeholder_uses_uppercase_name():
    redactor = redactix.Redactor(detectors=[])
    redactor.register_detector("employee_id", r"EMP-123")
    assert redactor.redact("Employee EMP-123") == "Employee {{EMPLOYEE_ID}}"


def test_overlap_prefers_higher_priority():
    redactor = redactix.Redactor(detectors=[])
    redactor.register_detector("short_id", r"EMP123", priority=100)
    redactor.register_detector("employee_id", r"EMP123456", priority=200)
    assert redactor.redact("Employee EMP123456") == "Employee {{EMPLOYEE_ID}}"


def test_overlap_equal_priority_prefers_longer_match():
    redactor = redactix.Redactor(detectors=[])
    redactor.register_detector("short_id", r"EMP123", priority=100)
    redactor.register_detector("employee_id", r"EMP123456", priority=100)
    assert redactor.redact("Employee EMP123456") == "Employee {{EMPLOYEE_ID}}"


def test_invalid_custom_regex():
    redactor = redactix.Redactor(detectors=[])
    with pytest.raises(ValueError):
        redactor.register_detector("name", "[")


def test_empty_custom_detector_name():
    redactor = redactix.Redactor(detectors=[])
    with pytest.raises(ValueError):
        redactor.register_detector("", r"Jane")


def test_invalid_builtin_detector_name():
    with pytest.raises(ValueError):
        redactix.Redactor(detectors=["address"])


def test_duplicate_builtin_detector_name_in_constructor():
    with pytest.raises(ValueError):
        redactix.Redactor(detectors=["email", "email"])


def test_custom_detector_cannot_duplicate_builtin_name():
    redactor = redactix.Redactor(detectors=["email"])
    with pytest.raises(ValueError):
        redactor.register_detector("email", r"alex@example\.com")


def test_custom_detector_names_are_normalized():
    redactor = redactix.Redactor(detectors=[])
    redactor.register_detector("Employee_ID", r"EMP-123")
    assert redactor.detect("EMP-123")[0].detector_name == "employee_id"
