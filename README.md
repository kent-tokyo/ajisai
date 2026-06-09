# Ajisai (紫陽花)

A lightweight, blazing-fast ETL engine compatible with Apache Hop, rewritten in Rust.

> Keeps the power of Apache Hop's data transformation capabilities — without the JVM, in a single binary.

English | [日本語](README_ja.md) | [中文](README_zh.md)

---

![Demo](docs/screenshots/demo.gif)

| Branching pipeline | After execution | Properties panel |
|:---:|:---:|:---:|
| ![Pipeline](docs/screenshots/04_showcase_pipeline.png) | ![Run](docs/screenshots/05_showcase_run.png) | ![Properties](docs/screenshots/06_properties_panel.png) |

| Startup | Japanese UI | Japanese run |
|:---:|:---:|:---:|
| ![Startup](docs/screenshots/01_startup.png) | ![Japanese](docs/screenshots/08_japanese_ui.png) | ![Japanese run](docs/screenshots/09_japanese_run.png) |

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
| **Visual GUI** | ○ | ○ | ○ | × | × | × |
| **CLI batch execution** | ○ | ○ | ○ | ○ | ○ | Script |
| **Apache Hop compatible** | reads `.hpl` | native | △ shared ancestry | × | × | × |
| **File I/O** | CSV / JSON / Excel / Parquet / XML / REST API | Many | Many | HDFS / S3 etc. | DB only | CSV / Parquet etc. |
| **Scale target** | up to ~100M rows | up to ~10M rows | up to ~10M rows | billions+ | DB-dependent | up to ~100M rows |
| **Windows support** | ○ | ○ | ○ | △ | ○ | ○ |
| **No cluster required** | ○ | ○ | ○ | × | ○ | ○ |
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
- **CUI / GUI** — CLI tool and visual pipeline editor (Electron + React, VS Code style)
- **Cross-platform** — Windows / macOS / Linux
- **Multilingual** — Japanese / English (UI and CLI)

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

### Run a pipeline (.hpl)

Defines data transformation steps: reading, processing, and writing data.

```bash
ajisai-cli run -p path/to/pipeline.hpl
```

### Run a workflow (.hwf)

Defines orchestration logic: controlling pipeline execution order, file operations, and error handling.

```bash
ajisai-cli run-workflow -p path/to/workflow.hwf
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

## Supported Transforms (53)

### I/O

| Transform | Description |
|---|---|
| `CsvFileInput` | Read CSV files |
| `CsvFileOutput` | Write CSV files |
| `JsonFileInput` | Read JSON files (array or JSONL format) |
| `JsonFileOutput` | Write JSON files (array or JSONL format) |
| `JsonFieldInput` | Parse a JSON string field into separate fields |
| `JsonFieldOutput` | Serialize selected fields into a JSON string field |
| `ExcelFileInput` | Read Excel files (.xlsx) |
| `ExcelFileOutput` | Write Excel files (.xlsx) |
| `ParquetFileInput` | Read Parquet files |
| `ParquetFileOutput` | Write Parquet files |
| `XmlFileInput` | Read XML files |
| `TableInput` | Read from a database table via SQL (SQLite / PostgreSQL / MySQL) |
| `TableOutput` | Write rows to a database table (Insert / Upsert / Overwrite) |
| `GenerateRows` | Generate rows from inline data |
| `RestClient` | HTTP GET / POST / PUT / DELETE |
| `GetFileNames` | Scan directory, output file metadata as rows |
| `LoadFileContent` | Read file content into a field |
| `WriteToFile` | Write field value to a file |
| `PipelineExecutor` | Run a .hpl sub-pipeline |

### Transformation

| Transform | Description |
|---|---|
| `FilterRows` | Filter rows by condition expressions |
| `SelectValues` | Select, rename, and cast fields |
| `SortRows` | Sort by multiple fields (rayon parallel sort) |
| `AddConstants` | Append constant-value fields to each row |
| `AddSequence` | Auto-increment sequence field |
| `CalculatorStep` | Compute new fields: arithmetic, string ops, type casts |
| `Deduplicate` | Remove duplicate rows by all fields or a key subset |
| `IfNull` | Replace null values with typed defaults |
| `StringOperations` | Trim, case conversion, pad, substring on string fields |
| `ReplaceInString` | Search and replace across multiple field/pattern pairs |
| `ConcatFields` | Join multiple fields with a separator into a new field |
| `SplitFieldToRows` | Expand one delimited field into multiple rows |
| `AnalyticQuery` | Window functions: ROW_NUMBER, RANK, LAG/LEAD, SUM/AVG/MIN/MAX over partition |
| `MemoryGroupBy` | Group-by aggregation (sum / avg / min / max / count) |
| `AppendStreams` | Merge multiple input streams |
| `RowNormaliser` | Pivot wide to long |
| `RowDenormaliser` | Pivot long to wide |
| `WriteToLog` | Log rows at a given level |
| `CloneRow` | Duplicate each row N times |
| `FieldSplitter` | Split field by delimiter into multiple columns |
| `UniqueRows` | Keep first occurrence per key |
| `NumberRange` | Classify numeric values into bands |
| `ValueMapper` | Map field values via lookup table |
| `ExecuteSQL` | Execute SQL once or per row |
| `Dummy` | Pass-through (no-op) |
| `Abort` | Abort pipeline on condition |
| `RegexEval` | Extract regex capture groups |
| `ScriptStep` | Execute Rhai script per row |

### Join / Lookup

| Transform | Description |
|---|---|
| `MergeJoin` | Sort-merge join of two sorted streams (Inner / Left / Right / Full) |
| `StreamLookup` | In-memory hash join for dimension lookups |
| `DatabaseLookup` | Parameterized SQL lookup against a database |

### Variables / Flow

| Transform | Description |
|---|---|
| `SetVariable` | Set a pipeline variable |
| `GetVariable` | Read a pipeline variable into a field |
| `SwitchCase` | Route rows to different outputs by value |

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
│   ├── core/          Row / RowSchema / Value types, pipeline execution engine, model (Node/Edge)
│   ├── transforms/    Built-in transform implementations (50+ transforms)
│   ├── hop-compat/    .hpl / .hwf / .ktr / .dtsx parser, Apache Hop compatibility layer
│   ├── cli/           ajisai-cli binary
│   └── server/        JSON-RPC server for Electron GUI (stdin/stdout IPC)
├── electron/          Electron + React GUI (VS Code style, Phase 6B)
└── tests/fixtures/    Sample pipelines and test data
```

