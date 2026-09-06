# Ajisai Rebuild Roadmap

**Status:** RESET / planning baseline  
**Canonical roadmap:** this file  
**Last updated:** 2026-09-05  
**Version policy:** v2.0.0 is the target stable release for the rebuilt product. This roadmap does not change the current package versions; Phase 0 establishes one authoritative version source before the first publishable tag.

The target definition and release gates are maintained in [`docs/release/v2.0-target.md`](docs/release/v2.0-target.md). Until those gates pass, the workspace remains a `0.1.0` development baseline.

## 1. Product thesis

Ajisai is rebuilt as a local-first visual data pipeline product that a first-time user can install, understand, run, and deploy without learning a large platform first.

The primary competitive problem is authoring and operating experience, not an assumption that Apache Hop's processing engine is slow. The goal is not to reproduce every Hop screen or plugin. The goal is to be:

1. **easier** — a guided path from data source to a validated, runnable pipeline;
2. **consistently fast** — preserve or improve on Hop-class processing performance with a bounded-memory native engine and reproducible benchmarks;
3. **safer** — least-privilege execution, explicit secrets, and secure defaults;
4. **easier to deploy** — signed desktop packages, a standalone CLI, and a small container image;
5. **migration-friendly** — Apache Hop projects can be assessed and imported with an explicit compatibility report.

Apache Hop compatibility is a migration boundary, not Ajisai's internal architecture. Ajisai's native format, engine, UI, and plugin contract must remain useful without Apache Hop.

Airbyte is also a direct competitor for source/destination onboarding and operational data movement. Its guided connection UI is a design benchmark. User-observed bugs are treated as reliability hypotheses to reproduce and turn into Ajisai regression tests, not as an unqualified public claim about Airbyte.

## 2. Reset rules

- No existing implementation is considered complete merely because it exists in the repository.
- Existing Rust crates, transforms, parsers, and Electron UI are **reuse candidates**. Phase 0 decides whether each part is retained, rewritten, or removed.
- No performance, memory, compatibility, packaging, or security claim is published without a named fixture, environment, command, and retained result.
- A parsed `.hpl` file is not called compatible until its behavior is checked against the pinned Apache Hop oracle.
- A desktop build is not called distributable until install, launch, update, uninstall, signing, and platform smoke tests pass.
- Security gates apply to every phase. A later hardening phase does not excuse unsafe defaults earlier.
- Checked boxes mean there is reviewable evidence in the repository or a linked external run. Plans and source inspection remain unchecked.

### Status legend

- `[ ]` not demonstrated
- `[x]` demonstrated with retained evidence
- `[L]` bounded local work
- `[H]` resource-intensive validation or benchmark
- `[E]` external action, service, certificate, hardware, or user study

No rebuild phase is complete at this baseline.

## 3. Users and primary jobs

### Primary users

- An analyst who needs to clean and join CSV, Excel, JSON, Parquet, or database data without writing a program.
- A data engineer who wants a reviewable pipeline definition for cron, CI, or a container.
- A team migrating a bounded set of Apache Hop pipelines and wanting a precise unsupported-feature report.
- An operator who needs reproducible configuration, observable runs, safe secret handling, and predictable resource use.

### Golden path

`Install -> Create or import -> Connect data -> Preview -> Fix guided validation -> Run locally -> Export a deployment recipe -> Observe and rerun`

The default UI must optimize this path. Advanced metadata, plugin, and administration functions must not compete with it for attention.

### Explicit non-goals for the first stable generation

- Full coverage of Apache Hop's roughly 400-plugin ecosystem.
- A distributed compute engine competing with Spark, Flink, or Beam.
- A hosted multi-tenant control plane.
- Real-time collaborative editing.
- Arbitrary in-process native plugins or unrestricted scripts.
- Lossless export back to every Apache Hop feature.

### Competitive position

| Product | Strength to respect | Ajisai opportunity | Boundary |
|---|---|---|---|
| Apache Hop 2.19.0 | Fast, mature execution; broad visual pipeline/workflow and metadata capabilities | Radically reduce navigation, concepts shown at once, configuration steps, and debugging distance while retaining competitive execution | Hop is the migration and execution oracle, not a UI template |
| Airbyte | Strong guided source -> destination -> connection setup and a large connector-oriented product surface | Combine similarly clear onboarding with richer visual transformation, local-first execution, and stronger state/error transparency | Ajisai is not attempting full connector-catalog or managed-cloud parity in the first stable generation |
| Ajisai | Native local execution, one definition for Studio and CLI, explicit resource/security policy | Prove that easy authoring and trustworthy operation can coexist with high performance | All advantages remain targets until measured |

Ajisai should borrow interaction principles, not screen layouts: progressive disclosure, immediate connection tests, useful defaults, a visible review step, and one obvious next action. It must improve on failure trust: the UI may never report success when the engine has not durably confirmed it.

## 4. Measurable product gates

These are targets, not current claims. The reference hardware, datasets, and raw results must be committed under `benchmarks/` before a gate can pass.

| Quality | Stable-generation gate |
|---|---|
| First success | At least 8 of 10 representative new users complete install and a useful three-step pipeline in 10 minutes without maintainer help. |
| Learnability | The five golden-path tasks have at least 90% completion and no repeated severity-1 usability failure in the beta study. |
| Configuration burden | A first CSV-to-database pipeline requires no more than five primary decisions after selecting source and destination; advanced options remain collapsed. |
| State trust | In UI/API fault-injection tests, stale success, duplicate execution, lost edits, and irrecoverable ambiguous run states are zero. |
| CLI startup | Cold `ajisai --help` and validation startup p50 <= 200 ms on each reference platform. |
| Studio startup | First usable editor p50 <= 2 s on each reference platform. |
| Streaming memory | A declared 10 GiB streaming fixture completes within a 512 MiB RSS ceiling unless the plan explicitly contains a blocking transform. |
| Hop comparison | Against pinned Apache Hop 2.19.0, Ajisai produces equivalent normalized results, reaches at least Hop's median throughput on 4 of 5 agreed local workloads, and has no unexplained regression greater than 10%. A 2x speedup is an optimization target, not a release claim. |
| Reliability | The golden corpus passes on Windows, macOS, and Linux; cancellation and injected-failure tests leave no silently partial committed output. |
| Security | No open critical/high finding in shipped code; threat model reviewed; secrets never serialized into pipeline files or normal logs. |
| Installation | Fresh-user install, launch, update, rollback, and uninstall smoke tests pass for every advertised artifact. |
| Compatibility | Every imported Hop fixture produces one of `supported`, `supported-with-difference`, or `unsupported`; silent field loss is zero. |

## 5. Target architecture

