# Ajisai

Apache Hop 互換の超軽量・爆速 ETL エンジン。Rust で再構築。

> Apache Hop の強力なデータ変換機能を維持しつつ、煩雑な UI をシンプルに。

[English](README.md) | 日本語 | [中文](README_zh.md)

---

## 特徴

- **Apache Hop 互換** — `.hpl` パイプラインファイルを直接読み込める
- **高速・省メモリ** — Rust ネイティブバイナリ。tokio による非同期実行、rayon による CPU 並列処理
- **CUI / GUI 両対応** — CLI ツール (Phase 1)、ビジュアルエディタ (Phase 3 予定)
- **クロスプラットフォーム** — Windows / macOS / Linux
- **多言語対応** — 日本語 / English (Phase 4 予定)

---

## インストール

```bash
git clone <repo>
cd ajisai
cargo build --release
```

バイナリは `target/release/ajisai-cli` に生成されます。

---

## クイックスタート

### パイプラインを実行

```bash
ajisai-cli run -p path/to/pipeline.hpl
```

### パイプラインを検証（実行なし）

```bash
ajisai-cli validate -p path/to/pipeline.hpl
```

### 利用可能な Transform 一覧

```bash
ajisai-cli list-transforms
```

### 環境変数を渡す

```bash
ajisai-cli run -p pipeline.hpl -e INPUT_DIR=/data -e OUTPUT_DIR=/output
```

パイプライン内で `${INPUT_DIR}` として参照できます。

---

## サポートする Transform

| Transform | 説明 |
|---|---|
| `CsvFileInput` | CSV ファイルの読み込み（ヘッダー検出・型変換対応）|
| `CsvFileOutput` | CSV ファイルへの書き込み |
| `JsonFileInput` | JSON ファイルの読み込み（Array / JSONL 両対応）|
| `JsonFileOutput` | JSON ファイルへの書き込み（Array / JSONL 両対応）|
| `FilterRows` | 条件式による行フィルタリング |
| `SelectValues` | フィールドの選択・名前変更・型キャスト |
| `SortRows` | 複数フィールドによるソート（rayon 並列ソート）|
| `AddConstants` | 固定値フィールドの追加 |
| `CalculatorStep` | フィールド計算（四則演算・文字列操作・型変換・IfNull など）|
| `StreamLookup` | インメモリハッシュ結合（ディメンションルックアップ）|
| `MergeJoin` | ソートマージ結合（Inner / Left / Right / Full）|
| `Deduplicate` | 重複行排除（全フィールド / キー指定）|
| `TableInput` | DBテーブルをSQLで読み込み（SQLite / PostgreSQL / MySQL）|
| `TableOutput` | DBテーブルへの書き込み（Insert / Upsert / Overwrite）|

---

## パイプラインファイル (.hpl) の例

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

Apache Hop で作成した `.hpl` ファイルをそのまま読み込むことができます。

---

## アーキテクチャ

```
ajisai/
├── crates/
│   ├── core/          Row / RowSchema / Value 型定義、実行エンジン
│   ├── transforms/    標準 Transform 実装群
│   ├── hop-compat/    .hpl / .hwf XML パーサ、Apache Hop 互換レイヤー
│   └── cli/           ajisai-cli バイナリ
└── tests/fixtures/    サンプルパイプライン・テストデータ
```

### 実行モデル

```
[CsvInput] ──mpsc──> [FilterRows] ──mpsc──> [SortRows] ──mpsc──> [CsvOutput]
  tokio task           tokio task             tokio task            tokio task
                                           (rayon 内部)
```

- ノード間通信: `tokio::sync::mpsc` (bounded) でバックプレッシャー制御
- I/O バウンド処理: tokio async/await
- CPU バウンド処理 (Sort など): rayon 並列

---

## 開発状況

| フェーズ | 内容 | 状態 |
|---|---|---|
| Phase 1 | CLI + 基本 Transform + .hpl 互換 | 完成 |
| Phase 2 | JSON / Calculator / StreamLookup / MergeJoin / Deduplicate / DB / .hwf | 完成 |
| Phase 3 | GUI (egui ビジュアルエディタ) | 計画中 |
| Phase 4 | 日英多言語 UI、パッケージング | 計画中 |

---

## ビルド & テスト

```bash
# デバッグビルド
cargo build

# リリースビルド
cargo build --release

# テスト実行
cargo test --workspace

# ログレベルを指定して実行
ajisai-cli --log-level debug run -p pipeline.hpl
```

---

## ライセンス表記

> "Apache Hop" は Apache Software Foundation の商標です。Ajisai は ASF と無関係な独立プロジェクトであり、ASF による承認・提携・保証はありません。

---

## Apache Hop との違い

| | Apache Hop | Ajisai |
|---|---|---|
| 実行環境 | JVM | Rust ネイティブバイナリ |
| メモリ使用量 | 数百 MB〜 | 数 MB〜 |
| 起動時間 | 数秒 | 即時 |
| UI | 高機能・複雑 | シンプル優先 |
| .hpl/.hwf 互換 | ネイティブ | 読み込み対応 |

---

## ライセンス

MIT OR Apache-2.0
