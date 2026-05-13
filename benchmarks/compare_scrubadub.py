from __future__ import annotations

import argparse
import gc
import importlib.util
import statistics
import time
from collections.abc import Callable

import redactix


SAMPLE_TEXT = """
Jane Doe can be reached at alex@example.com or +1 415-555-2671.
Backup contact: jordan.smith@example.org, phone 212-555-0198.
Billing card on file: 4111 1111 1111 1111.
Internal note: keep the original punctuation and sentence boundaries intact.
""".strip()


def build_payload(repetitions: int) -> str:
    return "\n".join(SAMPLE_TEXT for _ in range(repetitions))


def time_function(func: Callable[[str], str], text: str, iterations: int) -> list[float]:
    timings = []
    gc_was_enabled = gc.isenabled()
    gc.disable()
    try:
        for _ in range(iterations):
            started = time.perf_counter()
            func(text)
            timings.append(time.perf_counter() - started)
    finally:
        if gc_was_enabled:
            gc.enable()
    return timings


def summarize(label: str, timings: list[float], characters: int) -> dict[str, str]:
    mean_seconds = statistics.mean(timings)
    median_seconds = statistics.median(timings)
    chars_per_second = characters / mean_seconds if mean_seconds else 0.0
    return {
        "library": label,
        "mean_ms": f"{mean_seconds * 1000:.3f}",
        "median_ms": f"{median_seconds * 1000:.3f}",
        "chars_per_sec": f"{chars_per_second:,.0f}",
    }


def print_table(rows: list[dict[str, str]]) -> None:
    columns = ["library", "mean_ms", "median_ms", "chars_per_sec"]
    headers = {
        "library": "Library",
        "mean_ms": "Mean ms",
        "median_ms": "Median ms",
        "chars_per_sec": "Chars/sec",
    }
    widths = {
        column: max(len(headers[column]), *(len(row[column]) for row in rows))
        for column in columns
    }

    print(" | ".join(headers[column].ljust(widths[column]) for column in columns))
    print(" | ".join("-" * widths[column] for column in columns))
    for row in rows:
        print(" | ".join(row[column].ljust(widths[column]) for column in columns))


def scrubadub_available() -> bool:
    return importlib.util.find_spec("scrubadub") is not None


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Compare Redactix redaction speed with Scrubadub on a fixed local text payload.",
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=100,
        help="Number of timed redaction runs per library.",
    )
    parser.add_argument(
        "--repetitions",
        type=int,
        default=100,
        help="Number of times to repeat the sample text in the benchmark payload.",
    )
    args = parser.parse_args()

    if args.iterations < 1:
        raise SystemExit("--iterations must be at least 1")
    if args.repetitions < 1:
        raise SystemExit("--repetitions must be at least 1")

    text = build_payload(args.repetitions)
    characters = len(text)

    rows = [
        summarize(
            "Redactix",
            time_function(redactix.redact, text, args.iterations),
            characters,
        )
    ]

    if scrubadub_available():
        import scrubadub

        rows.append(
            summarize(
                "Scrubadub",
                time_function(scrubadub.clean, text, args.iterations),
                characters,
            )
        )
    else:
        print("Scrubadub is not installed; install it to include the comparison.")
        print("Example: uv pip install scrubadub")
        print()

    print(f"Payload: {characters:,} characters")
    print(f"Iterations: {args.iterations}")
    print_table(rows)


if __name__ == "__main__":
    main()