The exact crate and package names are finalized by ADRs in Phase 0, but the boundaries are mandatory.

```text
Ajisai Studio / CLI / automation API
                |
        versioned command API
                |
     validator + planner + runner
       /          |           \
native format  execution IR  run events
       |          |           |
connectors    transforms   observability
       \          |          /
       capability and policy layer
                |
       filesystem / network / secrets

Apache Hop files -> isolated importer -> compatibility report -> native format
Third-party code -> signed package -> capability grant -> sandboxed runtime
```

### Mandatory design constraints

- Versioned, documented Ajisai-native **pipeline and workflow** formats are the source of truth. Pipeline is the row-stream graph; workflow is the orchestration graph. Parsing, validation, planning, and execution are separate stages.
- The engine uses bounded queues and reports which transforms require buffering. Accidental unbounded collection is forbidden.
- Data has an explicit schema, null semantics, deterministic coercion rules, and stable error records.
- The UI is a client of the same public command API used by the CLI; it does not contain execution semantics.
- File, network, process, database, secret, and script access are capabilities that can be denied.
- Built-in and third-party extensions have different trust levels. Third-party extensions do not run as unrestricted native code by default.
- Run events, metrics, and errors use versioned structured records; human-readable logs are a rendering of those records.
- Pipeline saves are atomic and recoverable. Outputs declare commit behavior and partial-output policy.

## 6. Phase overview

| Phase | Outcome | Depends on | Milestone |
|---|---|---|---|
| 0 | Reset, evidence baseline, and architecture decisions | — | Rebuild charter |
| 1 | Native specification and trusted foundation | 0 | Executable contracts |
| 2 | Minimal end-to-end engine and CLI | 1 | Headless preview |
| 3 | Beginner-first Studio vertical slice | 2 | Usable alpha |
| 4 | Production data plane and core transforms | 2 | Practical alpha |
| 5 | Projects, environments, workflows, and operations | 2, 4 | Deployable alpha |
| 6 | Apache Hop migration boundary | 1, 4, 5 | Migration beta |
| 7 | Security isolation and extension SDK | 1, 4 | Extension beta |
| 8 | Performance and memory program | 2, 4, 6 | Measured beta |
| 9 | Reliability, observability, and recovery | 4, 5 | Operations beta |
| 10 | Packaging and low-friction delivery | 3, 7, 9 | Release candidate |
| 11 | Cross-platform beta and adoption validation | 6–10 | Stable candidate |
| 12 | Stable release and maintenance system | 11 | 1.0-quality gate |

Phases show dependency order, not a promise that only one workstream runs at a time. Security, accessibility, documentation, and measurement are continuous gates.

## 7. Detailed phases

### Phase 0 — Reset and evidence baseline

**Outcome:** one product definition, one evidence model, and explicit decisions about what survives the rewrite.

#### 0.1 Repository and claim audit

- [x] `[L]` Inventory every crate, Transform, parser, UI path, CI job, fixture, and release artifact. Evidence: [`docs/rebuild/current-state.md`](docs/rebuild/current-state.md).
- [x] `[L]` Map each README claim to `verified`, `partially verified`, `unverified`, or `contradicted` evidence. Evidence: [`docs/rebuild/current-state.md`](docs/rebuild/current-state.md).
- [x] `[L]` Record current version sources and resolve the Rust workspace/Electron version split through an ADR; do not change versions as part of the audit. Evidence: [`docs/rebuild/current-state.md`](docs/rebuild/current-state.md) and [`docs/adr/0002-evidence-and-version-policy.md`](docs/adr/0002-evidence-and-version-policy.md).
- [x] `[L]` Classify existing code as `retain`, `wrap temporarily`, `rewrite`, or `remove`, with a reason and owner. Evidence: [`docs/rebuild/current-state.md`](docs/rebuild/current-state.md).
- [x] `[L]` Freeze representative current fixtures so rewrite regressions can be detected. Evidence: [`tests/fixtures/manifest.json`](tests/fixtures/manifest.json) and [`scripts/verify-fixtures.py`](scripts/verify-fixtures.py) verify 12 immutable definitions/oracles; generated outputs are explicitly mutable.

#### 0.2 User and workflow discovery

- [ ] `[E]` Interview at least five users across analyst, data-engineer, migration, and operator roles.
- [ ] `[E]` Observe three users completing equivalent work in Apache Hop; record friction without copying its information architecture.
- [ ] `[E]` Walk through Airbyte's source, destination, connection, schema, sync, run-status, and error-recovery flows; record what reduces cognitive load.
- [ ] `[L]` Convert known user-observed Airbyte failures into anonymized hypotheses with exact trigger, expected state, actual state, and recovery path; do not record a competitor defect as fact until reproduced against a pinned version.
- [x] `[L]` Select five golden workflows and three deliberately unsupported workflows. Evidence: [`docs/product/golden-workflows.md`](docs/product/golden-workflows.md).
- [ ] `[L]` Define beginner and advanced modes from observed needs, not menu count.
- [ ] `[L]` Validate low-fidelity interaction prototypes before committing to the Studio shell or full engine integration.

#### 0.3 Architecture and product ADRs

- [x] `[L]` ADR: native document format and schema-version evolution. Evidence: [`docs/adr/0003-native-schema-evolution.md`](docs/adr/0003-native-schema-evolution.md).
- [x] `[L]` ADR: row versus columnar internal representation, including adapter cost. Evidence: [`docs/adr/0004-row-columnar-boundary.md`](docs/adr/0004-row-columnar-boundary.md).
- [x] `[L]` ADR: scheduler, backpressure, blocking transforms, cancellation, and resource budgets. Evidence: [`docs/adr/0005-scheduler-resource-policy.md`](docs/adr/0005-scheduler-resource-policy.md).
- [x] `[L]` ADR: Studio shell selection using measured startup, size, accessibility, updater, and sandbox criteria. Evidence: [`docs/adr/0006-studio-shell.md`](docs/adr/0006-studio-shell.md); final measurement gate remains open.
- [x] `[L]` ADR: extension sandbox and capability model. Evidence: [`docs/adr/0007-extension-capabilities.md`](docs/adr/0007-extension-capabilities.md).
- [ ] `[L]` ADR: supported platforms, minimum OS versions, and distribution channels.

#### 0.4 Baseline measurements and threat model

- [ ] `[H]` Pin reference hardware, toolchains, Apache Hop 2.19.0, and benchmark datasets.
- [ ] `[H]` Measure current Ajisai and Hop startup, elapsed time, throughput, peak RSS, and output hashes.
- [x] `[L]` Threat-model untrusted pipeline files, path traversal, SSRF, SQL, spreadsheets, XML, archives, scripts, plugins, secrets, updates, and IPC. Evidence: [`docs/release/risk-register.md`](docs/release/risk-register.md).
- [x] `[L]` Create the risk register and define release-blocking severity rules. Evidence: [`docs/release/risk-register.md`](docs/release/risk-register.md); independent security review remains open.

