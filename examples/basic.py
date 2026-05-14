import redactix


text = "Employee EMP123456 used alex@example.com and 4111 1111 1111 1111."

redactor = redactix.Redactor(
    detectors=["email", "credit_card"],
    mask_strategy="placeholder",
)
redactor.register_detector(
    name="employee_id",
    pattern=r"\bEMP\d{6}\b",
    placeholder="{{EMPLOYEE_ID}}",
)

print(redactor.detect(text))
print(redactor.redact(text))
print(redactor.redact_with_report(text))

fixed = redactix.Redactor(mask_strategy="fixed", fixed_mask="[redacted]")
print(fixed.redact("Email alex@example.com"))
