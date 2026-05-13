from typing import Literal, Sequence

RedactionMode = Literal["placeholder", "mask"]
BuiltinPattern = Literal["email", "phone", "credit_card"]

class Redactor:
    def __init__(
        self,
        custom_patterns: dict[str, str] | None = None,
        placeholders: dict[str, str] | None = None,
        mode: RedactionMode = "placeholder",
        patterns: Sequence[BuiltinPattern] | None = None,
    ) -> None: ...
    def detect(self, text: str) -> list[dict]: ...
    def redact(self, text: str, mode: RedactionMode | None = None) -> str: ...

def detect(text: str) -> list[dict]: ...
def redact(text: str, mode: RedactionMode = "placeholder") -> str: ...
