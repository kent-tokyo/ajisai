# Ajisai

Ajisai 是一个正在用 Rust 重建的本地优先 ETL 平台，重点是清晰的编排、
安全的 Apache Hop 迁移和有界资源执行。`Pipeline` 表示行流图，`Workflow`
表示编排图。性能与兼容性声明必须以 [`ROADMAP.md`](ROADMAP.md) 中固定的
fixture 和测量结果为依据。

## 当前状态

工作区是重建中的 `0.1.0` 基线，不是生产版，也不声称完整 Hop 兼容。当前
已实现的切片包括：

- Native Pipeline/Workflow JSON 契约、验证、规范化迁移、语义 diff 和 Hop
  error hop 保留；
- 有界 Tokio 执行、取消/超时/行数/缓冲行限制、原子 sink、结构化运行事件
  和 secret 脱敏；
- CSV、JSON、XML、Parquet、Excel、SQLite/PostgreSQL 和 REST transforms；
- Hop 的 `supported`、`supported-with-difference`、`unsupported` assessment
  以及不执行内容的项目清单；
- `validate`、`explain`、`inspect`、`preview`、`assess`、`scan`、`migrate`、
  `doctor`、`run`、`list-transforms` CLI。

Studio 易用性、Hop 2.19.0 差分基准、大 fixture 内存/磁盘测试、fuzzing、独立
安全审查和可复现安装包仍未完成。

## 构建与运行

```bash
cargo build
cargo test --workspace
cargo run --bin ajisai-cli -- validate -p tests/fixtures/minimal.ajp
cargo run --bin ajisai-cli -- run -p tests/fixtures/full-csv.ajp
```

运行策略包括 `--max-rows`、`--max-buffered-rows`、`--timeout-secs` 和
`--no-network`。Hop 项目可在执行前检查：

```bash
ajisai-cli scan -p path/to/hop-project --json
ajisai-cli assess -p path/to/pipeline.hpl --json
```

## 文档

- [`ROADMAP.md`](ROADMAP.md)：Phase 0–12 正式路线图和证据门槛
- [`docs/rebuild/current-state.md`](docs/rebuild/current-state.md)：实现审计
- [`docs/product/golden-workflows.md`](docs/product/golden-workflows.md)：验收契约
- [`docs/adr/`](docs/adr/)：架构决策
- [`docs/release/`](docs/release/)：候选版清单、风险和安全审计
- [`benchmarks/README.md`](benchmarks/README.md)：基准测试契约
- [`CHANGELOG.md`](CHANGELOG.md)：仅保留历史版本记录

## 许可证

MIT OR Apache-2.0。
