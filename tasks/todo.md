# Ajisai 開発 TODO

## Phase 1: MVP（完了）

- [x] ワークスペース初期化（Cargo.toml + 4クレートスケルトン）
- [x] `ajisai-core` — Value / Row / Transform trait / PipelineEngine
- [x] `ajisai-transforms` — CsvInput/Output / Filter / Select / Sort / AddConstants
- [x] `ajisai-hop-compat` — .hpl XMLパーサ → Pipeline変換
- [x] `ajisai-cli` — run / validate / list-transforms
- [x] `tests/fixtures/sample.hpl` でCSV→フィルタ→CSV動作確認
- [x] `README.md` / `README_ja.md` 作成

---

## Phase 2: Transform拡張（完了）

- [x] `TableInput` / `TableOutput`（sqlx: SQLite / PostgreSQL / MySQL）
- [x] `DatabaseLookup`（DB参照結合。key_field で引数バインド）
- [x] `MergeJoin`（Inner / LeftOuter / RightOuter / Full）
- [x] `JsonFileInput` / `JsonFileOutput`（Array / JSONL 両対応）
- [x] `CalculatorStep`（四則演算・文字列操作・型変換・IfNull など）
- [x] `StreamLookup`（インメモリハッシュ結合）
- [x] `Deduplicate`（全フィールド / キー指定の重複排除）
- [x] ワークフロー `.hwf` 解析・実行（`ajisai-cli run-workflow`）

---

## Phase 3: GUI（完了）

- [x] `crates/gui` クレート作成（egui 0.31 + eframe 0.31）
- [x] DAGビジュアルエディタ（ノード配置 / ベジェエッジ / ポート接続）
- [x] Transformパレット（左パネル・クリックで追加）
- [x] プロパティパネル（右パネル・JSON設定編集）
- [x] ログパネル（下部・実行ログ表示）
- [x] グリッド背景・ノードドラッグ・右クリック削除
- [x] GUIからエンジン実行（背景スレッド + tokio + mpsc ログストリーム）
- [x] スクロールホイールでズーム（カーソル中心に拡縮）
- [x] `cargo run --bin ajisai-gui` でウィンドウ起動確認

---

## Phase 4: 多言語・パッケージング（完了）

- [x] `rust-i18n` v4 で日英切り替え（GUI メニュー / CLI `--lang en|ja`）
- [x] GUI Language メニューで実行中に言語切替（即時反映）
- [x] `.hpl` 書き出し（GUI → Apache Hop 互換 XML / `write_hpl` / `write_hpl_file`）
- [x] GitHub Actions `release.yml` — タグ push で macOS / Linux / Windows バイナリ生成
- [x] GitHub Actions `ci.yml` — PR/push で自動ビルド・テスト
- [x] `rfd` オプション機能フラグ（`--features rfd` でネイティブファイルダイアログ）

---

## Phase 5A: Priority A Transform（完了）

- [x] `MemoryGroupBy`（インメモリ集計: sum / avg / min / max / count）
- [x] `AppendStreams`（複数入力ストリームの連結）
- [x] `SwitchCase`（条件分岐で出力先を切替）
- [x] `ExcelFileInput` / `ExcelFileOutput`（.xlsx 読み書き）
- [x] `WriteToLog`（ログ出力 Transform）
- [x] `GenerateRows`（指定行数のダミー行生成）
- [x] エンジン機能強化（複数入力ポート対応など）

---

## Phase 5B: Priority B Transform（完了）

- [x] `AddSequence`（連番フィールド追加）
- [x] `SetVariable` / `GetVariable`（パイプライン変数の読み書き）
- [x] `XmlFileInput`（XML → 行変換）
- [x] `RestClient`（HTTP GET/POST/PUT/DELETE）
- [x] `ParquetFileInput` / `ParquetFileOutput`（Apache Parquet 読み書き）
- [x] `RowNormaliser` / `RowDenormaliser`（行の正規化・非正規化）

---

## Phase 5 リファクタリング（完了）

- [x] `utils.rs` 共通ユーティリティ抽出
- [x] `parquet_utils.rs` Parquet 共通処理抽出
- [x] `Value::compare()` 実装（ソート・比較の統一）
- [x] スキーマキャッシュ統一
- [x] パストラバーサル保護（ファイル系 Transform）

---

## Phase 5C-A: Priority C グループA（完了）

- [x] `CloneRow`（行の複製）
- [x] `FieldSplitter`（1フィールドを複数フィールドに分割）
- [x] `UniqueRows`（ソート済み入力前提の重複排除）
- [x] `NumberRange`（数値範囲フィルタ）
- [x] `ValueMapper`（値のマッピング変換）
- [x] `ExecuteSQL`（任意 SQL 実行）

---

## Phase 5C-B: Priority C グループB（完了）

- [x] `Dummy`（パススルー・デバッグ用）
- [x] `Abort`（条件一致時にパイプラインを中断）
- [x] `RegexEval`（正規表現マッチ・グループ抽出）

---

## Phase 5C-C: Priority C グループC（完了）

- [x] `GetFileNames`（ディレクトリからファイル名一覧を行として出力）
- [x] `LoadFileContent`（ファイル内容をフィールドとして読み込み）
- [x] `WriteToFile`（任意テキストをファイルに書き出し）

---

## Phase 5D: グループD — スクリプティング（完了）

- [x] `ScriptStep`（Rhai スクリプトエンジン組み込み）
  - `rhai = { version = "1", features = ["sync"] }` で `Send` 対応
  - 入力行のフィールドを Rhai スコープ変数としてバインド
  - `output_fields` に宣言したフィールドをスクリプト実行後に読み返し

---

## Phase 5E: グループE — サブパイプライン実行（完了）

- [x] `PipelineExecutor`（.hpl サブパイプラインを `close()` 時に1回実行、MVP）
- [x] アーキテクチャ変更: `TransformRegistry` を `ajisai-transforms` → `ajisai-core` に移動（循環依存解消）
- [x] `hop_workflow_to_ajisai()` に `registry_factory` コールバックを追加
- [x] Pentaho/Kettle（.ktr/.kjb）ファイル形式サポート
- [x] SSIS（.dtsx）ファイル形式サポート

---

## 対象外（スコープ外）

以下は意図的に実装しない。必要になった時点で改めて検討する。

- ストリーミング `PipelineExecutor`（行単位のサブパイプライン逐次呼び出し）
- Kafka / Avro / ORC（外部依存が大きい）
- FTP / SFTP / メール（セキュリティ複雑性）
- クラウドストレージ（S3, GCS）
- Hop Server / クラスタ実行
