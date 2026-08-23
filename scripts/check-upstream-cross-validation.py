#!/usr/bin/env python3
"""Recompute the committed NeuroM and L-Measure cross-validation fixture."""

from __future__ import annotations

import math
import os
from pathlib import Path
import platform
import stat
import subprocess
import sys
import tempfile


EXPECTED_NEUROM_VERSION = "4.0.5"
EXPECTED_LMEASURE_VERSION = "5.2-revision-510"
REPOSITORY = Path(__file__).resolve().parents[1]
FIXTURE = REPOSITORY / "wasm-plugin" / "test-data" / "metric-cross-validation.tsv"

SWC_FIXTURES = {
    "orthogonal-binary": """\
1 1 -2 0 0 2 -1
2 3 -1 0 0 1.5 1
3 3 0 0 0 1 2
4 3 1 0 0 0.5 3
5 3 0 1 0 0.5 3
""",
    # L-Measure 5.2 treats a one-point soma-to-neurite transition as an extra
    # Pk_classic compartment. The soma-free form isolates the one intended
    # binary neurite bifurcation; NeuroM uses the equivalent soma-bearing form.
    "orthogonal-binary-lmeasure": """\
1 3 -1 0 0 1.5 -1
2 3 0 0 0 1 1
3 3 1 0 0 0.5 2
4 3 0 1 0 0.5 2
""",
    "linear-taper": """\
1 1 -1 0 0 2 -1
2 3 0 0 0 2 1
3 3 1 0 0 1.5 2
4 3 2 0 0 1 3
""",
    "right-angle-chain": """\
1 1 -1 0 0 1 -1
2 3 0 0 0 1 1
3 3 1 0 0 1 2
4 3 1 1 0 1 3
5 3 1 2 0 1 4
""",
}

FIELD_NAMES = [
    "metric_id",
    "definition_version",
    "fixture",
    "upstream",
    "upstream_version",
    "upstream_callable",
    "upstream_units",
    "upstream_value",
    "axodendron_value",
    "compatibility",
    "tolerance",
    "known_difference",
]


def parse_numbers(value: str) -> list[float]:
    return [float(part) for part in value.split(",")]


def assert_close(actual: list[float], expected: list[float], tolerance: float, label: str) -> None:
    if len(actual) != len(expected):
        raise AssertionError(f"{label}: expected {len(expected)} values, got {len(actual)}")
    for index, (left, right) in enumerate(zip(actual, expected, strict=True)):
        if not math.isfinite(left) or abs(left - right) > tolerance:
            raise AssertionError(
                f"{label}[{index}]: expected {right:.17g} +/- {tolerance:g}, got {left:.17g}"
            )


def find_bifurcation(morphology):
    points = [section for section in morphology.sections if len(section.children) == 2]
    if len(points) != 1:
        raise AssertionError(f"expected exactly one binary bifurcation, got {len(points)}")
    return points[0]


def neurom_values(paths: dict[str, Path]) -> dict[str, list[float]]:
    try:
        import neurom
        from neurom import features
    except ImportError as error:
        raise SystemExit(
            f"NeuroM {EXPECTED_NEUROM_VERSION} is required: "
            f"{sys.executable} -m pip install neurom=={EXPECTED_NEUROM_VERSION}"
        ) from error

    if neurom.__version__ != EXPECTED_NEUROM_VERSION:
        raise SystemExit(
            f"expected NeuroM {EXPECTED_NEUROM_VERSION}, found {neurom.__version__}"
        )

    branch = neurom.load_morphology(paths["orthogonal-binary"])
    bifurcation = find_bifurcation(branch)
    taper = list(neurom.load_morphology(paths["linear-taper"]).sections)
    meander = list(neurom.load_morphology(paths["right-angle-chain"]).sections)
    if len(taper) != 1 or len(meander) != 1:
        raise AssertionError("linear fixtures must each decompose into one NeuroM section")

    return {
        "local-bifurcation-angle": [
            float(features.bifurcation.local_bifurcation_angle(bifurcation))
        ],
        "remote-bifurcation-angle": [
            float(features.bifurcation.remote_bifurcation_angle(bifurcation))
        ],
        "sibling-ratio": [float(features.bifurcation.sibling_ratio(bifurcation))],
        "partition-asymmetry-terminal": [
            float(features.bifurcation.partition_asymmetry(bifurcation, uylings=True))
        ],
        "taper-rate": [float(features.section.taper_rate(taper[0]))],
        "segment-meander-angle": [
            float(value) for value in features.section.section_meander_angles(meander[0])
        ],
    }


