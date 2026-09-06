# ADR-0007: Extension sandbox and capabilities

## Status

Accepted as a safety baseline.

## Decision

Extensions are denied by default and receive only declared capabilities:
filesystem roots, network hosts, secrets, subprocesses, and output targets.
Untrusted native dynamic libraries and unrestricted scripting are not part of
the stable surface. Package hashes, compatibility ranges, and permissions are
validated before loading.

## Consequences

The current built-in registry remains trusted code. A future extension SDK must
implement the capability handshake and out-of-process crash isolation before
being enabled in a stable release.
