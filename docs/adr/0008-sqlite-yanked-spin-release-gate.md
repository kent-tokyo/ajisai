# ADR 0008: yanked `spin` in the SQLite dependency graph

Status: Resolved

## Context

`cargo deny check advisories` previously reported yanked `spin 0.9.8` through
`flume -> sqlx-sqlite`. The warning was transitive and did not affect the
default network policy.

## Decision

- Upgrade to the compatible non-yanked `spin 0.9.9` release and verify the
  locked graph with `cargo deny check advisories`.
- Keep the advisory check in CI and release reports.
- Retain the existing parameterized-query, transaction, and network-policy
  controls for SQLite support.
- Keep SQLite support enabled only with the existing parameterized-query,
  transaction, and network-policy controls.

## Consequences

The dependency gate is reproducible and auditable. The yanked-package blocker
is closed; independent security review and artifact verification remain
separate release gates.