**Exit gate:** ADRs are accepted, claims have evidence states, the compatibility and benchmark corpora are immutable/versioned, and every retained component has a justification.

### Phase 1 — Native specification and trusted foundation

**Outcome:** Ajisai files can be parsed, upgraded, validated, and explained without running them.

#### 1.1 Versioned native format

- [x] `[L]` Specify the initial pipeline/workflow document contract, retaining Hop-compatible responsibilities and nesting. Evidence: [`crates/core/src/native_format.rs`](crates/core/src/native_format.rs), [`tests/fixtures/minimal.ajp`](tests/fixtures/minimal.ajp), and [`tests/fixtures/minimal.ajw`](tests/fixtures/minimal.ajw). Project, connection, parameter, and policy documents remain open.
- [x] `[L]` Publish JSON Schema or an equivalent machine-readable schema with examples. Evidence: [`spec/ajisai.pipeline.schema.json`](spec/ajisai.pipeline.schema.json), [`spec/ajisai.workflow.schema.json`](spec/ajisai.workflow.schema.json), and native fixtures under [`tests/fixtures`](tests/fixtures).
- [x] `[L]` Preserve unknown fields in the initial pipeline document round trip and reject unsupported format versions clearly. Evidence: Native Document tests in [`crates/core/src/native_format.rs`](crates/core/src/native_format.rs); workflow round-trip coverage remains open.
- [ ] `[L]` Add deterministic formatting, stable IDs, semantic diff fixtures, and migrations between schema versions. Canonical JSON, stable ID validation, structural JSON-path diff, explicit v1 migration boundary, and CLI rewrite path are now available and tested; future-version migration registry and retained diff fixtures remain open.
- [ ] `[L]` Separate secret references from values and forbid inline secrets by default. ConnectionMeta serialization omits passwords, and Native config validation rejects common inline secret keys; secret-provider references and broader config coverage remain open.

#### 1.2 Type and transform contracts

- [ ] `[L]` Define scalar, decimal, temporal, binary, list/struct, and null behavior.
- [ ] `[L]` Define schema propagation, coercion, locale/time-zone behavior, and error-row contracts.
- [x] `[L]` Give each Transform a versioned manifest: inputs, outputs, config schema, buffering class, capabilities, and determinism. Initial `list-transforms --json` contract publishes manifest version, deterministic flag, and capability/buffering placeholders; detailed per-transform schemas remain open. Evidence: [`crates/cli/src/commands/list.rs`](crates/cli/src/commands/list.rs).
- [ ] `[L]` Generate CLI help and Studio forms from the same manifest.

#### 1.3 Validator and planner

- [x] `[L]` Validate graph structure and required fields (initial structural subset). Evidence is implemented in [`crates/core/src/native_format.rs`](crates/core/src/native_format.rs); types, resource references, capabilities, and output conflicts remain open.
- [x] `[L]` Return stable diagnostic codes with exact node/field locations and suggested fixes. Evidence: Native Pipeline/Workflow validator tests in [`crates/core/src/native_format.rs`](crates/core/src/native_format.rs).
- [ ] `[L]` Produce an explainable execution plan without touching external systems.
- [x] `[L]` Add golden diagnostics and property tests for malformed and adversarial documents. Current evidence covers malformed names, duplicate IDs, missing references, cycles, and workflow outcome types; broader adversarial coverage remains open.

**Exit gate:** the native corpus round-trips deterministically, every invalid fixture produces the expected code and location, and validation performs no undeclared I/O.

### Phase 2 — Minimal end-to-end engine and CLI

**Outcome:** one production-shaped pipeline runs correctly through the new contracts.

#### 2.1 Bounded execution kernel

