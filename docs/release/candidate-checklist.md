# Ajisai v2.0.0 target candidate checklist

Current workspace baseline: `0.1.0`. Target publishable tag: `v2.0.0`.

This checklist records the evidence required before treating the current
working tree as a version candidate. It does not change the workspace version.

## Reproducible checks

- [x] `cargo test -p ajisai-core -p ajisai-transforms -p ajisai-cli`
- [x] `cargo run --bin ajisai-cli -- validate -p tests/fixtures/minimal.ajp`
- [x] `cargo run --bin ajisai-cli -- validate -p tests/fixtures/minimal.ajw`
- [x] `cargo run --bin ajisai-cli -- validate -p tests/fixtures/minimal.ajp --json`
- [x] `cargo run --bin ajisai-cli -- inspect -p tests/fixtures/minimal.ajp`
- [x] Native CSV vertical slice matches [`full-csv.expected.csv`](../../tests/fixtures/full-csv.expected.csv)
- [x] Immutable fixture manifest verification (`tests/fixtures/manifest.json`, `scripts/verify-fixtures.py`)
- [x] Vulnerability reporting and v2.x supported-version policy (`SECURITY.md`, `docs/release/support-policy.md`)
- [x] Release workflow archives source/toolchain build provenance (`scripts/generate-build-info.py`)
- [x] Release artifact checksum/provenance verifier (`scripts/verify-release-artifacts.py`)
- [x] Verifier rejects unsafe or missing checksum paths
- [x] Release workflow defines Sigstore keyless signing for `SHA256SUMS`
- [x] Release workflow builds and archives standalone `ajisai-cli` binaries per OS
- [x] Release workflow smoke-tests each standalone CLI binary (`--version`, `doctor --json`)
- [x] Pinned non-root container definition and read-only-root deployment guidance (`Dockerfile`, `docs/release/container.md`)
- [x] Reproducible cron, CI, and container invocation recipes (`docs/release/scheduling.md`)
- [x] Release workflow defines container build and JSON-RPC ping smoke test
- [x] Explicit project-root path resolver rejects absolute, traversal, and escaping paths
- [x] `run --project-root PATH` scopes CSV, XML, Excel, Parquet, file, directory, and sub-pipeline transforms
- [x] `LoadFileContent` row-derived paths obey project-root scope
- [x] `run-workflow --project-root PATH` propagates root scope to pipeline actions
- [x] `run-workflow --json` emits a stable failure envelope and atomically records missing-file failures
- [x] `git diff --check`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy -p ajisai-core --all-targets -- -D warnings`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] CI runs strict workspace Clippy on Ubuntu, macOS, and Windows (`.github/workflows/ci.yml`)
- [x] CI enforces the local unsafe-code policy with `RUSTFLAGS=-Dunsafe_code`
- [x] CI runs the repository secret scan (`scripts/secret-scan.sh`)
- [x] Deterministic CycloneDX 1.5 SBOM generation (`scripts/generate-sbom.py`)
- [x] Versioned benchmark manifest and reproducible diagnostic runner (`benchmarks/manifest.json`, `scripts/run-benchmark.py`)
- [x] Benchmark runner smoke test: `csv-clean-fixture` exit 0 and expected output SHA-256 matches
- [x] Benchmark runner separates cold/warm direct-binary iterations and preserves output hashes
- [x] Benchmark runner supports `--skip-build` for repeatable measurements against a pinned binary
- [x] Benchmark runner records output bytes and platform-normalized child RSS diagnostics
- [x] `cargo deny check licenses sources`
- [x] `cargo deny check advisories` (passes without warnings after updating transitive `spin` to 0.9.9)
- [x] Release workflow reruns advisory/license/source audit before artifact creation
- [x] Hop XML writer/parser compatibility round-trip test
- [x] Repository Hop fixtures parse with names, transforms, and hops
- [x] Hop fixture → Ajisai engine → CSV output matches retained oracle
- [x] Native error-hop fixture routes error rows to an isolated sink
- [x] Native `is_error` edge survives serialization round-trip
- [x] Panic source is converted into a pipeline error and run terminates
- [x] Uncommitted CSV temporary output is removed on sink drop
- [x] JSON array/lines sink temporary output commits atomically and cleans up on drop
- [x] XML sink writes through a temporary file and atomically renames on flush
- [x] Parquet and Excel sinks write through temporary files and atomically rename on success
- [x] SQLite Database sink commit-at-close persists rows under an isolated transaction test
- [x] Remediated quick-xml 0.31 path via calamine 0.36.1 and updated anyhow/crossbeam-epoch advisories

## Candidate gates still open

- [x] Full workspace test (`cargo test --workspace`); release build on pinned reference hardware remains open
- [x] Optimized standalone CLI release build (`cargo build --release --bin ajisai-cli`) and `--version`/`doctor --json` smoke
- [x] CLI signal-driven cancellation integration test (`crates/cli/tests/signal_cancel.rs`, SIGINT -> exit 130)
- [x] CSV invalid typed values are handled deterministically as nulls
- [x] Deterministic CLI cancellation contract test with injected signal future
- [x] Stable CLI exit-code classification (including localized errors) and validation JSON contract
- [x] `run --json` versioned success envelope with row statistics and structured events
- [x] `run --json` stable failure envelope with classified exit code
- [x] `run --record PATH` atomically persists success and dry-run JSON records
- [x] `run --record PATH` atomically persists failure JSON records with the classified exit code
- [x] Run records use process-scoped staging paths to avoid concurrent writer collisions
- [x] Run-record staging files are cleaned up when the final atomic rename fails
- [x] `run-workflow --json --record PATH` emits and persists versioned workflow summaries
- [x] `run-workflow --dry-run --json` validates workflow structure and action compatibility without executing actions
- [x] Workflow action-failure records are not overwritten by the CLI fallback envelope
- [x] Success, dry-run, and failure records include a shared `run_id` correlation key
- [x] `new -o PATH` creates a valid canonical Native Pipeline accepted by `validate`
- [x] `run --dry-run --json` side-effect-free plan envelope
- [x] `completions <shell>` generates bash/zsh/fish/powershell completion scripts
- [x] `--lang ja|en` renders actionable localized CLI errors
- [x] `doctor --json` reports discoverable config and cache directories
- [x] `doctor` resolves XDG and Windows APPDATA/LOCALAPPDATA config/cache conventions
- [x] Cooperative `run --max-rows` input budget
- [x] Cooperative `run --max-buffered-rows` policy for blocking side-input buffers
- [x] Cooperative `run --timeout-secs` execution budget
- [x] `run --no-network` policy blocks REST client initialization
- [x] `run --no-network` policy blocks remote database initialization
- [x] REST SSRF policy rejects private/loopback/link-local IPv4/IPv6 literals and URL credentials
- [x] REST DNS resolution rejects hostnames resolving to blocked IP ranges before request dispatch
- [x] `list-transforms --json` emits a versioned manifest envelope
- [x] `migrate` validates and rewrites a Native document using canonical JSON
- [x] Hop assessment emits stable compatibility statuses/reason codes without execution
- [x] Hop project scan inventories files and pipelines without following symlinks or executing content
- [x] Versioned structured run/node events with stable node IDs and terminal run status
- [x] Preview command with bounded rows and no sink side effects
- [x] Large generated source stops at a bounded row budget without unbounded buffering (`large_source_stops_at_row_budget_without_unbounded_buffering`)
- [x] Injected Transform failure stops a 100,000-row source and records terminal failed events (`injected_failure_stops_large_source_and_records_terminal_failure`); RSS/full-disk and connector-specific failure evidence remain open
- [ ] Hop 2.19.0 normalized-result and throughput comparison
- [ ] Security audit, dependency/license audit, and reproducible artifact archive (license/source and advisory audits now pass after updating `spin` to 0.9.9; Rhai is opt-in and removed from the default graph; release workflow archives locked cargo metadata, CycloneDX SBOM, and sorted SHA256 checksums, while independent security review and artifact verification remain)

`cargo clippy --workspace --all-targets -- -D warnings` passes for the current
workspace. This is static-analysis evidence only and does not replace the
independent security review or dependency advisory gate.

The audit configuration is checked in at [`deny.toml`](../../deny.toml). The
default candidate build excludes the opt-in SQLx MySQL feature (and therefore
its `rsa` path), but the audit still reports unresolved transitive advisories
(yanked `spin` via SQLx 0.9/SQLite; `smartstring` is confined to the opt-in scripting feature) and
default-license policy failures; fixed
`quick-xml`, `anyhow`, and `crossbeam-epoch` findings are retained in the audit
record. The unresolved findings keep
the formal candidate gate open. The direct quick-xml DoS findings were
remediated by upgrading to 0.41.
Detailed evidence and the non-waiver decision are recorded in
[`security-audit-2026-09-05.md`](security-audit-2026-09-05.md).

## Scope statement

The candidate includes the initial Native Pipeline/Workflow contracts,
structural validation, side-effect-free explain/inspect, bounded CSV input,
cooperative cancellation, atomic CSV output, and the CSV vertical slice. It is
not yet a claim of full Apache Hop feature parity or production readiness.
