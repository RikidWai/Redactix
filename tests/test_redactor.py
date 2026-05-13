import pytest
import redactix


def test_custom_name_pattern():
    redactor = redactix.Redactor(
        custom_patterns={"name": r"\bJane Doe\b"},
        default_patterns=True,
    )
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


def test_redactor_defaults_to_custom_only():
    redactor = redactix.Redactor(custom_patterns={"name": r"\bJane Doe\b"})
    assert redactor.redact("Jane Doe emailed alex@example.com") == "{{NAME}} emailed alex@example.com"


def test_custom_name_pattern_mask_mode():
    redactor = redactix.Redactor(
        custom_patterns={"name": r"\bJane Doe\b"},
        default_patterns=True,
    )
    assert redactor.redact("Jane Doe emailed alex@example.com", mode="mask") == (
        "******** emailed ****************"
    )


def test_custom_name_detect_example():
    text = "Jane Doe can be contacted at alex@example.com."
    redactor = redactix.Redactor(
        custom_patterns={"name": r"\bJane Doe\b"},
        default_patterns=True,
    )
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
        default_patterns=True,
    )
    assert redactor.redact("Jane Doe can be contacted at alex@example.com.") == (
        "{{PERSON}} can be contacted at {{HIDDEN_EMAIL}}."
    )


def test_default_mode_mask():
    redactor = redactix.Redactor(
        custom_patterns={"name": r"\bJane Doe\b"},
        mode="mask",
        default_patterns=True,
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


def test_default_patterns_enables_all_builtins():
    redactor = redactix.Redactor(default_patterns=True)
    assert redactor.redact("Email alex@example.com") == "Email {{EMAIL}}"


def test_default_patterns_cannot_be_combined_with_patterns():
    with pytest.raises(ValueError):
        redactix.Redactor(patterns=["email"], default_patterns=True)


def test_duplicate_builtin_pattern_name_in_constructor():
    with pytest.raises(ValueError):
        redactix.Redactor(patterns=["email", "email"])


def test_custom_pattern_cannot_duplicate_active_builtin():
    with pytest.raises(ValueError):
        redactix.Redactor(
            patterns=["email"],
            custom_patterns={"email": r"alex@example\.com"},
        )


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


def test_add_pattern_adds_custom_pattern():
    redactor = redactix.Redactor()
    redactor.add_pattern("name", r"\bJane Doe\b")
    assert redactor.redact("Jane Doe emailed alex@example.com") == "{{NAME}} emailed alex@example.com"


def test_add_pattern_fails_if_name_exists():
    redactor = redactix.Redactor(custom_patterns={"name": r"\bJane Doe\b"})
    with pytest.raises(ValueError):
        redactor.add_pattern("name", r"\bJohn Doe\b")


def test_add_pattern_fails_if_builtin_name_exists():
    redactor = redactix.Redactor(patterns=["email"])
    with pytest.raises(ValueError):
        redactor.add_pattern("email", r"alex@example\.com")


def test_replace_pattern_overrides_custom_pattern():
    redactor = redactix.Redactor(custom_patterns={"name": r"\bJane Doe\b"})
    redactor.replace_pattern("name", r"\bJohn Doe\b")
    assert redactor.redact("Jane Doe and John Doe") == "Jane Doe and {{NAME}}"


def test_replace_pattern_overrides_builtin_pattern():
    redactor = redactix.Redactor(patterns=["email"])
    redactor.replace_pattern("email", r"alex@example\.com")
    assert redactor.redact("Email alex@example.com or bob@example.com") == (
        "Email {{EMAIL}} or bob@example.com"
    )


def test_replace_pattern_fails_if_name_does_not_exist():
    redactor = redactix.Redactor()
    with pytest.raises(ValueError):
        redactor.replace_pattern("name", r"\bJane Doe\b")


def test_remove_pattern_disables_builtin():
    redactor = redactix.Redactor(default_patterns=True)
    redactor.remove_pattern("email")
    assert redactor.redact("Email alex@example.com. Card: 4111 1111 1111 1111.") == (
        "Email alex@example.com. Card: {{CREDIT_CARD}}."
    )


def test_remove_pattern_disables_custom_pattern():
    redactor = redactix.Redactor(custom_patterns={"name": r"\bJane Doe\b"})
    redactor.remove_pattern("name")
    assert redactor.redact("Jane Doe") == "Jane Doe"


def test_remove_pattern_fails_if_name_does_not_exist():
    redactor = redactix.Redactor()
    with pytest.raises(ValueError):
        redactor.remove_pattern("name")