- [x] `[L]` Stream CSV source rows directly through the bounded channel instead of accumulating a `Vec<Row>`; explicit type conversion and downstream disconnect are covered by unit tests. Evidence: [`crates/transforms/src/csv/input.rs`](crates/transforms/src/csv/input.rs); global lifecycle, memory budgets, and cancellation remain open.
- [ ] `[L]` Implement lifecycle states: plan, open, run, drain, commit, cancel, fail, close.
- [ ] `[L]` Enforce bounded channels and per-operator memory budgets.
- [x] `[L]` Propagate fan-out/downstream disconnect errors so a source cannot report success after a closed route. Evidence: [`crates/core/src/engine.rs`](crates/core/src/engine.rs) and CSV disconnect test; full cancellation and merge semantics remain open.
- [x] `[L]` Populate execution row counters at source and sink boundaries for truthful run statistics. Evidence: [`crates/core/src/engine.rs`](crates/core/src/engine.rs); per-transform metrics and persisted run events remain open.
- [x] `[L]` Add cooperative cancellation to the shared execution context and check it at node/source/row boundaries. Evidence: [`crates/core/src/context.rs`](crates/core/src/context.rs), [`crates/core/src/engine.rs`](crates/core/src/engine.rs), and CLI SIGINT integration coverage; broader connector-specific cancellation fixtures remain open.
- [x] `[L]` Wire CLI Ctrl-C to the shared cancellation context and return a deterministic cancellation error. Evidence: [`crates/cli/src/commands/run.rs`](crates/cli/src/commands/run.rs) and [`crates/cli/tests/signal_cancel.rs`](crates/cli/tests/signal_cancel.rs); SIGINT is verified at process level with exit code 130.
- [x] `[L]` Add a deterministic CLI cancellation contract test with an injected signal future, proving the shared context is cancelled and the user-facing error is stable. Evidence: [`crates/cli/src/commands/run.rs`](crates/cli/src/commands/run.rs) and [`crates/cli/tests/signal_cancel.rs`](crates/cli/tests/signal_cancel.rs).
- [x] `[L]` Make non-append CSV output atomic by committing a same-directory temporary file only during successful close. Evidence: [`crates/transforms/src/csv/output.rs`](crates/transforms/src/csv/output.rs); crash-recovery cleanup and atomic semantics for every sink remain open.
- [x] `[L]` Remove an uncommitted CSV temporary file during sink drop after failure or cancellation. Evidence: [`crates/transforms/src/csv/output.rs`](crates/transforms/src/csv/output.rs); crash-recovery cleanup for every sink remains open.
- [x] `[L]` Add a version-fixed candidate checklist separating reproduced evidence from open release gates. Evidence: [`docs/release/candidate-checklist.md`](docs/release/candidate-checklist.md).
- [x] `[L]` Execute the workspace-wide test suite and record its evidence in the candidate checklist. Evidence: [`docs/release/candidate-checklist.md`](docs/release/candidate-checklist.md); release optimization/linking remains environment-limited.
- [x] `[L]` Upgrade Hop XML parsing to quick-xml 0.41 and adapt text decoding APIs, removing the audited 0.37 parser path. Evidence: [`crates/hop-compat/Cargo.toml`](crates/hop-compat/Cargo.toml) and parser compatibility build; broader independent dependency review remains open.
- [x] `[L]` Check in explicit cargo-deny license/source policy for reproducible audits. Evidence: [`deny.toml`](deny.toml); unresolved advisories and policy exceptions remain release blockers.
- [x] `[L]` Pass cargo-deny license and source checks with the checked-in policy. Evidence: `cargo deny check licenses sources` -> `licenses ok, sources ok`.
- [x] `[L]` Record the dated security audit, remediations, and non-waived blockers. Evidence: [`docs/release/security-audit-2026-09-05.md`](docs/release/security-audit-2026-09-05.md).
- [x] `[L]` Add a Hop XML writer/parser round-trip contract test covering transform metadata and hops. Evidence: [`crates/hop-compat/src/parser/writer.rs`](crates/hop-compat/src/parser/writer.rs).
- [x] `[L]` Parse the repository's `sample.hpl` and `showcase.hpl` fixtures as compatibility smoke tests. Evidence: [`crates/hop-compat/src/parser/writer.rs`](crates/hop-compat/src/parser/writer.rs).
- [x] `[L]` Execute a Hop fixture through the Ajisai engine into an isolated temporary CSV and compare a retained oracle. Evidence: [`tests/fixtures/sample-candidate.hpl`](tests/fixtures/sample-candidate.hpl) and [`tests/fixtures/sample-candidate.expected.csv`](tests/fixtures/sample-candidate.expected.csv).
- [x] `[L]` Preserve Hop error-hop semantics in Native Pipeline edges and Native CLI/runner construction. Evidence: [`crates/core/src/model.rs`](crates/core/src/model.rs), [`crates/cli/src/commands/run.rs`](crates/cli/src/commands/run.rs), and [`crates/core/src/runner.rs`](crates/core/src/runner.rs).
- [x] `[L]` Verify error-row routing end-to-end with an isolated Native error-hop fixture and retained CSV oracle. Evidence: [`tests/fixtures/error-hop.ajp`](tests/fixtures/error-hop.ajp) and [`tests/fixtures/error-hop.expected.csv`](tests/fixtures/error-hop.expected.csv).
- [x] `[L]` Add Native error-edge serialization round-trip coverage so Hop error hops cannot be silently dropped. Evidence: [`crates/core/src/native_format.rs`](crates/core/src/native_format.rs).
- [x] `[L]` Guarantee transform `close()` runs after both successful and failed node execution, preserving sink flush/cleanup on errors. Evidence: [`crates/core/src/engine.rs`](crates/core/src/engine.rs); panic isolation and cancellation-specific close tests remain open.
- [x] `[L]` Abort all remaining node tasks after the first node error or panic, preventing orphaned execution after a failed run. Evidence: [`crates/core/src/engine.rs`](crates/core/src/engine.rs); panic-specific fixture and cancellation close tests remain open.
- [x] `[L]` Add a panic-isolation regression test proving a panicking source becomes a deterministic pipeline error. Evidence: [`crates/core/src/engine.rs`](crates/core/src/engine.rs).
- [x] `[L]` Add a sink cleanup regression test proving uncommitted CSV temporary output is removed on drop. Evidence: [`crates/transforms/src/csv/output.rs`](crates/transforms/src/csv/output.rs).
- [ ] `[L]` Make partial-output and atomic-commit behavior explicit for every sink. CSV, JSON, XML, Parquet, and Excel file sinks now use same-directory temporary files, atomic rename, and cleanup; database transactions are atomic when `batch_size=0`, while batch/append semantics remain open.
- [x] `[L]` Emit versioned structured run/node events with stable node IDs and an execution-context snapshot API. Evidence: [`crates/core/src/context.rs`](crates/core/src/context.rs), [`crates/core/src/engine.rs`](crates/core/src/engine.rs), and `emits_versioned_run_and_node_events` regression test; node failure and run terminal events are covered, while richer resource/retry fields remain open.

#### 2.2 Golden vertical slice

- [x] `[L]` Implement an initial Native-format `GenerateRows -> CsvFileOutput` slice with environment-variable resolution. Evidence: [`tests/fixtures/minimal-run.ajp`](tests/fixtures/minimal-run.ajp) and CLI smoke output; the full CSV input -> select/cast -> filter -> CSV slice remains open.
- [x] `[L]` Implement CSV input -> select/cast -> filter -> CSV output. Evidence: [`tests/fixtures/full-csv.ajp`](tests/fixtures/full-csv.ajp) and [`tests/fixtures/full-csv.expected.csv`](tests/fixtures/full-csv.expected.csv); CLI execution matched the expected output.
- [x] `[L]` Cover quoted fields, invalid encoding, missing columns, nulls, type errors, and cancellation. Quoted delimiters, empty-value null mapping, deterministic invalid-type-to-null coercion, unsupported-encoding rejection, missing-column parse errors, and OS-level SIGINT cancellation are covered by CSV/CLI tests; broader malformed-input and connector-specific cancellation fixtures remain open.
- [x] `[L]` Verify the initial CSV slice against a retained expected-output oracle. Evidence: `diff -u tests/fixtures/full-csv.expected.csv /private/tmp/ajisai-full-csv.csv`; independent/normalized oracles for broader fixtures remain open.
- [ ] `[H]` Run memory-bound and injected-failure tests on the large fixture. Bounded side-input and generated-source row policies plus injected Transform failure termination are tested (`rejects_side_input_when_buffer_policy_is_exceeded`, `large_source_stops_at_row_budget_without_unbounded_buffering`, `injected_failure_stops_large_source_and_records_terminal_failure`); large-fixture RSS, disk-pressure, and connector-specific failure evidence remain open.

#### 2.3 CLI contract

