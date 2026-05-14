from typing import Literal, Optional, Sequence, TypedDict

MaskStrategy = Literal["placeholder", "fixed", "length_preserving"]
BuiltinDetector = Literal["email", "phone", "credit_card"]

class RawDetection(TypedDict):
    start: int
    end: int
    value: str
    entity_type: str
    detector_name: str
    replacement: str

class RawRedactionResult(TypedDict):
    text: str
    detections: list[RawDetection]

class Redactor:
    def __init__(
        self,
        detectors: Optional[Sequence[BuiltinDetector]] = None,
        mask_strategy: MaskStrategy = "placeholder",
        placeholder_format: str = "{{{entity_type}}}",
        mask_char: str = "*",
        fixed_mask: str = "***",
    ) -> None: ...
    def detect(self, text: str) -> list[RawDetection]: ...
    def redact(self, text: str) -> str: ...
    def redact_with_report(self, text: str) -> RawRedactionResult: ...
    def register_detector(
        self,
        name: str,
        pattern: str,
        placeholder: Optional[str] = None,
        enabled: bool = True,
        priority: int = 100,
    ) -> None: ...

def detect(
    text: str,
    detectors: Optional[Sequence[BuiltinDetector]] = None,
    mask_strategy: MaskStrategy = "placeholder",
    placeholder_format: str = "{{{entity_type}}}",
    mask_char: str = "*",
    fixed_mask: str = "***",
) -> list[RawDetection]: ...

def redact(
    text: str,
    detectors: Optional[Sequence[BuiltinDetector]] = None,
    mask_strategy: MaskStrategy = "placeholder",
    placeholder_format: str = "{{{entity_type}}}",
    mask_char: str = "*",
    fixed_mask: str = "***",
) -> str: ...

def redact_with_report(
    text: str,
    detectors: Optional[Sequence[BuiltinDetector]] = None,
    mask_strategy: MaskStrategy = "placeholder",
    placeholder_format: str = "{{{entity_type}}}",
    mask_char: str = "*",
    fixed_mask: str = "***",
) -> RawRedactionResult: ...
