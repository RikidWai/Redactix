import redactix


def test_detect_email():
    assert redactix.detect("Email alex@example.com")[0]["type"] == "email"


def test_detect_phone():
    assert redactix.detect("Call +1 415-555-2671")[0]["type"] == "phone"


def test_detect_credit_card():
    matches = redactix.detect("Card 4111 1111 1111 1111")
    assert matches[0]["type"] == "credit_card"


def test_invalid_credit_card_is_not_detected():
    assert redactix.detect("Card 1234 5678 9012 3456") == []


def test_credit_card_inside_long_digit_run_is_not_detected():
    assert redactix.detect("Token 04111111111111111111") == []


def test_phone_inside_word_is_not_detected():
    assert redactix.detect("refA415-555-2671") == []


def test_phone_after_plus_boundary_is_not_detected():
    assert redactix.detect("Call ++1 415-555-2671") == []


def test_detect_builtin_example():
    text = "Contact me at alex@example.com or +1 415-555-2671. Card: 4111 1111 1111 1111."
    assert redactix.detect(text) == [
        {
            "type": "email",
            "start": 14,
            "end": 30,
            "text": "alex@example.com",
            "replacement": "{{EMAIL}}",
        },
        {
            "type": "phone",
            "start": 34,
            "end": 49,
            "text": "+1 415-555-2671",
            "replacement": "{{PHONE}}",
        },
        {
            "type": "credit_card",
            "start": 57,
            "end": 76,
            "text": "4111 1111 1111 1111",
            "replacement": "{{CREDIT_CARD}}",
        },
    ]


def test_character_indexes_are_python_indexes():
    matches = redactix.detect("Email 😄 alex@example.com")
    assert matches[0]["start"] == 8
    assert matches[0]["end"] == 24
