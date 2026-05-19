# Ajisai (紫陽花)

A lightweight, blazing-fast ETL engine compatible with Apache Hop, rewritten in Rust.

> Keeps the power of Apache Hop's data transformation capabilities — without the JVM, in a single binary.

English | [日本語](README_ja.md) | [中文](README_zh.md)

---

## Why Ajisai?

### The problem with existing ETL tools

| Tool | Pain point |
|---|---|
| Apache Hop / Talend / Pentaho | Requires JVM. Seconds to start, hundreds of MB of memory just to boot |
| Apache Spark / Flink | Built for petabyte scale. Massive overkill for batch jobs under tens of GB |
| dbt | SQL transformations only. Poor fit for file I/O and complex row-level logic |
| Airbyte / Fivetran | Connector-focused EL(T). Hard to express transformation logic |
| Python (pandas/Polars) | Flexible, but no GUI for non-engineers and no visual pipeline designer |

### What Ajisai does differently

- **Single binary** — copy `ajisai-cli` and run. Zero runtime dependencies
- **Instant startup** — no JVM warmup. Works great in cron jobs, CI, and AWS Lambda
- **Low memory** — process millions of rows in tens of MB
- **GUI + CLI** — design pipelines visually, execute them from the command line
- **Reuse Apache Hop assets** — load existing `.hpl` files without modification

---

## Comparison with other tools

| | **Ajisai** | Apache Hop | Pentaho PDI | Apache Spark | dbt | Polars (Python) |
|---|---|---|---|---|---|---|
| **Runtime** | Native Rust | JVM | JVM | JVM / cluster | Python + DB adapter | Python |
| **Installation** | Single binary | JVM + 500 MB+ | JVM + 500 MB+ | Cluster setup | pip + DB connection | pip |
| **Startup time** | **< 10 ms** | 3–10 s | 3–10 s | 30 s+ | Several seconds | ~1 s |
| **Memory footprint** | **~10 MB+** | 256 MB+ | 256 MB+ | GB+ | DB-dependent | tens of MB+ |
| **Visual GUI** | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **CLI batch execution** | ✅ | ✅ | ✅ | ✅ | ✅ | Script |
| **Apache Hop compatible** | ✅ reads `.hpl` | ✅ native | ⚠️ shared ancestry | ❌ | ❌ | ❌ |
| **File I/O** | CSV / JSON / DB | Many | Many | HDFS / S3 etc. | DB only | CSV / Parquet etc. |
| **Scale target** | up to ~100M rows | up to ~10M rows | up to ~10M rows | billions+ | DB-dependent | up to ~100M rows |
| **Windows support** | ✅ | ✅ | ✅ | ⚠️ | ✅ | ✅ |
| **No cluster required** | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ |
| **License** | MIT / Apache-2.0 | Apache-2.0 | Apache-2.0 | Apache-2.0 | Apache-2.0 | MIT |

### Where Ajisai shines

- **CI/CD-embedded ETL** — runs inside GitHub Actions or GitLab CI with no extra dependencies
- **Edge / embedded environments** — IoT devices and RAM-constrained systems
- **Migrating from Apache Hop** — swap the execution engine without touching `.hpl` files
- **Microservice ETL** — keep Docker images minimal; no JVM layer
- **Scheduled batch jobs** — cron-friendly; no JVM warmup cost per invocation

---

## Features

- **Apache Hop compatible** — reads `.hpl` pipeline files directly
- **Fast & memory-efficient** — native Rust binary; async execution via tokio, CPU parallelism via rayon
- **CUI / GUI** — CLI tool and visual pipeline editor (egui)
- **Cross-platform** — Windows / macOS / Linux
- **Multilingual** — Japanese / English

---

## Installation

```bash
git clone <repo>
cd ajisai
cargo build --release
```

The binary is generated at `target/release/ajisai-cli`.

---

## Quick Start

### Run a pipeline

```bash
ajisai-cli run -p path/to/pipeline.hpl
```

### Validate a pipeline without executing it

```bash
ajisai-cli validate -p path/to/pipeline.hpl
```

### List available transforms

```bash
ajisai-cli list-transforms
```

### Pass environment variables

```bash
ajisai-cli run -p pipeline.hpl -e INPUT_DIR=/data -e OUTPUT_DIR=/output
```

Reference them inside the pipeline as `${INPUT_DIR}`.

---

## Supported Transforms (19)

### I/O

