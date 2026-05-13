from typing import Literal, Sequence, TypedDict

RedactionMode = Literal["placeholder", "mask"]
BuiltinPattern = Literal["email", "phone", "credit_card"]

class Match(TypedDict):
    type: str
    start: int
    end: int
    text: str
    replacement: str

class Redactor:
    def __init__(
        self,
        custom_patterns: dict[str, str] | None = None,
        placeholders: dict[str, str] | None = None,
        mode: RedactionMode = "placeholder",
        patterns: Sequence[BuiltinPattern] | None = None,
        default_patterns: bool = False,
    ) -> None: ...
    def detect(self, text: str) -> list[Match]: ...
    def redact(self, text: str, mode: RedactionMode | None = None) -> str: ...
    def add_pattern(self, name: str, pattern: str) -> None: ...
    def replace_pattern(self, name: str, pattern: str) -> None: ...
    def remove_pattern(self, name: str) -> None: ...

def detect(text: str) -> list[Match]: ...
def redact(text: str, mode: RedactionMode = "placeholder") -> str: ...
