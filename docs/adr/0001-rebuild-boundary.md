# ADR-0001: Ajisai rebuild boundary

**Status:** Proposed for Phase 0 review  
**Date:** 2026-09-05

## Decision

Ajisai is rebuilt around versioned native **pipeline and workflow documents**, a validated execution plan, a bounded local runner, and a thin Studio/CLI client boundary. Pipeline remains the row-stream transformation graph; workflow remains the orchestration graph for pipeline actions, conditions, retries, and operational steps. Apache Hop support is an importer and compatibility-report subsystem. It is not the native runtime representation.

The primary product measure is a beginner's ability to create, validate, understand, and recover a pipeline. Processing performance remains a first-class constraint: the engine must meet or improve on the pinned Apache Hop 2.19.0 baseline under equivalent semantics, rather than relying on assumptions about JVM cost.

Airbyte is a UI and operational-flow reference for source/destination/connection onboarding. Ajisai may adopt interaction principles such as progressive disclosure, immediate connection tests, useful defaults, and a review step. It will not copy Airbyte's product scope or treat an unverified user report about an Airbyte bug as a factual claim.

## Consequences

- Existing crates and the Electron UI can be reused only behind the new contracts.
- Transform count is not a milestone; each retained Transform needs a manifest, semantic tests, capability declaration, and buffering class.
- Hop import must surface differences before execution and quarantine unsupported content.
- Workflow/pipeline names, responsibilities, branching outcomes, and pipeline-action nesting remain compatible concepts even when the Studio presents them through a simpler guided flow.
- UI success states require durable backend acknowledgement and a recoverable run record.
- A high-performance path that violates a memory or capability policy fails validation.

## Rejected alternatives

- Reproducing Hop's menu and perspective structure: this preserves the primary usability problem.
- Replacing Hop with a connector-only replication product: this misses visual transformation and local ETL needs.
- Calling XML parsing "compatibility": syntax preservation does not prove runtime behavior.
