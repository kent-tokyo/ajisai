# ADR-0004: Row and columnar representation boundary

## Status

Accepted for the initial engine.

## Decision

The public Transform contract remains row-oriented for Hop compatibility and
simple streaming backpressure. Columnar or batched execution may be added
behind an internal adapter only after a benchmark demonstrates a material gain
for a named workload. Adapters must preserve null, ordering, and error-row
semantics.

## Consequences

The first release is easy to reason about and migrate. Fully buffered
operators must declare their buffering class and cannot silently switch
representation.
