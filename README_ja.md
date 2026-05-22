# Ajisai

Apache Hop 互換の超軽量・爆速 ETL エンジン。Rust で再構築。

> Apache Hop の強力なデータ変換機能を維持しつつ、JVM を捨て、シングルバイナリで動く。

[English](README.md) | 日本語 | [中文](README_zh.md)

---

![Demo](docs/screenshots/demo.gif)

| 分岐パイプライン | 実行結果 | プロパティパネル |
|:---:|:---:|:---:|
| ![Pipeline](docs/screenshots/04_showcase_pipeline.png) | ![Run](docs/screenshots/05_showcase_run.png) | ![Properties](docs/screenshots/06_properties_panel.png) |

| 起動画面 | 日本語 UI | 実行後 |
|:---:|:---:|:---:|
| ![Startup](docs/screenshots/01_startup.png) | ![Japanese](docs/screenshots/08_japanese_ui.png) | ![Japanese run](docs/screenshots/09_japanese_run.png) |

---

## なぜ Ajisai を選ぶのか

### 既存の ETL ツールの問題

| ツール | 問題点 |
|---|---|
| Apache Hop / Talend / Pentaho | JVM 必須。起動に数秒、メモリ消費が数百 MB 以上 |
| Apache Spark / Flink | 大規模向け。数十 GB 以下のバッチには過剰 |
| dbt | SQL 変換専用。ファイル I/O・複雑な行変換が苦手 |
| Airbyte / Fivetran | EL(T) コネクタ中心。変換ロジックが書きにくい |
| Python (pandas/Polars) | 柔軟だが、非エンジニア向けの GUI がない |

### Ajisai が解決すること

- **単一バイナリ** — `ajisai-cli` 1ファイルをコピーするだけで動作（依存ゼロ）
- **即時起動** — JVM ウォームアップなし。cron / CI / Lambda で使いやすい
- **省メモリ** — 数百万行のパイプラインを数十 MB で処理
- **GUI & CLI 両対応** — ビジュアルエディタでパイプライン設計 → CLI で本番実行
- **Apache Hop 資産を活かす** — 既存の `.hpl` ファイルをそのまま読み込める

---

## 他ツールとの詳細比較

| | **Ajisai** | Apache Hop | Pentaho PDI | Apache Spark | dbt | Polars (Python) |
|---|---|---|---|---|---|---|
| **実行環境** | Rust ネイティブ | JVM | JVM | JVM / クラスター | Python + DBアダプタ | Python |
| **インストール** | バイナリ 1ファイル | JVM + 500MB+ | JVM + 500MB+ | クラスター構築 | pip + DB接続 | pip |
| **起動時間** | **即時 (< 10ms)** | 3〜10秒 | 3〜10秒 | 30秒〜 | 数秒 | 〜1秒 |
| **メモリ消費** | **〜10MB〜** | 256MB〜 | 256MB〜 | GB〜 | DBに依存 | 数十 MB〜 |
| **ビジュアル GUI** | ○ | ○ | ○ | × | × | × |
| **CLI バッチ実行** | ○ | ○ | ○ | ○ | ○ | スクリプト |
| **Apache Hop 互換** | .hpl 読み込み | ネイティブ | △ 共通祖先 | × | × | × |
| **ファイル I/O** | CSV / JSON / Excel / Parquet / XML / REST API | 多数 | 多数 | HDFS / S3 等 | DBのみ | CSV / Parquet 等 |
| **対象規模** | 〜数億行 | 〜数千万行 | 〜数千万行 | 数十億行〜 | DBに依存 | 〜数億行 |
| **Windows 対応** | ○ | ○ | ○ | △ | ○ | ○ |
| **クラスター不要** | ○ | ○ | ○ | × | ○ | ○ |
| **ライセンス** | MIT / Apache-2.0 | Apache-2.0 | Apache-2.0 | Apache-2.0 | Apache-2.0 | MIT |

### Ajisai が特に有利なシーン

- **CI/CD 組み込み ETL** — GitHub Actions / GitLab CI で追加依存なしに実行
- **エッジ / 組み込み環境** — IoT デバイスや RAM 制限環境でのデータ変換
- **Apache Hop からの移行** — `.hpl` ファイルを変更せずに高速実行エンジンに切り替え
- **マイクロサービスの ETL** — Docker イメージを最小化したい場合
- **定期バッチ** — cron で軽量実行。JVM ウォームアップのオーバーヘッドがない

---

## 特徴

- **Apache Hop 互換** — `.hpl` パイプラインファイルを直接読み込める
- **高速・省メモリ** — Rust ネイティブバイナリ。tokio による非同期実行、rayon による CPU 並列処理
- **CUI / GUI 両対応** — CLI ツールとビジュアルパイプラインエディタ
- **クロスプラットフォーム** — Windows / macOS / Linux
- **多言語対応** — 日本語 / English

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

### パイプライン (.hpl) を実行

データの読み込み・加工・書き出しなどの変換処理を定義したファイル。

```bash
ajisai-cli run -p path/to/pipeline.hpl
```

### ワークフロー (.hwf) を実行

パイプラインの実行順序の制御・ファイル操作・エラー処理などのオーケストレーションを定義したファイル。

