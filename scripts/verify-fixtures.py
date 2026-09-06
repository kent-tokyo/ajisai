#!/usr/bin/env python3
"""Verify the immutable regression fixture corpus against its hash manifest."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=Path("tests/fixtures/manifest.json"))
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    root = args.manifest.parent
    mismatches = []
    for fixture in manifest["fixtures"]:
        path = root / fixture["path"]
        actual = digest(path) if path.is_file() else "missing"
        if actual != fixture["sha256"]:
            mismatches.append(f"{fixture['path']}: expected {fixture['sha256']}, got {actual}")
    if mismatches:
        for mismatch in mismatches:
            print(mismatch)
        return 1
    print(f"verified {len(manifest['fixtures'])} immutable fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
