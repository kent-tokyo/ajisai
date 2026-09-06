# Ajisai

Ajisai is a local-first Rust ETL platform being rebuilt for clear authoring,
bounded execution, safe Apache Hop migration, and one definition shared by
Studio and CLI. `Pipeline` is the row-stream graph; `Workflow` is the
orchestration graph. Performance and compatibility claims require the pinned
fixtures and measurements in [`ROADMAP.md`](ROADMAP.md).

## Current status

The workspace is an active rebuild baseline (`0.1.1`), not a production or
full-Hop-compatibility release. Implemented slices include:

- Native Pipeline/Workflow JSON contracts, validation, canonical migration,
  semantic diff, and Hop error-hop preservation;
- bounded Tokio execution with cancellation, timeout, row and buffered-row
  limits, atomic file sinks, structured run/node events, and secret redaction;
- CSV, JSON, XML, Parquet, Excel, SQLite/PostgreSQL, and REST transforms;
- Hop assessment (`supported`, `supported-with-difference`, `unsupported`) and
  non-executing project inventory;
- CLI commands for `validate`, `explain`, `inspect`, `preview`, `assess`,
  `scan`, `migrate`, `doctor`, `run`, and `list-transforms`.

Known release gates remain open: Studio usability, Hop 2.19.0 differential
benchmarks, large-fixture memory/disk tests, fuzzing, independent security
review, and reproducible installation artifacts.

## Build and run

```bash
cargo build
cargo test --workspace
cargo run --bin ajisai-cli -- validate -p tests/fixtures/minimal.ajp
cargo run --bin ajisai-cli -- run -p tests/fixtures/full-csv.ajp
# Machine-readable result for CI
cargo run --bin ajisai-cli -- run -p tests/fixtures/full-csv.ajp --json
# Inspect the execution plan without side effects
cargo run --bin ajisai-cli -- run -p tests/fixtures/full-csv.ajp --dry-run --json
# Validate a Hop workflow graph without executing actions
cargo run --bin ajisai-cli -- run-workflow -p path/to/workflow.hwf --dry-run --json
# Bash completion
cargo run --bin ajisai-cli -- completions bash > /tmp/ajisai-cli.bash
```

Useful execution policies are `--max-rows`, `--max-buffered-rows`,
`--timeout-secs`, and `--no-network`. Hop projects can be inspected before
execution:

```bash
ajisai-cli scan -p path/to/hop-project --json
ajisai-cli assess -p path/to/pipeline.hpl --json
```

## Documentation

- [`ROADMAP.md`](ROADMAP.md): canonical Phase 0–12 plan and evidence gates
- [`docs/rebuild/current-state.md`](docs/rebuild/current-state.md): current audit
- [`docs/product/golden-workflows.md`](docs/product/golden-workflows.md): acceptance contracts
- [`docs/adr/`](docs/adr/): architecture and policy decisions
- [`docs/release/`](docs/release/): candidate checklist, risks, and security
- [`benchmarks/README.md`](benchmarks/README.md): reproducible benchmark contract
- [`CHANGELOG.md`](CHANGELOG.md): historical release notes only

## License

MIT OR Apache-2.0.