- [x] `[L]` Add Native Pipeline/Workflow `.ajp`/`.ajw` validation and Native Pipeline execution paths to the existing CLI. Evidence: [`crates/cli/src/commands/validate.rs`](crates/cli/src/commands/validate.rs), [`crates/cli/src/commands/run.rs`](crates/cli/src/commands/run.rs), and the `minimal.ajp`, `minimal.ajw`, and `minimal-run.ajp` smoke fixtures.
- [x] `[L]` Add side-effect-free `explain` output for native Pipeline/Workflow structure. Evidence: [`crates/cli/src/commands/explain.rs`](crates/cli/src/commands/explain.rs).
- [x] `[L]` Add stable JSON output to `validate` for Hop and Native Pipeline/Workflow documents. Evidence: [`crates/cli/src/commands/validate.rs`](crates/cli/src/commands/validate.rs); JSON schema versioning and broader CLI commands remain open.
- [x] `[L]` Add side-effect-free `inspect` output for native document metadata, graph counts, and node/action schema summaries. Evidence: [`crates/cli/src/commands/inspect.rs`](crates/cli/src/commands/inspect.rs); preview execution remains open.
- [x] `[L]` Surface source/sink row statistics in CLI success output. Evidence: [`crates/cli/src/commands/run.rs`](crates/cli/src/commands/run.rs); richer per-node resource metrics remain open.
- [x] `[L]` Add bounded Native CSV source preview that emits JSON rows and inferred schema without invoking downstream sinks. Evidence: [`crates/cli/src/commands/preview.rs`](crates/cli/src/commands/preview.rs); multi-source/non-CSV preview remains open.
- [ ] `[L]` Complete the CLI contract with `new`, `preview`, `inspect`, stable JSON output, resource limits, and shell completions; `new` generates a valid canonical Native Pipeline, `run` and `run-workflow --json` emit versioned success/dry-run/failure envelopes with records, `run-workflow --dry-run` validates action graphs and compatibility without execution, and `completions <shell>` is generated from Clap. Full workflow UX remains open.
- [x] `[L]` Define stable CLI exit codes for not-found (2), validation/unsupported input (3), cancellation (130), and runtime failure (1); existing `validate --json` remains the first stable JSON automation contract. Evidence: [`crates/cli/src/main.rs`](crates/cli/src/main.rs) and [`crates/cli/src/commands/validate.rs`](crates/cli/src/commands/validate.rs).
- [ ] `[L]` Add dry-run, resource limits, parameter overrides, and a no-network policy flag. `run --dry-run --json` now builds the registry-backed plan (including unknown-transform rejection) without side effects; cooperative `run --max-rows`, `run --max-buffered-rows`, `run --timeout-secs`, and `run --no-network` gate input, blocking side-input buffering, time, REST, and remote database initialization; byte-level memory budgets and broader connector coverage remain open.
- [x] `[L]` Add shell completion and actionable error rendering in Japanese and English. Evidence: Clap-generated completions plus `--lang ja|en` localized not-found, unsupported-format, and invalid-environment diagnostics.

**Exit gate:** the vertical slice passes correctness, bounded-memory, failure, and CLI contract tests on all three operating systems.

### Phase 3 — Beginner-first Ajisai Studio

**Outcome:** a first-time user can build and understand the golden vertical slice without reading a manual.

#### 3.1 Information architecture

- [ ] `[L]` Center the UI on `Source -> Shape -> Destination`, recent projects, and templates.
- [ ] `[L]` Hide environment, plugin, and execution internals until relevant.
- [ ] `[L]` Provide command search and contextual actions instead of expanding permanent menus.
- [ ] `[L]` Keep `Pipeline` and `Workflow` as explicit concepts in the document model and advanced view, while the beginner path presents the smallest next action.
- [ ] `[L]` Keep one obvious primary action per step and show a review summary before the first run.
- [ ] `[L]` Define consistent Japanese and English terminology and translation keys.

#### 3.2 Guided authoring

- [ ] `[L]` Add a source-first wizard that infers schema from a bounded sample.
- [ ] `[L]` Test source and destination connectivity before save and keep credentials out of the rendered configuration.
- [ ] `[L]` Render forms from Transform manifests with examples, safe defaults, and inline diagnostics.
- [ ] `[L]` Preview data at any node with provenance, sampling limits, and sensitive-value masking.
- [ ] `[L]` Show schema changes and broken downstream references before run.
- [ ] `[L]` Support undo/redo, autosave, crash recovery, and conflict-safe atomic saves.

#### 3.3 Accessible canvas and debugging

- [ ] `[L]` Provide keyboard-complete graph editing and a non-canvas list/outline view.
- [ ] `[L]` Meet WCAG 2.2 AA for contrast, focus, labels, zoom, and reduced motion.
- [ ] `[L]` Display row counts, duration, errors, and sample lineage without overwhelming the default view.
- [ ] `[L]` Add deterministic component, accessibility, and end-to-end tests.
- [ ] `[E]` Run the first-task study and retain anonymized task outcomes.

#### 3.4 Trustworthy UI state

- [ ] `[L]` Make persisted backend state authoritative; optimistic UI state is visibly pending until acknowledged.
- [ ] `[L]` Give save, validate, preview, run, cancel, and retry requests stable IDs and idempotency behavior.
- [ ] `[L]` Test reload, sidecar restart, duplicate response, delayed response, dropped notification, and version-conflict scenarios.
- [ ] `[L]` Preserve the last confirmed state and offer a concrete retry/recovery action after every failed mutation.
- [ ] `[L]` Never collapse `failed`, `cancelled`, `partially committed`, `unknown`, or `stale` into a generic completed state.

**Exit gate:** the three-step pipeline passes end-to-end UI tests and the first alpha usability gate; fault injection produces no false success or lost confirmed state; the UI has no privileged execution logic.

### Phase 4 — Production data plane and core transforms

**Outcome:** Ajisai handles the common local ETL workload with explicit semantics and resource behavior.

#### 4.1 Tier-1 connectors

- [ ] `[L]` CSV, JSON/JSONL, Parquet, Excel, and local filesystem connectors.
- [ ] `[L]` SQLite and PostgreSQL source/sink with parameterized queries and transaction policy. SQLite commit-at-close, remote-network denial, and SQLx 0.9 dynamic-query audit wrappers are tested; MySQL is isolated behind an explicit opt-in feature pending its own advisory/threat-model review.
- [ ] `[L]` HTTP source with timeouts, size limits, redirect policy, TLS validation, and host allowlists.
- [ ] `[L]` For each connector, add malformed-input, cancellation, retry, partial-write, and round-trip fixtures.

#### 4.2 Tier-1 transforms

- [ ] `[L]` Select/rename/cast, filter, derive, string/date operations, null handling, deduplicate, sort, aggregate, join, union, and lookup.
- [ ] `[L]` Specify ordering, equality, floating-point, decimal, collation, and null semantics per Transform.
- [ ] `[L]` Label transforms `streaming`, `partition-buffered`, or `fully-buffered` in CLI and Studio.
- [ ] `[L]` Add differential/property tests and retained output hashes for every Transform.

#### 4.3 Resource-aware execution

