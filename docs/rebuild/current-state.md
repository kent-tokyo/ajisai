# Current-state audit (rebuild baseline)

**Date:** 2026-09-06  
**Scope:** repository-local inventory only; this is not a runtime or release certification.

## Inventory

| Area | Current evidence | Rebuild disposition |
|---|---|---|
| Workspace | `crates/core`, `crates/transforms`, `crates/hop-compat`, `crates/cli`, `crates/server` in the root workspace | Reuse candidates; validate boundaries in Phase 0/1 |
| Execution | `ajisai-core::PipelineEngine` uses per-hop bounded Tokio channels and one task per node; side inputs are collected before main processing | Rewrite/benchmark candidate; buffering, cancellation, and commit semantics need contracts |
| Data model | `PipelineState`, `Node`, `Edge`, `Row`, `RowSchema`, and `Value` are serializable/runtime types | Split native document, execution IR, and data contracts |
| Transforms | `crates/transforms/src` contains file, database, HTTP, join, aggregate, and row transforms; scripting is an explicit opt-in feature | Reclassify by manifest, capability, buffering, and semantic test coverage |
| Hop input | `crates/hop-compat` parses Hop pipeline/workflow XML, emits transform assessments, and inventories project pipelines/workflows without execution | Isolated importer; never equate parsing with behavioral compatibility |
| CLI | Commands include `new`, `run`, `validate`, `run-workflow`, `list-transforms`, `assess`, `scan`, `migrate`, `doctor`, `explain`, `inspect`, and `preview`; both run commands emit versioned JSON summaries with `run_id`, persist records atomically, accept project roots, and support side-effect-free dry-runs; localized errors retain stable exit codes | Preserve automation boundary only after stable command/exit/JSON contracts; UI correlation and full workflow UX remain open |
| Desktop UI | Electron + React renderer, Rust JSON-RPC sidecar, localStorage autosave | Reuse only as a client prototype; backend state and failure recovery require redesign |
| CI | Rust format/build/test matrix is present for three OSes; dependency audit runs in CI and again in the release artifact job; Electron/package checks are separate or absent | Add unified contract, accessibility, security, and package gates |
| Fixtures | `tests/fixtures/manifest.json` pins 12 definitions/oracles; `scripts/verify-fixtures.py` verifies SHA-256 and generated outputs are explicitly mutable | Add broader generated workloads and independent Hop oracles |
| Version sources | Root Rust workspace is `0.1.0`; `electron/package.json` is `0.3.0`; target stable release is v2.0.0 | Resolve one authoritative source and bump only after v2.0 gates pass |

## Claims ledger

| Claim area | Current state | Evidence needed before public claim |
|---|---|---|
| Hop compatibility | Parser, project scan, and explicit transform assessment exist | Version-pinned differential behavior matrix and real-project validation |
| Processing speed | Native async/parallel implementation exists; `benchmarks/manifest.json` and `scripts/run-benchmark.py` provide hash-checked diagnostic runs with cold/warm labels | Same semantics, fixtures, hardware, repeated measurements, and RSS/disk instrumentation against Hop 2.19.0 |
| Memory | Bounded edge channels exist, but operators may buffer | Peak RSS and spill tests by operator class and input scale |
| UI usability | Canvas, forms, autosave, and logs exist | Golden-task study, UI/API fault injection, accessibility checks |
| Security | Some path checks and Electron isolation settings exist; `run --project-root` scopes current filesystem transforms, including row-derived `LoadFileContent` paths | Threat model, connector-wide policy audit, parser fuzzing, dependency review, and independent review |
| Installation | Cargo and Electron build/release definitions exist | Clean-machine install, signing, update/rollback, and uninstall evidence |

## Current high-risk boundaries

These are risks to address, not confirmed vulnerabilities:

1. Arbitrary path, network, database, XML, spreadsheet, and script configuration crosses the transform boundary.
2. The current UI receives sidecar notifications and maintains renderer state; delayed, duplicated, or dropped events need authoritative reconciliation tests.
3. The current engine's side-input collection and buffering transforms can defeat a global memory target.
4. Hop XML parsing can preserve text while still changing semantics, ordering, metadata, or side effects.
5. Release workflows build and smoke-test standalone CLI artifacts; a pinned non-root container definition is also checked in, but publication, container digest, signature, updater, and clean-machine behavior are separate facts.

## Audit conclusion

The repository is a useful prototype and fixture source. It is not yet evidence of a stable product, complete Hop compatibility, or competitive performance. Phase 0 therefore proceeds with contract-first rewrite slices and retains existing code only when a test demonstrates that it satisfies the new boundary.
