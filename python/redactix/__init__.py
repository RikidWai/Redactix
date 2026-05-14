"""Fast detection and redaction for common structured PII in text."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal, Optional, Sequence

from . import _core

BuiltinDetector = Literal["email", "phone", "credit_card"]
MaskStrategy = Literal["placeholder", "fixed", "length_preserving"]


@dataclass
class Detection:
    start: int
    end: int
    value: str
    entity_type: str
    detector_name: str
    replacement: str


@dataclass
class RedactionResult:
    text: str
    detections: list[Detection]


class Redactor:
    def __init__(
        self,
        detectors: Optional[Sequence[BuiltinDetector]] = None,
        mask_strategy: MaskStrategy = "placeholder",
        placeholder_format: str = "{{{entity_type}}}",
        mask_char: str = "*",
        fixed_mask: str = "***",
    ) -> None:
        self._inner = _core.Redactor(
            detectors=_detector_list(detectors),
            mask_strategy=mask_strategy,
            placeholder_format=placeholder_format,
            mask_char=mask_char,
            fixed_mask=fixed_mask,
        )

    def register_detector(
        self,
        name: str,
        pattern: str,
        placeholder: Optional[str] = None,
        enabled: bool = True,
        priority: int = 100,
    ) -> None:
        self._inner.register_detector(
            name=name,
            pattern=pattern,
            placeholder=placeholder,
            enabled=enabled,
            priority=priority,
        )

    def detect(self, text: str) -> list[Detection]:
        return [_to_detection(detection) for detection in self._inner.detect(text)]

    def redact(self, text: str) -> str:
        return self._inner.redact(text)

    def redact_with_report(self, text: str) -> RedactionResult:
        raw_result = self._inner.redact_with_report(text)
        return RedactionResult(
            text=raw_result["text"],
            detections=[
                _to_detection(detection) for detection in raw_result["detections"]
            ],
        )


def detect(
    text: str,
    detectors: Optional[Sequence[BuiltinDetector]] = None,
    mask_strategy: MaskStrategy = "placeholder",
    placeholder_format: str = "{{{entity_type}}}",
    mask_char: str = "*",
    fixed_mask: str = "***",
) -> list[Detection]:
    redactor = Redactor(
        detectors=detectors,
        mask_strategy=mask_strategy,
        placeholder_format=placeholder_format,
        mask_char=mask_char,
        fixed_mask=fixed_mask,
    )
    return redactor.detect(text)


def redact(
    text: str,
    detectors: Optional[Sequence[BuiltinDetector]] = None,
    mask_strategy: MaskStrategy = "placeholder",
    placeholder_format: str = "{{{entity_type}}}",
    mask_char: str = "*",
    fixed_mask: str = "***",
) -> str:
    redactor = Redactor(
        detectors=detectors,
        mask_strategy=mask_strategy,
        placeholder_format=placeholder_format,
        mask_char=mask_char,
        fixed_mask=fixed_mask,
    )
    return redactor.redact(text)


def redact_with_report(
    text: str,
    detectors: Optional[Sequence[BuiltinDetector]] = None,
    mask_strategy: MaskStrategy = "placeholder",
    placeholder_format: str = "{{{entity_type}}}",
    mask_char: str = "*",
    fixed_mask: str = "***",
) -> RedactionResult:
    redactor = Redactor(
        detectors=detectors,
        mask_strategy=mask_strategy,
        placeholder_format=placeholder_format,
        mask_char=mask_char,
        fixed_mask=fixed_mask,
    )
    return redactor.redact_with_report(text)


def _to_detection(raw_detection: dict[str, object]) -> Detection:
    return Detection(
        start=int(raw_detection["start"]),
        end=int(raw_detection["end"]),
        value=str(raw_detection["value"]),
        entity_type=str(raw_detection["entity_type"]),
        detector_name=str(raw_detection["detector_name"]),
        replacement=str(raw_detection["replacement"]),
    )


def _detector_list(
    detectors: Optional[Sequence[BuiltinDetector]],
) -> Optional[list[BuiltinDetector]]:
    if detectors is None:
        return None
    return list(detectors)


__all__ = [
    "BuiltinDetector",
    "Detection",
    "MaskStrategy",
    "RedactionResult",
    "Redactor",
    "detect",
    "redact",
    "redact_with_report",
]
