#!/usr/bin/env python3
"""Run manifest workloads and emit reproducible, diagnostic benchmark evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

try:
    import resource
except ImportError:  # Windows does not expose getrusage.
    resource = None


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def child_peak_rss_kib() -> int | None:
    """Return cumulative child peak RSS normalized to KiB on macOS/Linux."""
    if resource is None:
        return None
    value = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    return round(value / 1024) if sys.platform == "darwin" else round(value)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=Path("benchmarks/manifest.json"))
    parser.add_argument("--output", type=Path, default=Path("benchmarks/latest.json"))
    parser.add_argument("--release", action="store_true", help="Use cargo --release")
    parser.add_argument("--skip-build", action="store_true", help="Use an existing target binary")
    parser.add_argument("--iterations", type=int, default=3, help="Runs per workload (first is cold)")
    args = parser.parse_args()
    if args.iterations < 1:
        parser.error("--iterations must be at least 1")

    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    root = args.manifest.resolve().parent.parent
    profile = "release" if args.release else "debug"
    binary = root / "target" / profile / ("ajisai-cli.exe" if os.name == "nt" else "ajisai-cli")
    build_command = ["cargo", "build", "--bin", "ajisai-cli"]
    if args.release:
        build_command.insert(2, "--release")
    build = None
    if not args.skip_build:
        build = subprocess.run(build_command, cwd=root, text=True, capture_output=True, check=False)
    elif not binary.exists():
        build = subprocess.CompletedProcess(build_command, 1, "", f"missing benchmark binary: {binary}")
    if build is not None and build.returncode != 0:
        report = {
            "schema_version": 1,
            "manifest": str(args.manifest),
            "diagnostic_only": True,
            "build": {"command": build_command, "exit_code": build.returncode, "stderr": build.stderr},
            "records": [],
        }
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(report, ensure_ascii=False, indent=2))
        return 1
    records = []
    with tempfile.TemporaryDirectory(prefix="ajisai-bench-") as temp_dir:
        for workload in manifest["workloads"]:
            for iteration in range(args.iterations):
                output_path = Path(temp_dir) / f"{workload['id']}-{iteration}.csv"
                command = [
                    str(binary),
                    "run",
                    "-p",
                    workload["pipeline"],
                    "--env",
                    f"OUTPUT_FILE={output_path}",
                    "--json",
                ]
                started = time.perf_counter()
                completed = subprocess.run(command, cwd=root, text=True, capture_output=True, check=False)
                elapsed_ms = round((time.perf_counter() - started) * 1000, 3)
                record = {
                    "id": workload["id"],
                    "iteration": iteration + 1,
                    "startup_phase": "cold" if iteration == 0 else "warm",
                    "command": command,
                    "exit_code": completed.returncode,
                    "elapsed_ms_wall": elapsed_ms,
                    "child_peak_rss_kib_cumulative": child_peak_rss_kib(),
                    "stdout": completed.stdout.strip(),
                    "stderr": completed.stderr.strip(),
                }
                if output_path.exists():
                    record["output_bytes"] = output_path.stat().st_size
                    record["output_sha256"] = sha256(output_path)
                    record["output_matches_expected"] = (
                        record["output_sha256"] == workload["expected_sha256"]
                    )
                records.append(record)

    report = {
        "schema_version": 1,
        "manifest": str(args.manifest),
        "diagnostic_only": True,
        "build": {"command": build_command, "exit_code": 0, "skipped": args.skip_build, "binary": str(binary)},
        "iterations_per_workload": args.iterations,
        "records": records,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if all(item["exit_code"] == 0 and item.get("output_matches_expected", False) for item in records) else 1


if __name__ == "__main__":
    raise SystemExit(main())
