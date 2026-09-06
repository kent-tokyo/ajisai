# Ajisai

Ajisai は、明快な操作、安全な Apache Hop 移行、メモリ境界のある実行を
目指して Rust で再構築中のローカルファースト ETL です。`Pipeline` は
行ストリームのグラフ、`Workflow` はオーケストレーションのグラフです。
性能・互換性の主張は [`ROADMAP.md`](ROADMAP.md) の固定 fixture と測定結果
に基づきます。

## 現在の状態

ワークスペースは再構築中の `0.1.1` ベースラインであり、本番版でも完全な
Hop 互換版でもありません。現在の実装スライスは次のとおりです。

- Native Pipeline/Workflow の JSON 契約、検証、正規化 migration、semantic
  diff、Hop の error hop 保持;
- bounded Tokio 実行、キャンセル・timeout・行数・buffered-row 上限、
  atomic sink、構造化 run/node イベント、secret redaction;
- CSV、JSON、XML、Parquet、Excel、SQLite/PostgreSQL、REST transforms;
- Hop の `supported` / `supported-with-difference` / `unsupported` assessment
  と、実行しないプロジェクト棚卸し;
- `validate`、`explain`、`inspect`、`preview`、`assess`、`scan`、`migrate`、
  `doctor`、`run`、`list-transforms` CLI。

Studio の使いやすさ、Hop 2.19.0 差分ベンチマーク、大規模メモリ・ディスク
試験、fuzzing、独立セキュリティレビュー、再現可能な配布物は未完了です。

## ビルドと実行

```bash
cargo build
cargo test --workspace
cargo run --bin ajisai-cli -- validate -p tests/fixtures/minimal.ajp
cargo run --bin ajisai-cli -- run -p tests/fixtures/full-csv.ajp
# CI向けの機械可読結果
cargo run --bin ajisai-cli -- run -p tests/fixtures/full-csv.ajp --json
# 実行計画だけを確認（副作用なし）
cargo run --bin ajisai-cli -- run -p tests/fixtures/full-csv.ajp --dry-run --json
# Hop workflow のグラフをアクション実行なしで検証
cargo run --bin ajisai-cli -- run-workflow -p path/to/workflow.hwf --dry-run --json
# bash補完
cargo run --bin ajisai-cli -- completions bash > /tmp/ajisai-cli.bash
```

実行ポリシーは `--max-rows`、`--max-buffered-rows`、`--timeout-secs`、
`--no-network` です。Hop プロジェクトは実行前に確認できます。

```bash
ajisai-cli scan -p path/to/hop-project --json
ajisai-cli assess -p path/to/pipeline.hpl --json
```

## ドキュメント

- [`ROADMAP.md`](ROADMAP.md): Phase 0–12 の正規ロードマップと証拠ゲート
- [`docs/rebuild/current-state.md`](docs/rebuild/current-state.md): 実装監査
- [`docs/product/golden-workflows.md`](docs/product/golden-workflows.md): 受入契約
- [`docs/adr/`](docs/adr/): アーキテクチャ決定
- [`docs/release/`](docs/release/): 候補版チェック・リスク・監査
- [`benchmarks/README.md`](benchmarks/README.md): ベンチマーク契約
- [`CHANGELOG.md`](CHANGELOG.md): 過去版の履歴のみ

## ライセンス

MIT OR Apache-2.0。
