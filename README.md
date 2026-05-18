# Ajisai (紫陽花)

A lightweight, blazing-fast ETL engine compatible with Apache Hop, rewritten in Rust.

> Keeps the power of Apache Hop's data transformation capabilities while eliminating the complexity.

English | [日本語](README_ja.md) | [中文](README_zh.md)

---

## Features

- **Apache Hop compatible** — reads `.hpl` pipeline files directly
- **Fast & memory-efficient** — native Rust binary; async execution via tokio, CPU parallelism via rayon
- **CUI / GUI** — CLI tool (Phase 1), visual pipeline editor (Phase 3, planned)
- **Cross-platform** — Windows / macOS / Linux
- **Multilingual** — Japanese / English (Phase 4, planned)

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

## Supported Transforms

| Transform | Description |
|---|---|
| `CsvFileInput` | Read CSV files (auto header detection, type coercion) |
| `CsvFileOutput` | Write CSV files |
| `JsonFileInput` | Read JSON files (array or JSONL format) |
| `JsonFileOutput` | Write JSON files (array or JSONL format) |
| `FilterRows` | Filter rows by condition expressions |
| `SelectValues` | Select, rename, and cast fields |
| `SortRows` | Sort by multiple fields (rayon parallel sort) |
| `AddConstants` | Append constant-value fields to each row |
| `CalculatorStep` | Compute new fields: arithmetic, string ops, type casts, IfNull |
| `StreamLookup` | In-memory hash join for dimension lookups |
| `MergeJoin` | Sort-merge join of two sorted streams (Inner / Left / Right / Full) |
| `Deduplicate` | Remove duplicate rows by all fields or a key subset |
| `TableInput` | Read from a database table via SQL (SQLite / PostgreSQL / MySQL) |
| `TableOutput` | Write rows to a database table (Insert / Upsert / Overwrite) |

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
│   └── cli/           ajisai-cli binary
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
| Phase 1 | CLI + core transforms + .hpl compatibility | Done |
| Phase 2 | JSON / Calculator / StreamLookup / MergeJoin / Deduplicate / DB / .hwf | Done |
| Phase 3 | GUI — egui visual pipeline editor | Planned |
| Phase 4 | Full i18n UI, multi-platform packaging | Planned |

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

## Ajisai vs Apache Hop

| | Apache Hop | Ajisai |
|---|---|---|
| Runtime | JVM | Native Rust binary |
| Memory footprint | Hundreds of MB+ | A few MB+ |
| Startup time | Several seconds | Instant |
| UI | Feature-rich, complex | Simplicity first |
| .hpl / .hwf | Native | Import supported |

---

## License

MIT OR Apache-2.0
