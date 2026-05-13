import pytest
import redactix


def test_custom_name_pattern():
    redactor = redactix.Redactor(custom_patterns={"name": r"\bJane Doe\b"})
    assert redactor.redact("Jane Doe emailed alex@example.com") == "{{NAME}} emailed {{EMAIL}}"


def test_select_builtin_patterns():
    redactor = redactix.Redactor(patterns=["email"])
    text = "Email alex@example.com or call +1 415-555-2671. Card: 4111 1111 1111 1111."
    assert redactor.detect(text) == [
        {
            "type": "email",
            "start": 6,
            "end": 22,
            "text": "alex@example.com",
            "replacement": "{{EMAIL}}",
        }
    ]


def test_empty_builtin_patterns_allows_custom_only_redactor():
    redactor = redactix.Redactor(
        custom_patterns={"name": r"\bJane Doe\b"},
        patterns=[],
    )
    assert redactor.redact("Jane Doe emailed alex@example.com") == "{{NAME}} emailed alex@example.com"


def test_custom_name_pattern_mask_mode():
    redactor = redactix.Redactor(custom_patterns={"name": r"\bJane Doe\b"})
    assert redactor.redact("Jane Doe emailed alex@example.com", mode="mask") == "******** emailed ****************"


def test_custom_name_detect_example():
    text = "Jane Doe can be contacted at alex@example.com."
    redactor = redactix.Redactor(custom_patterns={"name": r"\bJane Doe\b"})
    assert redactor.detect(text) == [
        {
            "type": "name",
            "start": 0,
            "end": 8,
            "text": "Jane Doe",
            "replacement": "{{NAME}}",
        },
        {
            "type": "email",
            "start": 29,
            "end": 45,
            "text": "alex@example.com",
            "replacement": "{{EMAIL}}",
        },
    ]


def test_custom_placeholder():
    redactor = redactix.Redactor(
        custom_patterns={"name": r"\bJane Doe\b"},
        placeholders={"name": "{{PERSON}}"},
    )
    assert redactor.redact("Jane Doe") == "{{PERSON}}"


def test_custom_placeholder_can_override_builtin():
    redactor = redactix.Redactor(
        custom_patterns={"name": r"\bJane Doe\b"},
        placeholders={"name": "{{PERSON}}", "email": "{{HIDDEN_EMAIL}}"},
    )
    assert redactor.redact("Jane Doe can be contacted at alex@example.com.") == (
        "{{PERSON}} can be contacted at {{HIDDEN_EMAIL}}."
    )


def test_default_mode_mask():
    redactor = redactix.Redactor(
        custom_patterns={"name": r"\bJane Doe\b"},
        mode="mask",
    )
    assert redactor.redact("Jane Doe emailed alex@example.com") == "******** emailed ****************"


def test_override_default_mode_per_call():
    redactor = redactix.Redactor(
        custom_patterns={"name": r"\bJane Doe\b"},
        mode="mask",
    )
    assert redactor.redact("Jane Doe", mode="placeholder") == "{{NAME}}"


def test_invalid_custom_regex():
    with pytest.raises(ValueError):
        redactix.Redactor(custom_patterns={"name": "["})


def test_empty_custom_pattern_name():
    with pytest.raises(ValueError):
        redactix.Redactor(custom_patterns={"": r"Jane"})


def test_invalid_mode_redactor_constructor():
    with pytest.raises(ValueError):
        redactix.Redactor(mode="unknown")


def test_invalid_builtin_pattern_name():
    with pytest.raises(ValueError):
        redactix.Redactor(patterns=["address"])


def test_invalid_mode_redactor_method():
    redactor = redactix.Redactor()
    with pytest.raises(ValueError):
        redactor.redact("Email alex@example.com", mode="unknown")


def test_custom_pattern_default_placeholder_uses_uppercase_type():
    redactor = redactix.Redactor(custom_patterns={"employee_id": r"EMP-123"})
    assert redactor.redact("Employee EMP-123") == "Employee {{EMPLOYEE_ID}}"


def test_overlapping_matches_keep_first_safe_span():
    redactor = redactix.Redactor(custom_patterns={"wide": r"alex@example\.com extra"})
    assert redactor.redact("alex@example.com extra") == "{{WIDE}}"
