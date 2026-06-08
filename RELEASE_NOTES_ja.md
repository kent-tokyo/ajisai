# Ajisai 0.1.0 リリースノート

**リリース日**: 2026-06-08  
**ステータス**: GA 候補（Phase 6B-6C 完了）

---

## 🎉 大きなマイルストーン: Electron GUI がライブ！

7 日間の集中開発を経て、**Phase 6B（GUI 移行）と Phase 6C（ポーランド・最適化）が完了しました**。Electron + React GUI は本番環境対応済みで、レガシー egui 実装のすべての機能が、モダンで高パフォーマンスなデスクトップアプリで利用可能になりました。

---

## 0.1.0 の新機能

### GUI（Electron + React）
✅ **JSON-RPC サイドカーサーバー**
- Rust コアがバックグラウンドプロセスとして実行
- stdin/stdout JSON-RPC プロトコル（外部依存なし）
- 6 つのコアメソッド: ping, get_transforms, run_pipeline, validate, load, save

✅ **ビジュアル パイプライン エディタ**
- @xyflow/react v12 による DAG キャンバス
- ノードのドラッグ・ドロップ配置
- リアルタイムエッジ描画・検証
- ズーム・パン・フィット操作

✅ **トランスフォームパレット**
- 50+ トランスフォーム（I/O、Transform、Join/Lookup、Variables/Flow でカテゴリ分類）
- クリックしてキャンバスにノード追加
- 各トランスフォーム型向けデフォルトコンフィグ

✅ **トランスフォーム別フォーム定義**
- 50 トランスフォームに型付きコンフィグフォーム
- フィールド型: 文字列、数値、ブール値、セレクト、テキストエリア、JSON エディタ
- リアルタイム検証
- 不明な型は JSON エディタにフォールバック

✅ **Undo/Redo スタック**
- Ctrl+Z / Ctrl+Y キーボードショートカット
- 最大 50 スナップショット履歴（egui 互換）
- 自動スタック管理

✅ **パイプライン実行**
- リアルタイム進捗トラッキング
- ノードステータスインジケータ（アイドル・実行中・完了・エラー）
- ストリーミングログ出力
- Rust コアとの自動同期

✅ **自動保存**
- localStorage 永続化
- アプリ再起動時にパイプライン復元
- 誤った作業喪失をなくす

✅ **メニューバー**
- File メニュー: 新規、開く、保存、終了
- Pipeline メニュー: 実行、ログクリア
- 完全なキーボードショートカットサポート（Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+R）

### Rust コア（crates/server）
- IPC ブリッジ用 `ajisai-server` バイナリ
- Edition 2024 対応
- フレームセーフな JSON-RPC 用単一 stdout ライター
- 完全なストリーミング対応

### ビルド・配布
- electron-builder 統合
- macOS（DMG、ZIP）、Windows（NSIS、ポータブル）、Linux（AppImage、DEB）
- electron-updater での自動更新対応
- GitHub Releases 設定

---

## パフォーマンス

**バンドルサイズ**
- JavaScript: 606.94 kB（@xyflow/react、React、Zustand を含む）
- CSS: 32.82 kB（ダークテーム + コンポーネントスタイル）
- 合計: 約 640 kB（gzip 圧縮時 約 180 kB）

**最適化**
- Canvas コンポーネントに React.memo()
- 不要なレンダリングを防ぐ useMemo() フック
- 50+ ノードパイプラインを効率的に処理

---

## 互換性

✅ **後方互換性**
- レガシー egui GUI は引き続きビルド可能（`cargo build --workspace`）
- すべての .hpl、.json パイプラインファイルは変更なしで動作
- Rust CLI（`ajisai-cli`）は影響なし

✅ **Apache Hop 互換性**
- .hpl ファイルをネイティブで読み書き
- .hwf、.ktr、.dtsx フォーマット対応（hop-compat 経由）
- すべての 50+ トランスフォームが互換

---

## はじめ方

### 開発モード
```bash
cd electron
npm install
npm run dev
```

ホットリロード対応・DevTools 付きで Electron アプリを起動します。

### 配布向けビルド
```bash
cd electron
npm run build
# 出力: out/Ajisai-0.1.0.dmg (macOS), Ajisai-0.1.0.exe (Windows), ajisai-0.1.0.AppImage (Linux)
```

### CLI（GUI なし）
```bash
cargo run --bin ajisai-cli -- run -p path/to/pipeline.hpl
```

---

## 既知の制限

- コード署名は未実装（署名なしビルドのみ）
- 自動更新には GitHub リリースへのバイナリアップロードが必要
- 進捗通知は MVP（ノード単位の開始/終了、行単位ではない）
- 50 トランスフォームカテゴリ表示は静的（formDescriptors.ts）

---

## 次のステップ（Phase 6D+）

### Phase 6D: egui GUI 廃止
- レガシー `crates/gui` をワークスペースからアーカイブ
- リファレンスブランチに移動

### Phase 7+: 将来の拡張
- カスタムトランスフォームスキーマジェネレータ
- パイプライン実行メトリクスダッシュボード
- データ系統追跡
- 高度な分析機能

---

## 技術詳細

### アーキテクチャ
```
ajisai-cli (Rust)
ajisai-server (Rust) ← Electron (React) via JSON-RPC stdio
ajisai-core (Rust)
ajisai-transforms (Rust, 50+ トランスフォーム)
ajisai-hop-compat (Rust, .hpl/.hwf/.ktr/.dtsx)
```

### IPC プロトコル (JSON-RPC 2.0)
```
リクエスト:  {"id": 1, "method": "run_pipeline", "params": {...}}
レスポンス: {"id": 1, "result": {...}}
通知:      {"method": "pipeline/progress", "params": {...}}
```

### 状態管理（Zustand）
- シングルストア: `usePipelineStore`
- イミュータブル更新と自動 Undo/Redo
- localStorage 自動同期
- Rust モデルから生成した TypeScript 型

---

## 貢献者

- **Rust コア**: Phase 1-5 フル実装（50+ トランスフォーム完了）
- **Electron GUI**: Phase 6B-6C（Canvas、フォーム、Undo/Redo、自動保存）
- **インフラ**: JSON-RPC ブリッジ、electron-builder、GitHub Releases

---

## ライセンス

MIT OR Apache-2.0

---

## フィードバック・報告

- **バグ報告**: GitHub Issues
- **機能リクエスト**: GitHub Discussions
- **貢献**: Fork + Pull Request を歓迎します！

---

**バージョン**: 0.1.0  
**リリースタイプ**: GA 候補  
**ビルド日**: 2026-06-08  
**ステータス**: ✅ テスト・フィードバック準備完了
