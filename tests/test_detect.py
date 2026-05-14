import redactix
from redactix import Detection


def test_detect_email():
    assert redactix.detect("Email alex@example.com")[0].entity_type == "EMAIL"


def test_detect_phone():
    assert redactix.detect("Call +1 415-555-2671")[0].entity_type == "PHONE"


def test_detect_credit_card():
    matches = redactix.detect("Card 4111 1111 1111 1111")
    assert matches[0].entity_type == "CREDIT_CARD"


def test_invalid_credit_card_is_not_detected():
    assert redactix.detect("Card 1234 5678 9012 3456") == []


def test_credit_card_inside_long_digit_run_is_not_detected():
    assert redactix.detect("Token 04111111111111111111") == []


def test_phone_inside_word_is_not_detected():
    assert redactix.detect("refA415-555-2671") == []


def test_phone_after_plus_boundary_is_not_detected():
    assert redactix.detect("Call ++1 415-555-2671") == []


def test_implausible_phone_is_not_detected():
    assert redactix.detect("Call 000-000-0000") == []


def test_detect_builtin_example():
    text = "Contact me at alex@example.com or +1 415-555-2671. Card: 4111 1111 1111 1111."
    assert redactix.detect(text) == [
        Detection(
            start=14,
            end=30,
            value="alex@example.com",
            entity_type="EMAIL",
            detector_name="email",
            replacement="{{EMAIL}}",
        ),
        Detection(
            start=34,
            end=49,
            value="+1 415-555-2671",
            entity_type="PHONE",
            detector_name="phone",
            replacement="{{PHONE}}",
        ),
        Detection(
            start=57,
            end=76,
            value="4111 1111 1111 1111",
            entity_type="CREDIT_CARD",
            detector_name="credit_card",
            replacement="{{CREDIT_CARD}}",
        ),
    ]


def test_character_indexes_are_python_indexes():
    matches = redactix.detect("Email 😄 alex@example.com")
    assert matches[0].start == 8
    assert matches[0].end == 24