def locate_lmeasure() -> Path:
    configured = os.environ.get("LMEASURE_EXEC")
    if configured:
        executable = Path(configured).expanduser().resolve()
    else:
        try:
            import importlib.util

            spec = importlib.util.find_spec("pylmeasure")
        except (ImportError, ValueError):
            spec = None
        if spec is None or spec.origin is None:
            raise SystemExit(
                "L-Measure is required; set LMEASURE_EXEC or install the validation wrapper "
                f"with {sys.executable} -m pip install pylmeasure==0.2.0"
            )
        package = Path(spec.origin).resolve().parent
        system = platform.system()
        if system == "Darwin":
            executable = package / "LMMac" / "lmeasure"
        elif system == "Linux":
            bits = platform.architecture()[0][:2]
            executable = package / f"LMLinux{bits}" / "lmeasure"
        elif system == "Windows":
            executable = package / "LMWin" / "Lm.exe"
        else:
            raise SystemExit(f"unsupported L-Measure validation platform: {system}")
    if not executable.is_file():
        raise SystemExit(f"L-Measure executable not found: {executable}")
    executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
    return executable


def lmeasure_value(executable: Path, swc: Path, temporary: Path) -> float:
    output = temporary / "lmeasure-output.txt"
    request = temporary / "lmeasure-input.txt"
    request.write_text(
        f"-f31,0,0,10\n-s{output}\n{swc.resolve()}\n",
        encoding="ascii",
    )
    process = subprocess.run(
        [str(executable), str(request)],
        check=True,
        capture_output=True,
        text=True,
    )
    banner = process.stdout + process.stderr
    if "Release Lmv5.2" not in banner or "REVISION: 510" not in banner:
        raise SystemExit("expected L-Measure 5.2 revision 510; received an unrecognized banner")
    line = output.read_text(encoding="ascii").strip()
    fields = line.split("\t")
    if len(fields) < 9 or fields[1].strip() != "Pk_classic":
        raise AssertionError(f"unexpected L-Measure Pk_classic output: {line!r}")
    return float(fields[2])


def load_rows() -> list[dict[str, str]]:
    with FIXTURE.open(newline="", encoding="utf-8") as stream:
        return [
            dict(zip(FIELD_NAMES, line.rstrip("\n").split("\t"), strict=True))
            for line in stream
            if line.strip() and not line.startswith("#")
        ]


def main() -> None:
    rows = load_rows()
    with tempfile.TemporaryDirectory(prefix="axodendron-upstream-") as directory:
        temporary = Path(directory)
        paths = {}
        for name, source in SWC_FIXTURES.items():
            path = temporary / f"{name}.swc"
            path.write_text(source, encoding="ascii")
            paths[name] = path

        observed = neurom_values(paths)
        observed["rall-ratio"] = [
            lmeasure_value(locate_lmeasure(), paths["orthogonal-binary-lmeasure"], temporary)
        ]

        for row in rows:
            metric_id = row["metric_id"]
            if row["definition_version"] != "1":
                raise AssertionError(f"{metric_id}: unexpected definition version")
            expected_version = (
                EXPECTED_NEUROM_VERSION if row["upstream"] == "NeuroM" else EXPECTED_LMEASURE_VERSION
            )
            if row["upstream_version"] != expected_version:
                raise AssertionError(f"{metric_id}: fixture upstream version is not pinned")
            tolerance = float(row["tolerance"])
            assert_close(
                observed[metric_id],
                parse_numbers(row["upstream_value"]),
                tolerance,
                f"{row['upstream']}:{metric_id}",
            )

            upstream = observed[metric_id]
            if row["compatibility"] == "unit-converted":
                comparable = [math.degrees(value) for value in upstream]
            elif row["compatibility"] == "convention-transformed":
                comparable = [180.0 - math.degrees(value) for value in upstream]
            else:
                comparable = upstream
            assert_close(
                comparable,
                parse_numbers(row["axodendron_value"]),
                tolerance,
                f"compatibility:{metric_id}",
            )

    print(
        f"verified {len(rows)} metric definitions against NeuroM {EXPECTED_NEUROM_VERSION} "
        f"and L-Measure {EXPECTED_LMEASURE_VERSION}"
    )


if __name__ == "__main__":
    main()
