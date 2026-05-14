from typing import Literal, Sequence, TypedDict

RedactionMode = Literal["placeholder", "mask"]
BuiltinDetector = Literal["email", "phone", "credit_card"]

class Match(TypedDict):
    type: str
    start: int
    end: int
    text: str
    replacement: str

class Redactor:
    def __init__(
        self,
        custom_detectors: dict[str, str] | None = None,
        placeholders: dict[str, str] | None = None,
        mode: RedactionMode = "placeholder",
        detectors: Sequence[BuiltinDetector] | None = None,
        default_detectors: bool = False,
    ) -> None: ...
    def detect(self, text: str) -> list[Match]: ...
    def redact(self, text: str, mode: RedactionMode | None = None) -> str: ...
    def add_detector(self, name: str, regex: str) -> None: ...
    def replace_detector(self, name: str, regex: str) -> None: ...
    def remove_detector(self, name: str) -> None: ...

def detect(text: str) -> list[Match]: ...
def redact(text: str, mode: RedactionMode = "placeholder") -> str: ...
