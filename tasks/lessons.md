# 教訓・設計メモ

## アーキテクチャ決定

### Row ベース vs Batch ベース
- **決定**: 行ベースを基本とする
- **理由**: Apache Hop の行単位セマンティクスとの互換性維持。Polars/DataFusion は impedance mismatch が大きい
- **例外**: SortRows / Deduplicate など内部でバッチが必要な Transform は内部で rayon を使う

### 並行処理モデル
- **決定**: tokio task per node + bounded mpsc channel でバックプレッシャー
- **理由**: I/O バウンドは tokio、CPU バウンドは rayon。tokio::task::spawn_blocking で橋渡し
- **バッファサイズ**: デフォルト1024行。メモリ vs スループットのトレードオフ

### GUI フレームワーク
- **決定**: egui + eframe
- **理由**: 純Rust、即時モード、クロスプラットフォーム、WebAssembly対応
- **代替**: Slint（DSL学習コスト高）、Tauri（Web知識必要）は見送り

### .hpl/.hwf 解析
- **決定**: quick-xml で XML を中間表現に変換してから ajisai-core の型にマッピング
- **理由**: Apache Hop の .hpl は XML 形式（Kettle/PDI 由来）
- **ネイティブ形式**: JSON（serde_json）。.hpl インポート/エクスポートは ajisai-hop-compat が担当

### i18n
- **決定**: rust-i18n（YAMLベース、`t!("key")` マクロ）
- **理由**: 日英2言語・複数形なし → fluent-rs は over-engineering

## 実装上の注意点

### TransformFactory の型
- `Arc<dyn Fn(serde_json::Value) -> Result<Box<dyn Transform>> + Send + Sync>` を使用
- serde_json::Value を設定として受け取ることで、hop-compat からの変換が容易になる

### Pipeline の topological_order
- ソースノード（入力エッジなし）を先頭に並べ、BFS で順序を決定
- 循環グラフの検出は現時点では未実装（Apache Hop 側が保証する前提）

### engine.rs のファンアウト
- 1つのノードが複数の後続ノードに接続する場合、Row を clone してそれぞれに送信
- Row は Arc<RowSchema> を共有するため、clone コストは values の Vec<Value> のみ

### Rhai の `sync` feature
- `rhai = { version = "1", features = ["sync"] }` が必須
- デフォルトビルドは `Rc<...>` ベースの型を使用するため `Send` が実装されない
- `Transform` trait は `Send` を要求するため、`sync` feature なしではコンパイルエラーになる
- `ScriptStep` 実装時にこの制約を確認済み

### TransformRegistry の配置
- `TransformRegistry` は `ajisai-transforms` ではなく `ajisai-core` に置く
- **理由**: `ajisai-hop-compat` は `ajisai-transforms` に依存しており、もし `ajisai-transforms` が `ajisai-hop-compat` に依存するとクレート間の循環依存が生じる
- `ajisai-core ← ajisai-hop-compat ← ajisai-transforms` という一方向依存を維持するため、Registry は `ajisai-core` に配置した
- `PipelineExecutor`（Phase 5E）の実装時にこのアーキテクチャ変更を実施

### `cargo fmt --check` は CI で必須
- コンパイルとテストが通っても `cargo fmt --check` が失敗すると CI が red になる
- コミット前に必ず `cargo fmt --all` を実行すること
- GitHub Actions の `ci.yml` に `cargo fmt --check` ステップが含まれている
