# ADR-0006: Studio shell selection

## Status

Provisional; measurement gate open.

## Decision

Studio must use a web-based shell with a local, least-privilege execution
sidecar. The shell may render and edit documents, but never executes
transforms or receives raw secrets. Final framework selection is deferred until
startup, package size, accessibility, updater, and sandbox measurements exist.

## Consequences

UI experimentation can proceed without coupling privileged execution to the
canvas. A 1.0 package cannot claim a final shell choice until the measurements
are retained.
