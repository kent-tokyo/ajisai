#!/usr/bin/env python3
"""Verify a release artifact directory from its SHA256SUMS and provenance files."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import re


CHECKSUM = re.compile(r"^([0-9a-fA-F]{64})  (.+)$")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    root = args.directory.resolve()
    checksum_file = root / "SHA256SUMS"
    if not checksum_file.is_file():
        print(f"missing {checksum_file}")
        return 1
    failures = []
    entries = []
    for line in checksum_file.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        match = CHECKSUM.match(line)
        if not match:
            failures.append(f"invalid checksum line: {line}")
            continue
        expected, relative = match.groups()
        path = (root / relative).resolve()
        if root not in path.parents or not path.is_file():
            failures.append(f"missing or unsafe artifact: {relative}")
            continue
        entries.append((relative, expected.lower(), path))
    for relative, expected, path in entries:
        actual = sha256(path)
        if actual != expected:
            failures.append(f"checksum mismatch: {relative}")
    for required in ("ajisai-sbom.cdx.json", "ajisai-build-info.json"):
        if not (root / required).is_file():
            failures.append(f"missing provenance file: {required}")
    if failures:
        for failure in failures:
            print(failure)
        return 1
    print(f"verified {len(entries)} release artifacts and provenance files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