- [ ] `[L]` Spill sort, aggregate, and joins to bounded temporary storage.
- [ ] `[L]` Expose concurrency, memory, disk, timeout, and row limits in policy.
- [ ] `[L]` Fail before execution when the plan cannot honor a hard resource policy.
- [ ] `[H]` Exercise low-memory, full-disk, slow-source, and interrupted-sink scenarios.

**Exit gate:** the five golden workflows are correct, bounded, cancellable, and diagnosable; Transform count is not used as a quality proxy.

### Phase 5 — Projects, environments, workflows, and operations

**Outcome:** a pipeline moves from a laptop to scheduled execution without manual reconstruction.

#### 5.1 Projects and environments

- [ ] `[L]` Define portable project roots, relative paths, environment overlays, parameters, and connection references. `run --project-root PATH` and `run-workflow --project-root PATH` now propagate an explicit root through pipeline actions; overlays, parameters, and connection references remain open.
- [ ] `[L]` Validate development/test/production overlays and show an effective-config diff.
- [ ] `[L]` Add OS keychain and environment/file secret providers; keep provider APIs replaceable.
- [ ] `[L]` Export a redacted, deterministic support bundle.

#### 5.2 Workflow orchestration

- [ ] `[L]` Add sequential/parallel pipeline tasks, conditions, retry with backoff, timeout, and failure branches.
- [ ] `[L]` Define run identity, idempotency guidance, and artifact passing.
- [x] `[L]` Provide local schedules as generated OS recipes, not a hidden always-on scheduler. Evidence: [`docs/release/scheduling.md`](docs/release/scheduling.md).
- [ ] `[L]` Add workflow simulation and failure-injection fixtures.

#### 5.3 Deployment contract

- [ ] `[L]` Generate a lockfile containing format, Transform, connector, and extension versions.
- [x] `[L]` Add `ajisai doctor` for capabilities, paths, network, resource-policy, config, and cache checks. Evidence: [`crates/cli/src/commands/doctor.rs`](crates/cli/src/commands/doctor.rs); secret-provider and deeper capability probes remain open.
- [x] `[L]` Generate reproducible cron, CI, and container examples from the same project. Evidence: [`docs/release/scheduling.md`](docs/release/scheduling.md); digest pinning and hosted-environment execution evidence remain release-specific.

**Exit gate:** one project runs unchanged in development and test using only explicit overlay/secret differences, and its deployment recipe is reproducible.

### Phase 6 — Apache Hop migration boundary

**Outcome:** users can make an informed migration decision without silent semantic loss.

The comparison oracle is Apache Hop **2.19.0** until an ADR intentionally changes it. Official Hop documentation describes visual pipelines/workflows, projects and environments, multiple run configurations, and a plugin-oriented architecture; Ajisai must map these concepts explicitly rather than treating XML parsing as compatibility.

#### 6.1 Inventory and assessment

- [x] `[L]` Scan a Hop project without executing it and inventory pipelines, workflows, metadata, plugins, variables, and external dependencies. Evidence: [`crates/hop-compat/src/project_scan.rs`](crates/hop-compat/src/project_scan.rs), `ajisai-cli scan --json`; pipelines and workflows are enumerated, while symlinks and files over 10 MiB are reported without execution.
- [x] `[L]` Produce machine-readable and human-readable compatibility reports. Evidence: [`crates/hop-compat/src/compatibility.rs`](crates/hop-compat/src/compatibility.rs), `ajisai-cli assess --json`, and the pinned `sample.hpl` fixture.
- [x] `[L]` Classify every item as `supported`, `supported-with-difference`, or `unsupported`, with a stable reason code. Evidence: `TRANSFORM_NATIVE`, `TRANSFORM_RENAMED`, `TRANSFORM_UNSUPPORTED`, and workflow `WORKFLOW_ACTION_NATIVE` / `WORKFLOW_ACTION_AS_PIPELINE` classifications plus regression coverage in [`crates/hop-compat/src/compatibility.rs`](crates/hop-compat/src/compatibility.rs).
- [x] `[L]` Reject active content, unsafe paths, entity expansion, and oversized documents during assessment. Project scan rejects symlinks, DTD/ENTITY/script-style XML markers, and files over 10 MiB before parsing; unsafe XML regression coverage is retained in [`crates/hop-compat/src/project_scan.rs`](crates/hop-compat/src/project_scan.rs).

#### 6.2 Importer

- [ ] `[L]` Convert only explicitly supported constructs into the native format.
- [ ] `[L]` Preserve unmapped source fragments in a quarantine attachment for review; never execute them.
- [ ] `[L]` Map projects, environments, parameters, connections, run configurations, pipelines, and workflows through versioned adapters.
- [ ] `[L]` Provide a guided manual replacement for common unsupported transforms.

#### 6.3 Differential compatibility

- [ ] `[H]` Run the public fixture corpus in pinned Hop and Ajisai under controlled inputs.
- [ ] `[H]` Compare normalized rows, schemas, errors, side effects, and relevant ordering.
- [ ] `[L]` Publish the tested compatibility matrix with exact fixture counts and known differences.
- [ ] `[E]` Validate at least three real projects with owner permission and redacted results.

**Exit gate:** no silent field or behavior loss in the corpus, all differences are reported before conversion, and real-project migration results are documented without claiming full Hop compatibility.

### Phase 7 — Security isolation and extension SDK

**Outcome:** risky capability is explicit, reviewable, and revocable.

#### 7.1 Policy enforcement

- [ ] `[L]` Enforce project-root filesystem policy with symlink and traversal tests. Shared path validation now rejects parent components and symlink components; `run --project-root PATH` configures an explicit canonical root and all current filesystem transforms, including row-derived `LoadFileContent`, enforce it at open/close/process boundaries with regression coverage. Connector-wide policy audits and future extension APIs remain open. Evidence: [`crates/core/src/context.rs`](crates/core/src/context.rs), [`crates/transforms/src/utils.rs`](crates/transforms/src/utils.rs).
- [ ] `[L]` Enforce network host/port/TLS policy and block local/cloud metadata SSRF by default. REST client validates HTTP(S), rejects URL credentials, private/loopback/link-local/multicast literal IPs and common metadata targets, resolves hostnames before requests to reject blocked IP ranges, and supports an explicit `allowed_hosts` list; TLS pinning remains open.
- [ ] `[L]` Enforce subprocess, database, secret, temporary-storage, and output policies.
- [ ] `[L]` Redact structured events, diagnostics, previews, crash reports, and support bundles. Connection metadata password serialization is redacted; broader event/config redaction remains open.

#### 7.2 Safe scripting and plugins

