# ADR-0003: Native schema evolution

## Status

Accepted for the 0.x implementation line.

## Decision

Native documents keep a mandatory `format` identifier and a numeric `version`.
Readers must reject newer major versions and preserve unknown fields when
round-tripping versions they understand. Migrations are explicit functions
from one version to the next; no implicit lossy conversion is allowed.

## Consequences

Fixtures can be pinned to a schema version and upgraded deterministically.
Compatibility reports can distinguish unsupported versions from malformed
documents. A future migration registry remains required before a 1.0 release.