| Transform | Description |
|---|---|
| `CsvFileInput` | Read CSV files (auto header detection, type coercion) |
| `CsvFileOutput` | Write CSV files |
| `JsonFileInput` | Read JSON files (array or JSONL format) |
| `JsonFileOutput` | Write JSON files (array or JSONL format) |
| `TableInput` | Read from a database table via SQL (SQLite / PostgreSQL / MySQL) |
| `TableOutput` | Write rows to a database table (Insert / Upsert / Overwrite) |

### Transformation

| Transform | Description |
|---|---|
| `FilterRows` | Filter rows by condition expressions |
| `SelectValues` | Select, rename, and cast fields |
| `SortRows` | Sort by multiple fields (rayon parallel sort) |
| `AddConstants` | Append constant-value fields to each row |
| `CalculatorStep` | Compute new fields: arithmetic, string ops, type casts |
| `Deduplicate` | Remove duplicate rows by all fields or a key subset |
| `IfNull` | Replace null values with typed defaults |
| `StringOperations` | Trim, case conversion, pad, substring on string fields |
| `ReplaceInString` | Search and replace across multiple field/pattern pairs |
| `ConcatFields` | Join multiple fields with a separator into a new field |
| `SplitFieldToRows` | Expand one delimited field into multiple rows |

### Join / Lookup

| Transform | Description |
|---|---|
| `MergeJoin` | Sort-merge join of two sorted streams (Inner / Left / Right / Full) |
| `StreamLookup` | In-memory hash join for dimension lookups |
| `DatabaseLookup` | Parameterized SQL lookup against a database |

---

## Pipeline File (.hpl) Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<pipeline>
  <name>My Pipeline</name>
  <transform>
    <name>CSV Input</name>
    <type>CSVFileInput</type>
    <filename>${INPUT_FILE}</filename>
    <separator>,</separator>
    <header>Y</header>
  </transform>
  <transform>
    <name>Filter Adults</name>
    <type>FilterRows</type>
  </transform>
  <transform>
    <name>CSV Output</name>
    <type>CSVFileOutput</type>
    <filename>${OUTPUT_FILE}</filename>
    <separator>,</separator>
    <header>Y</header>
  </transform>
  <order>
    <hop>
      <from>CSV Input</from>
      <to>Filter Adults</to>
      <enabled>Y</enabled>
    </hop>
    <hop>
      <from>Filter Adults</from>
      <to>CSV Output</to>
      <enabled>Y</enabled>
    </hop>
  </order>
</pipeline>
```

Pipeline files created with Apache Hop can be loaded directly.

---

## Architecture

```
ajisai/
├── crates/
│   ├── core/          Row / RowSchema / Value types, pipeline execution engine
│   ├── transforms/    Built-in transform implementations
│   ├── hop-compat/    .hpl / .hwf XML parser, Apache Hop compatibility layer
│   ├── cli/           ajisai-cli binary
│   └── gui/           egui visual pipeline editor
└── tests/fixtures/    Sample pipelines and test data
```

### Execution Model

```
[CsvInput] ──mpsc──> [FilterRows] ──mpsc──> [SortRows] ──mpsc──> [CsvOutput]
  tokio task           tokio task             tokio task            tokio task
                                           (rayon inside)
```

- Inter-node transport: bounded `tokio::sync::mpsc` channels (backpressure built-in)
- I/O-bound work: tokio async/await
- CPU-bound work (Sort, etc.): rayon parallel

---

## Roadmap

| Phase | Scope | Status |
|---|---|---|
| Phase 1 | CLI + core transforms + .hpl compatibility | ✅ Done |
| Phase 2 | JSON / Calculator / Join / Lookup / DB / .hwf workflow | ✅ Done |
| Phase 3 | GUI — egui visual pipeline editor | ✅ Done |
| Phase 4A | String transforms: IfNull / StringOps / ConcatFields / Split | ✅ Done |
| Phase 4B | GroupBy aggregation, Switch/Case routing, multi-output streams | In progress |
| Phase 4C | Excel / XML / REST Client, cloud storage | Planned |

---

## Build & Test

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run all tests
cargo test --workspace

# Run with debug logging
ajisai-cli --log-level debug run -p pipeline.hpl
```

---

## Trademark Notice

> "Apache Hop" is a trademark of the Apache Software Foundation. Ajisai is an independent project with no affiliation, endorsement, or sponsorship from the ASF.

---

## License

MIT OR Apache-2.0
