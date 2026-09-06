# Reproducible artifact evidence

The release workflow keeps the workspace version unchanged and records the
inputs needed to audit each candidate:

- `cargo metadata --locked --format-version 1` captures the resolved package
  graph from `Cargo.lock`.
- `scripts/generate-sbom.py` converts the locked graph into a deterministic
  CycloneDX 1.5 JSON SBOM, including package versions, Cargo PURLs, and license
  identifiers/expressions where Cargo metadata provides them.
- After dependency updates, two consecutive SBOM generations are compared
  byte-for-byte before candidate evidence is accepted.
- `scripts/generate-build-info.py` records the source revision, release ref,
  runner/target, and Rust/Cargo toolchain versions used to assemble the archive.
- Each downloaded platform artifact is hashed in sorted path order into
  `SHA256SUMS`.
- `scripts/verify-release-artifacts.py` verifies every listed hash, rejects
  unsafe/missing paths, and requires the SBOM and build provenance files; the
  release workflow runs this verifier before publication.
- The release workflow signs `SHA256SUMS` with Sigstore keyless OIDC and uploads
  the resulting bundle as `SHA256SUMS.sigstore.json`; consumers can verify the
  bundle with the project identity before trusting the archive hashes.
- The metadata file and checksums are uploaded with the formal GitHub release.
- Each release matrix job also uploads the standalone `ajisai-cli` binary, so
  the same checksum and signing chain covers headless automation as well as the
  Studio package.

This is provenance evidence, not a claim that builds are bit-for-bit identical
across operating systems. A candidate is not publishable until the checksums,
source revision, signing status, and independent download verification are
recorded for every platform.