```bash
ajisai-cli run-workflow -p path/to/workflow.hwf
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

## サポートする Transform (53種)

### I/O

| Transform | 説明 |
|---|---|
| `CsvFileInput` | CSV ファイル読み込み |
| `CsvFileOutput` | CSV ファイル書き込み |
| `JsonFileInput` | JSON ファイル読み込み（Array / JSONL 両対応）|
| `JsonFileOutput` | JSON ファイル書き込み（Array / JSONL 両対応）|
| `JsonFieldInput` | JSON 文字列フィールドをパースして個別フィールドに展開 |
| `JsonFieldOutput` | 指定フィールドを JSON 文字列フィールドに変換 |
| `ExcelFileInput` | Excel ファイル読み込み（.xlsx）|
| `ExcelFileOutput` | Excel ファイル書き込み（.xlsx）|
| `ParquetFileInput` | Parquet ファイル読み込み |
| `ParquetFileOutput` | Parquet ファイル書き込み |
| `XmlFileInput` | XML ファイル読み込み |
| `TableInput` | DB テーブルを SQL で読み込み（SQLite / PostgreSQL / MySQL）|
| `TableOutput` | DB テーブルへの書き込み（Insert / Upsert / Overwrite）|
| `GenerateRows` | 固定データ行を生成 |
| `RestClient` | HTTP GET / POST / PUT / DELETE |
| `GetFileNames` | ディレクトリをスキャンしてファイルメタデータを行として出力 |
| `LoadFileContent` | ファイル内容をフィールドに読み込み |
| `WriteToFile` | フィールド値をファイルに書き出し |
| `PipelineExecutor` | .hpl サブパイプラインを実行 |

### 変換

| Transform | 説明 |
|---|---|
| `FilterRows` | 条件式による行フィルタリング |
| `SelectValues` | フィールドの選択・名前変更・型キャスト |
| `SortRows` | 複数フィールドによるソート（rayon 並列ソート）|
| `AddConstants` | 固定値フィールドの追加 |
| `AddSequence` | 連番フィールドの追加 |
| `CalculatorStep` | フィールド計算（四則演算・文字列操作・型変換）|
| `Deduplicate` | 重複行排除（全フィールド / キー指定）|
| `IfNull` | NULL 値をデフォルト値で置換 |
| `StringOperations` | trim / 大小文字変換 / pad / substring |
| `ReplaceInString` | 文字列の検索・置換 |
| `ConcatFields` | 複数フィールドを区切り文字で結合 |
| `SplitFieldToRows` | 1 フィールドを区切り文字で複数行に展開 |
| `AnalyticQuery` | ウィンドウ関数（ROW_NUMBER, RANK, LAG/LEAD, パーティション内 SUM/AVG/MIN/MAX）|
| `MemoryGroupBy` | グループ集計（sum / avg / min / max / count）|
| `AppendStreams` | 複数の入力ストリームを結合 |
| `RowNormaliser` | 横持ち→縦持ち変換 |
| `RowDenormaliser` | 縦持ち→横持ち変換 |
| `WriteToLog` | 指定レベルで行をログ出力 |
| `CloneRow` | 各行を N 回複製 |
| `FieldSplitter` | フィールドを区切り文字で複数列に分割 |
| `UniqueRows` | キーで最初の行を保持 |
| `NumberRange` | 数値を範囲に分類 |
| `ValueMapper` | ルックアップテーブルで値をマッピング |
| `ExecuteSQL` | SQL を 1 回またはレコード単位で実行 |
| `Dummy` | パススルー（何もしない）|
| `Abort` | 条件付きでパイプラインを停止 |
| `RegexEval` | 正規表現のキャプチャグループを抽出 |
| `ScriptStep` | 各行に対して Rhai スクリプトを実行 |

### 結合 / ルックアップ

| Transform | 説明 |
|---|---|
| `MergeJoin` | ソートマージ結合（Inner / Left / Right / Full）|
| `StreamLookup` | インメモリハッシュ結合（ディメンションルックアップ）|
| `DatabaseLookup` | SQL によるデータベースルックアップ |

### 変数 / フロー制御

| Transform | 説明 |
|---|---|
| `SetVariable` | パイプライン変数を設定 |
| `GetVariable` | パイプライン変数をフィールドに読み込み |
| `SwitchCase` | 値に応じて行を異なる出力先に振り分け |

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
│   ├── hop-compat/    .hpl / .hwf / .ktr / .dtsx パーサ、Apache Hop 互換レイヤー
│   ├── cli/           ajisai-cli バイナリ
│   └── gui/           egui ビジュアルエディタ
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
| Phase 2 | JSON / Calculator / Join / Lookup / DB / .hwf ワークフロー | 完成 |
| Phase 3 | GUI — egui ビジュアルパイプラインエディタ | 完成 |
| Phase 4 | 多言語 UI・パッケージング・リリース CI | 完成 |
| Phase 5 | Transform 50種：スクリプト（Rhai）・サブパイプライン・ファイル操作 | 完成 |
| Phase 6A | ウィンドウ関数（AnalyticQuery）・JSON フィールド Transform・ワークフローアクション拡充 | 完成 |

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

## ライセンス

MIT OR Apache-2.0
