#!/usr/bin/env python3
"""Create a deterministic CycloneDX JSON SBOM from cargo metadata."""

import json
import argparse
import pathlib
import subprocess


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", default="ajisai-sbom.cdx.json")
    output = pathlib.Path(parser.parse_args().output)
    metadata = json.loads(
        subprocess.check_output(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            text=True,
        )
    )
    components = []
    for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
        component = {
            "type": "library",
            "bom-ref": f"pkg:cargo/{package['name']}@{package['version']}",
            "name": package["name"],
            "version": package["version"],
            "purl": f"pkg:cargo/{package['name']}@{package['version']}",
        }
        if package.get("license"):
            license_value = package["license"]
            license_key = "expression" if (" OR " in license_value or " AND " in license_value) else "id"
            component["licenses"] = [{"license": {license_key: license_value}}]
        components.append(component)

    bom = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "version": 1,
        "metadata": {"tools": [{"vendor": "Ajisai", "name": "cargo-metadata-sbom"}]},
        "components": components,
    }
    output.write_text(json.dumps(bom, ensure_ascii=False, indent=2) + "\n")
    print(output)


if __name__ == "__main__":
    main()