- [ ] `[L]` Replace unrestricted scripting with a resource-limited sandbox or omit it from stable. Rhai ScriptStep is opt-in via the `scripting` feature and excluded from the default candidate graph; sandboxing and a stable support decision remain open.
- [ ] `[L]` Define a versioned SDK and capability manifest with explicit host calls.
- [ ] `[L]` Run third-party extensions out of process or in a sandboxed runtime; deny native dynamic libraries by default.
- [ ] `[L]` Require package hashes/signatures, provenance, compatibility ranges, and permission review.
- [ ] `[L]` Provide kill, timeout, memory, output-size, and crash isolation.

#### 7.3 Secure development lifecycle

- [ ] `[L]` Add dependency/license policy, SBOM generation, secret scanning, static analysis, and unsafe-code policy. cargo-deny license/source/advisory checks, a dedicated CI dependency-audit job, strict workspace clippy, local unsafe-code enforcement, conservative CI secret scanning, and deterministic CycloneDX SBOM generation now pass; the yanked SQLx `spin` warning is resolved in `spin 0.9.9`, while broader history/entropy scanning and third-party unsafe-code auditing remain open.
- [ ] `[H]` Fuzz native format, Hop import, CSV/JSON/XML/Excel parsing, IPC/API, and extension boundaries.
- [ ] `[E]` Complete an independent security review and resolve all release-blocking findings.
- [x] `[L]` Publish vulnerability reporting and supported-version policies. Evidence: [`SECURITY.md`](SECURITY.md) and [`docs/release/support-policy.md`](docs/release/support-policy.md).

**Exit gate:** the threat-model abuse cases pass, extensions cannot exceed granted capabilities, and no open critical/high finding remains.

### Phase 8 — Performance and memory program

**Outcome:** performance claims are reproducible and do not trade away correctness or safety.

#### 8.1 Benchmark discipline

- [ ] `[L]` Version dataset generators, hashes, expected results, commands, hardware descriptors, and report schema. Evidence: [`benchmarks/manifest.json`](benchmarks/manifest.json) pins the first CSV fixture and expected hash; [`scripts/run-benchmark.py`](scripts/run-benchmark.py) emits a diagnostic JSON report. Reference hardware, generators, and Hop runs remain open.
- [ ] `[L]` Separate cold startup, warm startup, throughput, latency, peak RSS, disk spill, and output commit time. `scripts/run-benchmark.py` now builds once (or reuses a pinned binary with `--skip-build`) and labels direct-binary iterations as cold/warm while retaining wall time, output bytes, normalized cumulative child RSS, and output hashes; per-process RSS, disk spill, and commit-time instrumentation remain open.
- [ ] `[L]` Include narrow, wide, null-heavy, skewed, malformed, join, aggregate, and mixed-I/O workloads.
- [ ] `[L]` Reject benchmark runs with unequal semantics, caches, compression, output durability, or data.

#### 8.2 Engine optimization

- [ ] `[H]` Profile allocation, row cloning, serialization, scheduling, and blocking operators before changing them.
- [ ] `[L]` Implement batch/columnar paths only where the ADR and measurements justify them.
- [ ] `[L]` Add adaptive batching without violating cancellation or memory limits.
- [ ] `[L]` Make concurrency explicit and prevent async/runtime oversubscription.
- [ ] `[H]` Add regression thresholds with variance handling; do not gate on single-run timing.

#### 8.3 Competitive report

- [ ] `[H]` Run the pinned Ajisai/Hop matrix on clean reference hosts.
- [ ] `[L]` Publish median, spread, raw results, failures, and limitations for every workload.
- [ ] `[L]` Remove or qualify any public claim that the evidence does not support.

**Exit gate:** the performance product gates pass on all reference platforms, correctness hashes match, and reports can be reproduced from a clean checkout.

### Phase 9 — Reliability, observability, and recovery

**Outcome:** failures are understandable, bounded, and recoverable.

#### 9.1 Run records and diagnostics

- [ ] `[L]` Persist structured run, node, row-count, timing, resource, retry, and error events. `run --record PATH` and `run-workflow --record PATH` now atomically persist versioned success, dry-run, failure, and workflow-action summaries (including structured events on pipeline success); workflow action-failure details are preserved from CLI fallback handling, while resource/retry fields remain open.
- [ ] `[L]` Correlate UI, CLI, and engine records with one run ID while redacting secrets. CLI records now carry a process/time-derived `run_id`; UI correlation and stronger cross-process identity remain open.
- [ ] `[L]` Export OpenTelemetry-compatible traces/metrics behind an explicit opt-in.
- [ ] `[L]` Provide bottleneck explanations based on retained measurements, not guesses.
- [ ] `[L]` Reconcile Studio state from run records after reconnect instead of trusting missed in-memory notifications.

#### 9.2 Recovery semantics

- [ ] `[L]` Define restartability and checkpoint eligibility per connector and Transform.
- [ ] `[L]` Detect abandoned runs and incomplete outputs; never present them as success.
- [ ] `[L]` Support safe retry/resume where idempotency is demonstrable.
- [ ] `[H]` Test process kill, power-loss simulation, corrupt state, full disk, and dependency outage.

#### 9.3 Compatibility and migration stability

- [ ] `[L]` Test native-format upgrades across every supported version.
- [ ] `[L]` Test extension/API compatibility and intentional failure modes.
- [ ] `[L]` Document backup, rollback, and recovery before destructive migrations.

**Exit gate:** fault-injection scenarios produce the specified terminal state and recovery action, with no silently committed partial result.

### Phase 10 — Packaging and low-friction delivery

**Outcome:** installation is simpler than assembling a runtime and remains secure after installation.

#### 10.1 Artifacts

- [ ] `[L]` Build a standalone CLI for supported Windows, macOS, and Linux targets. Release workflow now builds and uploads `target/release/ajisai-cli*` for each OS alongside Studio artifacts; clean-machine install and runtime smoke evidence remain open.
- [ ] `[L]` Build Studio packages only for the minimum OS versions selected in Phase 0.
- [ ] `[L]` Build a minimal pinned container image with non-root defaults and a read-only-root option. Evidence: pinned Rust 1.97 multi-stage [`Dockerfile`](Dockerfile), non-root runtime, `.dockerignore`, read-only deployment guidance, and a release CI build/ping smoke job in [`docs/release/container.md`](docs/release/container.md); published digest and executed CI evidence remain open.
- [ ] `[L]` Embed license inventory, SBOM, source revision, and reproducible version information. Release workflow now archives locked Cargo metadata, CycloneDX SBOM, sorted SHA-256 checksums, and `ajisai-build-info.json` with source/toolchain provenance; signing and independent verification remain open.

#### 10.2 Trust and updates

