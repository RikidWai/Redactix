import redactix


text = "Contact me at alex@example.com or +1 415-555-2671. Card: 4111 1111 1111 1111."

print(redactix.detect(text))
print(redactix.redact(text))
print(redactix.redact(text, mode="mask"))

redactor = redactix.Redactor(custom_patterns={"name": r"\bJane Doe\b"})
print(redactor.redact("Jane Doe emailed alex@example.com"))
