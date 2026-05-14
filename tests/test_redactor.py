import pytest
import redactix


def test_custom_name_detector():
    redactor = redactix.Redactor(
        custom_detectors={"name": r"\bJane Doe\b"},
        default_detectors=True,
    )
    assert redactor.redact("Jane Doe emailed alex@example.com") == "{{NAME}} emailed {{EMAIL}}"


def test_select_builtin_detectors():
    redactor = redactix.Redactor(detectors=["email"])
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
    redactor = redactix.Redactor(custom_detectors={"name": r"\bJane Doe\b"})
    assert redactor.redact("Jane Doe emailed alex@example.com") == "{{NAME}} emailed alex@example.com"


def test_custom_name_detector_mask_mode():
    redactor = redactix.Redactor(
        custom_detectors={"name": r"\bJane Doe\b"},
        default_detectors=True,
    )
    assert redactor.redact("Jane Doe emailed alex@example.com", mode="mask") == (
        "******** emailed ****************"
    )


def test_custom_name_detect_example():
    text = "Jane Doe can be contacted at alex@example.com."
    redactor = redactix.Redactor(
        custom_detectors={"name": r"\bJane Doe\b"},
        default_detectors=True,
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
        custom_detectors={"name": r"\bJane Doe\b"},
        placeholders={"name": "{{PERSON}}"},
    )
    assert redactor.redact("Jane Doe") == "{{PERSON}}"


def test_custom_placeholder_can_override_builtin():
    redactor = redactix.Redactor(
        custom_detectors={"name": r"\bJane Doe\b"},
        placeholders={"name": "{{PERSON}}", "email": "{{HIDDEN_EMAIL}}"},
        default_detectors=True,
    )
    assert redactor.redact("Jane Doe can be contacted at alex@example.com.") == (
        "{{PERSON}} can be contacted at {{HIDDEN_EMAIL}}."
    )


def test_default_mode_mask():
    redactor = redactix.Redactor(
        custom_detectors={"name": r"\bJane Doe\b"},
        mode="mask",
        default_detectors=True,
    )
    assert redactor.redact("Jane Doe emailed alex@example.com") == "******** emailed ****************"


def test_override_default_mode_per_call():
    redactor = redactix.Redactor(
        custom_detectors={"name": r"\bJane Doe\b"},
        mode="mask",
    )
    assert redactor.redact("Jane Doe", mode="placeholder") == "{{NAME}}"


def test_invalid_custom_regex():
    with pytest.raises(ValueError):
        redactix.Redactor(custom_detectors={"name": "["})


def test_empty_custom_detector_name():
    with pytest.raises(ValueError):
        redactix.Redactor(custom_detectors={"": r"Jane"})


def test_invalid_mode_redactor_constructor():
    with pytest.raises(ValueError):
        redactix.Redactor(mode="unknown")


def test_invalid_builtin_detector_name():
    with pytest.raises(ValueError):
        redactix.Redactor(detectors=["address"])


def test_default_detectors_enables_all_builtins():
    redactor = redactix.Redactor(default_detectors=True)
    assert redactor.redact("Email alex@example.com") == "Email {{EMAIL}}"


def test_default_detectors_cannot_be_combined_with_detectors():
    with pytest.raises(ValueError):
        redactix.Redactor(detectors=["email"], default_detectors=True)


def test_duplicate_builtin_detector_name_in_constructor():
    with pytest.raises(ValueError):
        redactix.Redactor(detectors=["email", "email"])


def test_custom_detector_cannot_duplicate_active_builtin():
    with pytest.raises(ValueError):
        redactix.Redactor(
            detectors=["email"],
            custom_detectors={"email": r"alex@example\.com"},
        )


def test_invalid_mode_redactor_method():
    redactor = redactix.Redactor()
    with pytest.raises(ValueError):
        redactor.redact("Email alex@example.com", mode="unknown")


def test_custom_detector_default_placeholder_uses_uppercase_type():
    redactor = redactix.Redactor(custom_detectors={"employee_id": r"EMP-123"})
    assert redactor.redact("Employee EMP-123") == "Employee {{EMPLOYEE_ID}}"


def test_overlapping_matches_keep_first_safe_span():
    redactor = redactix.Redactor(custom_detectors={"wide": r"alex@example\.com extra"})
    assert redactor.redact("alex@example.com extra") == "{{WIDE}}"


def test_add_detector_adds_custom_detector():
    redactor = redactix.Redactor()
    redactor.add_detector("name", r"\bJane Doe\b")
    assert redactor.redact("Jane Doe emailed alex@example.com") == "{{NAME}} emailed alex@example.com"


def test_add_detector_fails_if_name_exists():
    redactor = redactix.Redactor(custom_detectors={"name": r"\bJane Doe\b"})
    with pytest.raises(ValueError):
        redactor.add_detector("name", r"\bJohn Doe\b")


def test_add_detector_fails_if_builtin_name_exists():
    redactor = redactix.Redactor(detectors=["email"])
    with pytest.raises(ValueError):
        redactor.add_detector("email", r"alex@example\.com")


def test_replace_detector_overrides_custom_detector():
    redactor = redactix.Redactor(custom_detectors={"name": r"\bJane Doe\b"})
    redactor.replace_detector("name", r"\bJohn Doe\b")
    assert redactor.redact("Jane Doe and John Doe") == "Jane Doe and {{NAME}}"


def test_replace_detector_overrides_builtin_detector():
    redactor = redactix.Redactor(detectors=["email"])
    redactor.replace_detector("email", r"alex@example\.com")
    assert redactor.redact("Email alex@example.com or bob@example.com") == (
        "Email {{EMAIL}} or bob@example.com"
    )


def test_replace_detector_fails_if_name_does_not_exist():
    redactor = redactix.Redactor()
    with pytest.raises(ValueError):
        redactor.replace_detector("name", r"\bJane Doe\b")


def test_remove_detector_disables_builtin():
    redactor = redactix.Redactor(default_detectors=True)
    redactor.remove_detector("email")
    assert redactor.redact("Email alex@example.com. Card: 4111 1111 1111 1111.") == (
        "Email alex@example.com. Card: {{CREDIT_CARD}}."
    )


def test_remove_detector_disables_custom_detector():
    redactor = redactix.Redactor(custom_detectors={"name": r"\bJane Doe\b"})
    redactor.remove_detector("name")
    assert redactor.redact("Jane Doe") == "Jane Doe"


def test_remove_detector_fails_if_name_does_not_exist():
    redactor = redactix.Redactor()
    with pytest.raises(ValueError):
        redactor.remove_detector("name")