- [ ] `[E]` Sign Windows artifacts and verify install/uninstall reputation behavior.
- [ ] `[E]` Sign and notarize macOS artifacts for both selected architectures.
- [ ] `[L]` Sign Linux artifacts and publish checksums and provenance.
- [ ] `[E]` Exercise staged update, signature failure, interrupted update, and rollback.
- [ ] `[L]` Keep automatic telemetry off by default and explain all outbound requests.

#### 10.3 Installation experience

- [ ] `[E]` Test fresh machines with no developer toolchain. Release matrix now smoke-tests each built standalone CLI binary with `--version` and `doctor --json`; fresh-machine install/uninstall evidence remains open.
- [ ] `[L]` Provide one quickstart project embedded in Studio and CLI packages.
- [ ] `[L]` Make `doctor`, logs, cache location, configuration location, and complete uninstall discoverable.
- [ ] `[H]` Record artifact size, install time, startup time, idle RSS, and update size per platform.

**Exit gate:** signed fresh-install through uninstall/update tests pass on each advertised platform with no Rust, Node, Java, or Python prerequisite.

### Phase 11 — Cross-platform beta and adoption validation

**Outcome:** the product works for people and projects outside the development machine.

#### 11.1 Closed beta

- [ ] `[E]` Recruit at least ten users spanning the four primary roles and three operating systems.
- [ ] `[E]` Run golden-path usability studies and collect task completion, time, failures, and support needs.
- [ ] `[E]` Migrate consenting Hop projects through the assessment-first flow.
- [ ] `[L]` Triage every beta failure into product, documentation, unsupported scope, or environment.

#### 11.2 Release-candidate gates

- [ ] `[H]` Run the full correctness, compatibility, fuzz, fault, performance, accessibility, and packaging matrices.
- [ ] `[L]` Complete translations and verify layout in Japanese and English.
- [ ] `[L]` Freeze native format/API changes except release blockers.
- [ ] `[L]` Publish accurate limitations, supported matrices, migration guidance, and benchmark methodology.

**Exit gate:** all measurable product gates pass, no release blocker is open, and at least three external projects complete useful work.

### Phase 12 — Stable release and maintenance system

**Outcome:** a supportable stable product, not merely a tagged build.

- [x] `[L]` Define support windows, deprecation periods, schema migrations, and extension compatibility policy. Evidence: [`docs/release/support-policy.md`](docs/release/support-policy.md).
- [ ] `[L]` Prepare signed release artifacts, checksums, SBOM, provenance, release notes, and rollback instructions. Workflow now archives checksums, SBOM, locked metadata, build provenance, independently verifies them, and signs `SHA256SUMS` with Sigstore keyless OIDC; tag-run signature verification and rollback rehearsal remain open.
- [ ] `[E]` Publish artifacts and verify every registry/download independently.
- [ ] `[E]` Verify updater paths only after published artifacts and signatures are confirmed.
- [ ] `[L]` Start crash-free, task-success, security-response, performance-regression, and installation-success scorecards.
- [ ] `[L]` Create the next roadmap from measured adoption and failure data rather than Transform count.

**Exit gate:** source, CI, signed artifacts, downloads, updater, docs, and support policy all independently verify the same release.

## 8. Cross-phase release gates

Every preview, alpha, beta, release candidate, and stable release must state each gate as `pass`, `fail`, or `not run`.

1. **Correctness:** golden fixtures and differential/property tests.
2. **Safety:** threat-model regressions, secret redaction, dependency/license policy.
3. **Resources:** bounded memory/disk/time and cancellation.
4. **Compatibility:** native schema migration and explicit Hop import matrix.
5. **UX/accessibility:** golden-path E2E, keyboard path, WCAG checks, translation layout.
6. **Performance:** comparable datasets and retained raw results; never README estimates.
7. **Delivery:** clean-machine install, launch, update/rollback, uninstall, and artifact verification.
8. **Documentation:** quickstart, diagnostics, limitations, migration, backup, and recovery.

## 9. Work order for the first implementation cycle

Do not begin by porting all existing Transforms. The first cycle is:

1. Phase 0.1 claim/repository audit and version-source ADR.
2. Phase 0.2 Hop/Airbyte task walkthroughs and low-fidelity interaction prototype tests.
3. Phase 0.3 native format, execution model, UI shell, and extension ADRs.
4. Phase 0.4 threat model plus small reproducible baseline.
5. Phase 1 native format, Transform manifest, and validator.
6. Phase 2 CSV vertical slice through the new engine and CLI.
7. Phase 3 the same slice through Studio, including state-fault injection.
8. Only then expand Tier-1 connectors and transforms in Phase 4.

Each implementation issue should fit one reviewable contract or fixture family. An issue closes only when its tests and documentation satisfy the relevant phase exit gate.

## 10. Evidence layout

The rebuild should create and maintain:

```text
docs/adr/                 architecture and product decisions
docs/product/             users, golden paths, terminology, usability results
docs/product/competitors/ pinned task walkthroughs and reproduced failure hypotheses
docs/security/            threat model, capability policy, risk register
spec/                     native schemas and version migrations
tests/golden/             correctness and diagnostic fixtures
tests/hop-compat/         pinned Hop inputs and normalized expected behavior
tests/fault/              cancellation and failure-injection scenarios
benchmarks/               generators, manifests, raw results, and reports
packaging/                artifact definitions and clean-machine smoke tests
```

## 11. Competitor reference baselines

The initial migration baseline is Apache Hop 2.19.0, dated 2026-08-17 on the official download page. Relevant official behavior and scope references:

- [What is Apache Hop?](https://hop.apache.org/manual/latest/getting-started/hop-what-is-hop.html)
- [Hop GUI](https://hop.apache.org/manual/latest/getting-started/hop-gui.html)
- [Hop tools and `hop-run`](https://hop.apache.org/manual/latest/getting-started/hop-tools.html)
- [Projects and environments](https://hop.apache.org/manual/latest/projects/projects-environments.html)
- [Pipeline run configurations](https://hop.apache.org/manual/latest/metadata-types/pipeline-run-config.html)
- [Apache Hop downloads](https://hop.apache.org/download/)

Pinning the oracle prevents a moving target. Supporting a later Hop version requires a deliberate compatibility-baseline update and a new differential run.

Airbyte is a UX and data-movement comparison, not the execution-semantics oracle. Its official platform documentation describes a UI that guides connection setup and automated syncs, alongside API, SDK, Terraform, cloud, and self-managed interfaces. Pin the tested Airbyte deployment/version in each walkthrough because the hosted and self-managed products change independently.

- [Airbyte data replication platform](https://docs.airbyte.com/platform)
- [Airbyte documentation](https://docs.airbyte.com/)