### Electron GUI Architecture

```
Electron main process
  ├─ Sidecar (ajisai-server stdin/stdout JSON-RPC)
  └─ IPC handlers (loadPipeline, savePipeline, runPipeline, etc.)

Electron renderer (React + TypeScript)
  ├─ Components (Canvas, Sidebar, PropertiesPanel, LogPanel)
  ├─ Zustand store (PipelineState, nodeStatuses, logLines)
  └─ contextBridge API (window.ajisai)
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

### Completed Phases

| Phase | Scope | Status |
|---|---|---|
| Phase 1 | CLI + core transforms + .hpl compatibility | ✅ Done |
| Phase 2 | JSON / Calculator / Join / Lookup / DB / .hwf workflow | ✅ Done |
| Phase 3 | GUI — egui visual pipeline editor (archived, replaced by Electron) | ✅ Done |
| Phase 4 | Multilingual UI, packaging, release CI | ✅ Done |
| Phase 5 | 50 transforms: scripting (Rhai), sub-pipeline, file ops | ✅ Done |
| Phase 6A | Window functions (AnalyticQuery), JSON field transforms | ✅ Done |

### Completed: Phase 6B-6C (Electron GUI Migration + Polish)

| Subphase | Scope | Status |
|---|---|---|
| 6B-0 | Extract shared types to ajisai-core | ✅ Done |
| 6B-1 | JSON-RPC server (crates/server) | ✅ Done |
| 6B-2 | Electron scaffold + Zustand store | ✅ Done |
| 6B-3 | Canvas (@xyflow/react) + UI components | ✅ Done |
| 6B-4 | File I/O + keyboard shortcuts + MenuBar | ✅ Done |
| 6C-1 | Per-transform form definitions (50+ forms) | ✅ Done |
| 6C-2 | Undo/Redo with zundo middleware | ✅ Done |
| 6C-3 | Window state persistence (localStorage) | ✅ Done |
| 6C-4 | Performance optimization (React.memo, useMemo) | ✅ Done |
| 6C-5 | Release preparation (electron-builder, auto-update) | ✅ Done |

### Completed: Phase 6D (egui GUI Deprecation) ✅ DONE

- ✅ Deleted `crates/gui` directory (egui implementation)
- ✅ Created archive branch: `archive/egui-gui-0.1.0`
- ✅ Full egui code history preserved in archive branch
- ✅ Electron + React GUI is now the primary UI standard
- **Access legacy code**: `git checkout archive/egui-gui-0.1.0`

### Current: Phase 7 (Enhanced Form Builder) 📋 PLANNED

After 0.3.0 release (see [Roadmap](#roadmap) below)

### Future: Phase 8 (Advanced Analytics) 📋 PLANNED

---

## Roadmap

### Release 0.3.0 (Current) 🔄 IN PROGRESS

Focus: **UI Polish + Performance Optimization**

| Component | Status |
|---|---|
| Phase 9: UI Polish (9-1 to 9-4) | ✅ Complete |
| Phase 6C: GUI Polish & Optimization (6C-1 to 6C-5) | ✅ Complete |
| Phase 6D: egui GUI Deprecation | ✅ Complete |

**Next**: Release v0.3.0 with `git tag v0.3.0`

### Release 0.4.0 🔄 PLANNED

Focus: **Advanced Data Handling + Profiling**

- Advanced data preview panel
- Column-level data profiling
- Real-time pipeline monitoring dashboard
- Custom transform creation UI
- Plugin system foundations

**Timeline**: Q3 2026

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
