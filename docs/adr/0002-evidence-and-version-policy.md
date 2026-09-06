# ADR-0002: Evidence and version policy

**Status:** Proposed for Phase 0 review  
**Date:** 2026-09-05

## Decision

Every performance, memory, compatibility, security, installation, and usability result records:

- product and dependency versions;
- source revision and platform/toolchain;
- fixture or dataset ID and input/output hashes;
- exact command and policy configuration;
- repetitions, summary statistic, spread, and failures;
- whether the result is local, heavy, or external.

The initial Hop oracle is Apache Hop 2.19.0. A later oracle requires an explicit baseline-update decision and a fresh differential run. The Rust workspace and Electron package version mismatch is recorded as a Phase 0 finding; this ADR does not change either version.

## Consequences

- README estimates cannot be used as measurement evidence.
- A source review, fixture, or parser test cannot be reported as runtime parity.
- Published release, registry, signature, updater, and clean-install checks remain separate gates.
