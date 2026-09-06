# ADR-0005: Scheduler, backpressure, cancellation, and resource policy

## Status

Accepted for the initial engine.

## Decision

Every edge uses a bounded channel. ExecutionContext carries shared
cancellation, row, timeout, and network policy state. Operators check policy
at node and row boundaries. A policy violation is a terminal failure and never
counts as a successful run. Sinks commit only after close succeeds.

## Consequences

The engine has deterministic stop conditions and bounded in-flight rows.
Memory budgets, spill policy, and per-operator concurrency remain explicit
follow-up extensions rather than hidden scheduler behavior.
