#!/usr/bin/env python3
"""Write deterministic build-input provenance for a release artifact."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess


def command_output(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path("ajisai-build-info.json"))
    args = parser.parse_args()
    info = {
        "schema_version": 1,
        "project": "ajisai",
        "source_revision": command_output(["git", "rev-parse", "HEAD"]),
        "source_ref": os.environ.get("GITHUB_REF_NAME", ""),
        "target": os.environ.get("TARGET", ""),
        "runner_os": os.environ.get("RUNNER_OS", ""),
        "rustc": command_output(["rustc", "-Vv"]),
        "cargo": command_output(["cargo", "-V"]),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(info, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
